#!/usr/bin/env python3
"""Rapporto del benchmark di tendenza (bench.sh; spec, sezione 7, gate 5).

    bench-report.py DIR...

Ogni DIR e' un run settimanale, in ordine cronologico, con uno o piu' results.json
(<etichetta>/results.json). Stampa Markdown per il summary del job: la tabella
dell'ultimo run, il confronto tra i kernel misurati nello stesso run (per esempio -O3
contro la variante o2: e' il confronto che decide, perche' i runner GitHub cambiano CPU
da un run all'altro) e un grafico Mermaid per metrica con l'andamento tra i run del
kernel di riferimento (il primo di ogni run).
"""

import json
import sys
from pathlib import Path

# Verso "meglio" di ogni metrica: serve al segno del confronto A/B.
HIGHER_IS_BETTER = {
    "hackbench_s": False,
    "schbench_wakeup_p99_us": False,
    "schbench_rps_p50": True,
    "fio_null_read_iops": True,
    "netperf_tcp_stream_mbps": True,
    "netperf_tcp_rr_tps": True,
}


def load(run_dir):
    """[results] di un run: il primo e' il kernel di riferimento (published)."""
    results = [json.loads(p.read_text()) for p in sorted(run_dir.rglob("results.json"))]
    results.sort(
        key=lambda r: (
            r["label"] != "published" and not r["label"].startswith("7"),
            r["label"],
        )
    )
    return results


def fmt(v):
    return f"{v:,.0f}" if abs(v) >= 100 else f"{v:.2f}"


def main():
    runs = [(Path(d), load(Path(d))) for d in sys.argv[1:]]
    runs = [(d, r) for d, r in runs if r]
    if not runs:
        sys.exit("nessun results.json")
    latest_dir, latest = runs[-1]
    ref = latest[0]
    out = [
        f"## Benchmark {ref['kver']} ({ref['date']})",
        "",
        f"QEMU/KVM {ref['vcpus']} vCPU, {ref['mem_mib']} MiB, {ref['seconds_per_test']} s per prova, host `{ref['host_cpu']}`",
        "",
    ]
    # Ultimo run: una colonna per kernel misurato; con piu' kernel, il delta verso il primo.
    labels = [r["label"] for r in latest]
    out.append(
        "| metrica | "
        + " | ".join(f"`{l}`" for l in labels)
        + (" | delta | " if len(latest) > 1 else " |")
    )
    out.append(
        "| --- |" + " --- |" * len(labels) + (" --- |" if len(latest) > 1 else "")
    )
    for metric in ref["metrics"]:
        row = [metric] + [
            fmt(r["metrics"][metric]["value"]) + " " + r["metrics"][metric]["unit"]
            if metric in r["metrics"]
            else "n/d"
            for r in latest
        ]
        if len(latest) > 1:
            deltas = []
            for r in latest[1:]:
                if metric in r["metrics"] and ref["metrics"][metric]["value"]:
                    pct = (
                        r["metrics"][metric]["value"] / ref["metrics"][metric]["value"]
                        - 1
                    ) * 100
                    better = (pct > 0) == HIGHER_IS_BETTER.get(metric, True)
                    deltas.append(f"{pct:+.1f}% ({'meglio' if better else 'peggio'})")
            row.append(", ".join(deltas) or "n/d")
        out.append("| " + " | ".join(row) + " |")
    if len(latest) > 1:
        out += [
            "",
            f"Il delta e' della variante rispetto a `{ref['label']}`, nello stesso run e sulla stessa macchina.",
        ]
    # Andamento del kernel di riferimento tra i run: un grafico per metrica.
    history = [r[0] for _, r in runs]
    if len(history) > 1:
        out += ["", f"### Andamento su {len(history)} run", ""]
        dates = [h["date"] for h in history]
        for metric, info in ref["metrics"].items():
            values = [
                h["metrics"][metric]["value"] if metric in h["metrics"] else 0
                for h in history
            ]
            direction = (
                "piu' alto e' meglio"
                if HIGHER_IS_BETTER.get(metric, True)
                else "piu' basso e' meglio"
            )
            out += [
                "```mermaid",
                "xychart-beta",
                f'    title "{metric} ({info["unit"]}, {direction})"',
                "    x-axis [" + ", ".join(f'"{d}"' for d in dates) + "]",
                f'    y-axis "{info["unit"]}"',
                "    line [" + ", ".join(f"{v:g}" for v in values) + "]",
                "```",
                "",
            ]
        out.append(
            "I runner GitHub cambiano CPU da un run all'altro (`host_cpu` in results.json): l'andamento e' indicativo, la decisione sta nel confronto A/B dello stesso run."
        )
    print("\n".join(out))


if __name__ == "__main__":
    main()
