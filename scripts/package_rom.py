#!/usr/bin/env python3
"""Empaqueta los ejecutables que Cargo acaba de compilar con el ARM7 del SDK."""
import json
import os
from pathlib import Path
import subprocess
import sys

sdk = Path(os.environ["BLOCKSDS"])
arm7 = Path(os.environ.get("DEVY_ARM7", sdk / "sys/arm7/main_core/arm7_maxmod.elf"))
nitrofs = os.environ.get("DEVY_NITROFS")
extra = []
if nitrofs:
    root = Path(nitrofs).resolve()
    if not root.is_dir():
        raise SystemExit(f"No existe DEVY_NITROFS: {root}")
    extra = ["-d", str(root)]

output = Path(sys.argv[2])
executables = {}
for line in Path(sys.argv[1]).read_text().splitlines():
    item = json.loads(line)
    if item.get("reason") == "compiler-artifact" and item.get("executable"):
        executables[item["target"]["name"]] = Path(item["executable"])
if not executables:
    raise SystemExit("Cargo no produjo un ejecutable; selecciona --example o --bin")
for name, elf in executables.items():
    rom = output / f"{name}.nds"
    subprocess.run([str(sdk / "tools/ndstool/ndstool"), "-c", str(rom),
                    "-9", str(elf), "-7", str(arm7),
                    "-g", "DEVY", "00", "DEVY"] + extra, check=True)
    print(f"ROM: {rom}")
