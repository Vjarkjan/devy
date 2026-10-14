#!/usr/bin/env python3
"""Convierte los assets del ejemplo con las herramientas del workspace."""
from pathlib import Path
import json
import subprocess

root = Path(__file__).resolve().parent.parent
metadata = json.loads(subprocess.check_output(["cargo", "+stable", "metadata", "--no-deps", "--format-version=1"], cwd=root))
bin_dir = Path(metadata["target_directory"]) / "debug"
out = root / "output/nitrofs"
out.mkdir(parents=True, exist_ok=True)
def run(tool, *args):
    subprocess.run([str(bin_dir / tool), *map(str, args)], cwd=root, check=True)
for path in sorted((root / "assets").glob("*.obj")):
    run("obj2dl", "--input", path, "--output", out / path.with_suffix(".dl").name, "--center")
for path in sorted((root / "assets/sprites").glob("*.png")):
    target = out / "sprites" / path.with_suffix(".sprite").name
    target.parent.mkdir(exist_ok=True)
    run("png2sprite", path, target)
for kind, ext in [("tiled", ".bg"), ("bitmap", ".bbg")]:
    target_dir = out / "backgrounds" / kind
    target_dir.mkdir(parents=True, exist_ok=True)
    for path in sorted((root / "assets/backgrounds" / kind).glob("*.png")):
        run("png2bg", kind, path, target_dir / path.with_suffix(ext).name)
run("wav2bank", "--input", root / "audio", "--output", out / "soundbank.bin", "--ids", root / "output/sounds.rs")
run("scene2bin", "--levels", root / "assets/levels", "--out", out / "levels", "--assets", root / "assets", "--prefabs", root / "assets/prefabs")
print(f"NitroFS: {out}")
