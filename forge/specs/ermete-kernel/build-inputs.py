#!/usr/bin/env python3
"""Gli input della build del kernel come JSON (docs/architecture/doc_kernel_build.md,
sezione 3 passo 8 e sezione 7): e' il predicato dell'attestazione dei pin che publish
allega alle immagini, e la chiave con cui il job inputs riconosce un kernel gia'
costruito da questi identici input. Entra solo cio' che cambia gli RPM: pin, manifest
delle sorgenti, delta di config, patch, regole di merge, build.sh e l'ambiente. Non
cmdline, boot/, retention.sh o il workflow: cambiarli non deve ricompilare nulla."""

import hashlib
import json
import pathlib
import re

k = pathlib.Path(__file__).resolve().parent


def sha(name):
    return hashlib.sha256((k / name).read_bytes()).hexdigest()


pins = dict(re.findall(r"^(\w+)=(.*)$", (k / "pins.env").read_text(), re.M))
base = re.search(r"^FROM (\S+)", (k / "builder/Containerfile").read_text(), re.M).group(
    1
)
print(
    json.dumps(
        {
            "pins": pins,
            "sources_sha256": sha("SOURCES/sources.sha256"),
            "kernel_local_sha256": sha("kernel-local"),
            "patches_list_sha256": sha("patches.list"),
            "patches_sha256": {
                p.name: sha(f"patches/{p.name}")
                for p in sorted((k / "patches").glob("*.patch"))
            },
            "fedora_wins_sha256": sha("fedora-wins.list"),
            "build_sh_sha256": sha("build.sh"),
            "builder_base": base,
            "containerfile_sha256": sha("builder/Containerfile"),
        },
        indent=2,
    )
)
