#!/usr/bin/env python3
"""Confronto di riproducibilita' tra due build dello stesso pin (spec, sezione 3 passo 8 e
sezione 7 gate 6): A e' il kernel pubblicato, B la ricostruzione del job settimanale.

    repro.py --a DIR --b DIR --out DIR

DIR contiene gli RPM del kernel (kernel-core, kernel-modules*) e, per B, kernel-devel: da
li' viene scripts/extract-vmlinux. Confronta: `config` (byte per byte), `System.map`,
`vmlinuz` (byte per byte, poi il vmlinux estratto sezione per sezione, offset e size da llvm-readelf) e
ogni modulo senza la firma appesa. La chiave che firma moduli e immagine nasce e muore in
ogni build (CONFIG_MODULE_SIG con chiave generata), quindi la firma dei .ko e il
certificato dentro vmlinux (.init.data) sono differenze attese: il resto no. Scrive
OUT/summary.md e OUT/results.json; esce 1 se c'e' una differenza non attesa.
"""

import argparse
import gzip
import hashlib
import lzma
import json
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

SIG_MARKER = b"~Module signature appended~\n"
# Sezioni di vmlinux in cui la chiave effimera lascia traccia: il certificato in
# system_certificate_list (.init.data) e il build-id, che ne dipende (.notes).
EXPECTED_SECTIONS = {".init.data", ".notes"}


def run(*cmd, **kw):
    done = subprocess.run(cmd, capture_output=True, text=True, **kw)
    if done.returncode != 0:
        sys.exit(f"{cmd[0]}: {done.stderr.strip()}")
    return done.stdout


def extract(rpms, dest, patterns):
    """rpm2cpio | cpio dei soli percorsi che servono, da ogni RPM che li ha."""
    dest.mkdir(parents=True, exist_ok=True)
    for rpm in rpms:
        with subprocess.Popen(["rpm2cpio", str(rpm)], stdout=subprocess.PIPE) as p:
            subprocess.run(
                ["cpio", "-idm", "--quiet", *patterns],
                stdin=p.stdout,
                cwd=dest,
                check=True,
            )


def sha(data):
    return hashlib.sha256(data).hexdigest()


def module_payload(data):
    """Il modulo senza la firma: [payload][signer][key id][firma][struct 12][marker]."""
    if not data.endswith(SIG_MARKER):
        return data
    info = data[-len(SIG_MARKER) - 12 : -len(SIG_MARKER)]
    _algo, _hash, _id_type, signer_len, key_id_len, _p1, _p2, _p3, sig_len = (
        struct.unpack(">8BI", info)
    )
    return data[: len(data) - len(SIG_MARKER) - 12 - sig_len - signer_len - key_id_len]


def module_bytes(path):
    """Il modulo decompresso (Fedora spedisce .ko.xz: la firma sta dentro il flusso)."""
    data = path.read_bytes()
    if path.suffix == ".xz":
        return lzma.decompress(data)
    if path.suffix == ".gz":
        return gzip.decompress(data)
    if path.suffix == ".zst":
        sys.exit(f"{path.name}: moduli .zst, manca il decompressore")
    return data


def modules(tree):
    return {
        p.relative_to(tree).as_posix(): sha(module_payload(module_bytes(p)))
        for p in tree.rglob("*.ko*")
    }


def sections(vmlinux):
    """{sezione: sha256 del contenuto}: offset e size da llvm-readelf, byte dal file."""
    data = vmlinux.read_bytes()
    (elf,) = json.loads(
        run("llvm-readelf", "-S", "--elf-output-style=JSON", str(vmlinux))
    )
    out = {}
    for entry in elf["Sections"]:
        section = entry["Section"]
        name, kind = section["Name"]["Name"], section["Type"]["Name"]
        if not name or kind in ("SHT_NULL", "SHT_NOBITS"):
            continue
        out[name] = sha(data[section["Offset"] : section["Offset"] + section["Size"]])
    return out


def kver(tree):
    (mods,) = [p for p in (tree / "lib/modules").iterdir()]
    return mods.name


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--a", required=True, type=Path)
    ap.add_argument("--b", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp())
    trees = {}
    for label, src in (("a", args.a), ("b", args.b)):
        rpms = sorted(src.rglob("kernel-core-*.rpm")) + sorted(
            src.rglob("kernel-modules*.rpm")
        )
        if not any(r.name.startswith("kernel-core-") for r in rpms):
            sys.exit(f"{src}: kernel-core-*.rpm assente")
        tree = work / label
        extract(rpms, tree, ["./lib/modules/*"])
        trees[label] = tree
    devel = sorted(args.b.rglob("kernel-devel-[0-9]*.rpm"))
    if not devel:
        sys.exit(
            f"{args.b}: kernel-devel-*.rpm assente (serve scripts/extract-vmlinux)"
        )
    extract(devel, work / "devel", ["./usr/src/kernels/*/scripts/extract-vmlinux"])
    (extract_vmlinux,) = list((work / "devel").rglob("extract-vmlinux"))
    extract_vmlinux.chmod(0o755)

    ka, kb = kver(trees["a"]), kver(trees["b"])
    results = {
        "kver": {"a": ka, "b": kb},
        "files": {},
        "vmlinux_sections": {},
        "modules": {},
    }
    findings = []
    if ka != kb:
        findings.append(f"versioni diverse: {ka} e {kb}")
    for name in ("config", "System.map", "vmlinuz"):
        da = (trees["a"] / "lib/modules" / ka / name).read_bytes()
        db = (trees["b"] / "lib/modules" / kb / name).read_bytes()
        results["files"][name] = da == db
        if da != db and name != "vmlinuz":
            findings.append(f"{name} diverso")
    # vmlinuz differisce per costruzione (certificato): conta cosa differisce dentro.
    for label in ("a", "b"):
        vmlinux = work / f"vmlinux-{label}"
        with vmlinux.open("wb") as f:
            subprocess.run(
                [
                    str(extract_vmlinux),
                    str(
                        trees[label]
                        / "lib/modules"
                        / (ka if label == "a" else kb)
                        / "vmlinuz"
                    ),
                ],
                stdout=f,
                check=True,
            )
    sa, sb = (
        sections(work / "vmlinux-a"),
        sections(work / "vmlinux-b"),
    )
    differing = sorted(s for s in set(sa) | set(sb) if sa.get(s) != sb.get(s))
    results["vmlinux_sections"] = {
        "total": len(set(sa) | set(sb)),
        "differing": differing,
    }
    unexpected = [s for s in differing if s not in EXPECTED_SECTIONS]
    if unexpected:
        findings.append(
            f"sezioni di vmlinux diverse oltre a quelle attese: {' '.join(unexpected)}"
        )
    ma, mb = modules(trees["a"]), modules(trees["b"])
    only_a, only_b = sorted(set(ma) - set(mb)), sorted(set(mb) - set(ma))
    diff_mods = sorted(m for m in set(ma) & set(mb) if ma[m] != mb[m])
    results["modules"] = {
        "total": len(set(ma) | set(mb)),
        "only_a": only_a,
        "only_b": only_b,
        "differing": diff_mods,
    }
    if only_a or only_b:
        findings.append(
            f"moduli presenti in una sola build: {len(only_a)} solo in A, {len(only_b)} solo in B"
        )
    # Per i primi moduli diversi, quali sezioni differiscono: distingue codice diverso da
    # soli simboli, rilocazioni o build-id.
    mod_sections = {}
    for name in diff_mods[:3]:
        hashes = {}
        for label in ("a", "b"):
            ko = work / f"mod-{label}-{len(mod_sections)}.ko"
            ko.write_bytes(module_payload(module_bytes(trees[label] / name)))
            hashes[label] = sections(ko)
        mod_sections[name] = sorted(
            s
            for s in set(hashes["a"]) | set(hashes["b"])
            if hashes["a"].get(s) != hashes["b"].get(s)
        )
    results["modules"]["sections"] = mod_sections
    if diff_mods:
        findings.append(
            f"{len(diff_mods)} moduli con contenuto diverso (firma esclusa)"
        )
    results["findings"] = findings
    (args.out / "results.json").write_text(json.dumps(results, indent=2) + "\n")

    ok = "identico" if not findings else "DIVERSO"
    lines = [
        f"## Riproducibilita' {ka}: {ok}",
        "",
        "| oggetto | esito |",
        "| --- | --- |",
    ]
    for name, same in results["files"].items():
        lines.append(f"| `{name}` | {'identico' if same else 'diverso'} |")
    lines.append(
        f"| vmlinux, {results['vmlinux_sections']['total']} sezioni | {len(differing)} diverse: {' '.join(differing) or 'nessuna'} |"
    )
    lines.append(
        f"| moduli, {results['modules']['total']} | {len(diff_mods)} diversi senza firma, {len(only_a) + len(only_b)} presenti in una sola build |"
    )
    if findings:
        lines += ["", "Differenze non attese (un bug da aprire):", ""] + [
            f"- {f}" for f in findings
        ]
        if diff_mods:
            lines += ["", "| modulo | sezioni diverse |", "| --- | --- |"] + [
                f"| `{name.rsplit('/', 1)[-1]}` | {' '.join(secs) or 'nessuna'} |"
                for name, secs in mod_sections.items()
            ]
            lines += (
                ["", "<details><summary>moduli diversi</summary>", "", "```"]
                + diff_mods[:200]
                + ["```", "</details>"]
            )
    else:
        lines += [
            "",
            "Le sole differenze stanno dove la chiave di firma effimera lascia traccia (firma dei moduli, certificato in vmlinux).",
        ]
    (args.out / "summary.md").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    sys.exit(1 if findings else 0)


if __name__ == "__main__":
    main()
