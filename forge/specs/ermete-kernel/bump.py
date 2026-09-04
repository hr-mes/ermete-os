#!/usr/bin/env python3
"""Il bot di bump del kernel Ermete (docs/architecture/doc_kernel_build.md, sezione 8).

    bump.py check   stampa su stdout un JSON con i pin correnti, quelli nuovi e le note
    bump.py apply   riscrive pins.env, i FROM dei Containerfile e la tabella dei pin di
                    KERNEL.md; stampa su stdout il corpo della PR (Markdown)

Coppia kernel (spec, sezione 2): per la serie X.Y che Fedora (stable, F43 poi F44) e
CachyOS (release GitHub di CachyOS/linux) spediscono entrambe, il patch level X.Y.Z piu'
alto presente su entrambi i lati. KERNEL_CHANNEL=stable prende la serie piu' nuova
comune, lts la serie longterm. Senza coppia il kernel resta dov'e' e una nota lo dice.
Con la coppia si muovono anche il commit di testa di CachyOS/kernel-patches per la
serie e il commit di linux-cachyos/config vigente alla data della release CachyOS.
Fuori dal kernel: i tag NVIDIA (open su GitHub, legacy dall'indice di download) dentro
il ramo pinnato, e il digest dell'immagine base dei Containerfile. I manifesti degli
hash non sono qui: li scrivono `build.sh --stage manifest` e `nvidia.sh manifest`.
Solo libreria standard: gira sul runner GitHub senza installare nulla.
"""

import json
import os
import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
PINS = HERE / "pins.env"
CONTAINERFILES = [HERE / d / "Containerfile" for d in ("builder", "boot", "nvidia")]
KERNEL_MD = HERE / "KERNEL.md"
FEDORA_RELEASES = ("F43", "F44")  # in ordine di preferenza per lo stesso patch level
LTS_SERIES = "6.18"  # KERNEL_CHANNEL=lts: la longterm che Fedora e CachyOS mantengono
BODHI = "https://bodhi.fedoraproject.org/updates/"
NVIDIA_INDEX = "https://download.nvidia.com/XFree86/Linux-x86_64/"
NVR_RE = re.compile(r"^kernel-(\d+\.\d+\.\d+)-(\d+)\.fc(\d+)$")
CACHY_TAG_RE = re.compile(r"^cachyos-(\d+\.\d+\.\d+)-(\d+)$")
FROM_RE = re.compile(r"^FROM (\S+?):(\S+?)@(sha256:[0-9a-f]{64})$", re.M)
MANIFEST_ACCEPT = ", ".join(
    [
        "application/vnd.oci.image.index.v1+json",
        "application/vnd.docker.distribution.manifest.list.v2+json",
        "application/vnd.oci.image.manifest.v1+json",
        "application/vnd.docker.distribution.manifest.v2+json",
    ]
)


def vtuple(version):
    return tuple(int(x) for x in version.split("."))


def series(version):
    return ".".join(version.split(".")[:2])


def http(url, headers=None, method="GET"):
    req = urllib.request.Request(url, headers=headers or {}, method=method)
    with urllib.request.urlopen(req, timeout=60) as resp:
        return resp.headers, resp.read()


def gh(url):
    """Una richiesta all'API GitHub, con il token del job se c'e' (60/ora senza)."""
    headers = {"Accept": "application/vnd.github+json"}
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    head, body = http(url, headers)
    return head, json.loads(body)


def github(endpoint, **params):
    """Gli elementi di un endpoint lista, pagina dopo pagina."""
    url = f"https://api.github.com/{endpoint}?{urllib.parse.urlencode(params)}"
    while url:
        head, items = gh(url)
        yield from items
        link = head.get("Link", "")
        url = next(
            (m.group(1) for m in re.finditer(r'<([^>]+)>; rel="next"', link)), None
        )


def read_pins():
    return dict(re.findall(r"^(\w+)=(.*)$", PINS.read_text(), re.M))


# --- kernel -----------------------------------------------------------------------


def fedora_kernels():
    """{patch level: NVR} delle build kernel stable; F43 prima di F44 a parita' di versione."""
    found = {}
    for release in FEDORA_RELEASES:
        page, pages = 1, 1
        while page <= pages:
            query = {
                "packages": "kernel",
                "releases": release,
                "status": "stable",
                "rows_per_page": 100,
                "page": page,
            }
            data = json.loads(http(f"{BODHI}?{urllib.parse.urlencode(query)}")[1])
            pages, page = data["pages"], page + 1
            for update in data["updates"]:
                for build in update["builds"]:
                    m = NVR_RE.match(build["nvr"])
                    if not m:
                        continue
                    version, rel, fc = m.groups()
                    if version not in found or (
                        found[version][0] == release and int(rel) > found[version][1]
                    ):
                        found[version] = (release, int(rel), f"{version}-{rel}.fc{fc}")
    return {v: nvr for v, (_, _, nvr) in found.items()}


def cachyos_releases():
    """{patch level: (tag, published_at)} dell'ultima release di CachyOS/linux per versione."""
    found = {}
    for rel in github("repos/CachyOS/linux/releases", per_page=100):
        m = CACHY_TAG_RE.match(rel["tag_name"])
        if not m or rel["draft"] or rel["prerelease"]:
            continue
        version, n = m.group(1), int(m.group(2))
        if version not in found or n > found[version][0]:
            found[version] = (n, rel["tag_name"], rel["published_at"])
    return {v: (tag, at) for v, (_, tag, at) in found.items()}


def kernel_pair(pins, notes):
    """(versione, NVR Fedora, tag CachyOS, data della release) della coppia scelta, o None."""
    fedora, cachy = fedora_kernels(), cachyos_releases()
    common = sorted(set(fedora) & set(cachy), key=vtuple)
    if pins["KERNEL_CHANNEL"] == "lts":
        common = [v for v in common if series(v) == LTS_SERIES]
    elif pins["KERNEL_CHANNEL"] != "stable":
        sys.exit(f"KERNEL_CHANNEL={pins['KERNEL_CHANNEL']}: atteso stable o lts")
    newest_fedora, newest_cachy = max(fedora, key=vtuple), max(cachy, key=vtuple)
    if not common:
        notes.append(
            f"kernel: nessuna coppia Fedora/CachyOS (Fedora {newest_fedora}, CachyOS {newest_cachy}); il kernel resta {pins['FEDORA_KERNEL_NVR']}"
        )
        return None
    version = common[-1]
    if vtuple(newest_fedora) > vtuple(version) or vtuple(newest_cachy) > vtuple(
        version
    ):
        notes.append(
            f"kernel: la coppia piu' alta e' {version} (Fedora {fedora[version]}, CachyOS {cachy[version][0]}); "
            f"oltre, senza coppia: Fedora {newest_fedora}, CachyOS {newest_cachy}"
        )
    return version, fedora[version], cachy[version][0], cachy[version][1]


def head_commit(repo, path, until=None):
    params = {"path": path, "per_page": 1}
    if until:
        params["until"] = until
    return next(github(f"repos/{repo}/commits", **params))["sha"]


# --- NVIDIA e immagine base ----------------------------------------------------------


def nvidia_open(current):
    """(tag, commit) piu' alto di NVIDIA/open-gpu-kernel-modules nel ramo (major) pinnato."""
    major = current.split(".")[0]
    tags = [
        t["name"]
        for t in github("repos/NVIDIA/open-gpu-kernel-modules/tags", per_page=100)
    ]
    best = max(
        (t for t in tags if re.fullmatch(rf"{major}\.\d+(\.\d+)?", t)), key=vtuple
    )
    # L'endpoint dei commit dereferenzia anche un tag annotato: e' il commit che nvidia.sh verifica.
    return best, gh(
        f"https://api.github.com/repos/NVIDIA/open-gpu-kernel-modules/commits/{best}"
    )[1]["sha"]


def nvidia_legacy(current):
    """La versione piu' alta del ramo pinnato nell'indice di download di NVIDIA."""
    major = current.split(".")[0]
    index = http(NVIDIA_INDEX)[1].decode()
    return max(re.findall(rf"href='({major}\.\d+(?:\.\d+)?)/'", index), key=vtuple)


def image_digest(image, tag):
    """Il digest che `podman pull image:tag` risolve: quello del manifest (index) del tag."""
    registry, _, name = image.partition("/")
    head, _ = http(
        f"https://{registry}/v2/{name}/manifests/{tag}",
        {"Accept": MANIFEST_ACCEPT},
        method="HEAD",
    )
    digest = head.get("Docker-Content-Digest", "")
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", digest):
        sys.exit(f"{image}:{tag}: digest assente nella risposta del registro")
    return digest


def base_images():
    """{"immagine:tag": digest pinnato} dai FROM dei Containerfile."""
    found = {}
    for cf in CONTAINERFILES:
        for m in FROM_RE.finditer(cf.read_text()):
            found[f"{m.group(1)}:{m.group(2)}"] = m.group(3)
    return found


# --- check / apply ---------------------------------------------------------------------


def compute():
    pins = read_pins()
    new, notes = {}, []
    pair = kernel_pair(pins, notes)
    if pair:
        version, nvr, tag, published = pair
        if nvr != pins["FEDORA_KERNEL_NVR"] or tag != pins["CACHYOS_RELEASE"]:
            new["FEDORA_KERNEL_NVR"], new["CACHYOS_RELEASE"] = nvr, tag
        # Il config di linux-cachyos vigente alla data della release: quello con cui CachyOS
        # ha spedito quel kernel, non la testa di oggi, che puo' essere della serie dopo.
        config = head_commit(
            "CachyOS/linux-cachyos", "linux-cachyos/config", until=published
        )
        if config != pins["CACHYOS_CONFIG_COMMIT"]:
            new["CACHYOS_CONFIG_COMMIT"] = config
        patches = head_commit("CachyOS/kernel-patches", series(version))
        if patches != pins["CACHYOS_PATCHES_COMMIT"]:
            new["CACHYOS_PATCHES_COMMIT"] = patches
    open_version, open_commit = nvidia_open(pins["NVIDIA_OPEN_VERSION"])
    if open_version != pins["NVIDIA_OPEN_VERSION"]:
        new["NVIDIA_OPEN_VERSION"], new["NVIDIA_OPEN_COMMIT"] = (
            open_version,
            open_commit,
        )
    legacy_version = nvidia_legacy(pins["NVIDIA_LEGACY_VERSION"])
    if legacy_version != pins["NVIDIA_LEGACY_VERSION"]:
        new["NVIDIA_LEGACY_VERSION"] = legacy_version
    images = {}
    for ref, pinned in base_images().items():
        digest = image_digest(*ref.rsplit(":", 1))
        if digest != pinned:
            images[ref] = {"old": pinned, "new": digest}
    return {
        "changed": bool(new or images),
        "pins": pins,
        "new": new,
        "images": images,
        "notes": notes,
    }


def pins_table(pins):
    rows = "\n".join(f"| `{k}` | `{v}` |" for k, v in pins.items())
    return f"<!-- pins:begin (tabella scritta da bump.py apply) -->\n| pin | valore |\n| --- | --- |\n{rows}\n<!-- pins:end -->"


def apply(result):
    text = PINS.read_text()
    for key, value in result["new"].items():
        text, n = re.subn(rf"^{key}=.*$", f"{key}={value}", text, flags=re.M)
        if n != 1:
            sys.exit(f"pins.env: {key} trovato {n} volte")
    PINS.write_text(text, newline="\n")
    for cf in CONTAINERFILES:
        content = cf.read_text()
        for ref, change in result["images"].items():
            content = content.replace(
                f"FROM {ref}@{change['old']}", f"FROM {ref}@{change['new']}"
            )
        cf.write_text(content, newline="\n")
    md, n = re.subn(
        r"<!-- pins:begin.*?<!-- pins:end -->",
        lambda _: pins_table(read_pins()),
        KERNEL_MD.read_text(),
        flags=re.S,
    )
    if n != 1:
        sys.exit("KERNEL.md: marcatori pins:begin/pins:end assenti")
    KERNEL_MD.write_text(md, newline="\n")


def body(result):
    lines = ["## Pin", "", "| pin | prima | dopo |", "| --- | --- | --- |"]
    lines += [
        f"| `{k}` | `{result['pins'][k]}` | `{v}` |" for k, v in result["new"].items()
    ]
    lines += [
        f"| `{ref}` | `{c['old'][7:19]}` | `{c['new'][7:19]}` |"
        for ref, c in result["images"].items()
    ]
    if result["notes"]:
        lines += ["", "## Note", ""] + [f"- {n}" for n in result["notes"]]
    return "\n".join(lines) + "\n"


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in ("check", "apply"):
        sys.exit(__doc__)
    result = compute()
    if sys.argv[1] == "check":
        print(json.dumps(result, indent=2))
        return
    apply(result)
    sys.stdout.write(body(result))


if __name__ == "__main__":
    main()
