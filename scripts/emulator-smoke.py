#!/usr/bin/env python3
"""Prueba gráfica de hello/drawing en una instancia propia de Xvfb y DeSmuME.

Requiere Python con Pillow, un DeSmuME CLI basado en SDL2, Xvfb, xdotool,
ImageMagick y cc. El shim fuerza sólo la presentación SDL por software.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--emulator", required=True)
    parser.add_argument("--xvfb", default=shutil.which("Xvfb"))
    parser.add_argument("--digraph-rom", type=Path, help="ROM del repositorio independiente digraph-ds")
    parser.add_argument("--examples", nargs="+", default=["drawing", "hello"], choices=["drawing", "hello", "hardware", "paint", "digraph"])
    args = parser.parse_args()
    if not args.xvfb:
        parser.error("Indica --xvfb /ruta/a/Xvfb")
    out = ROOT / "output/emulator"
    out.mkdir(parents=True, exist_ok=True)
    shim = out / "sdl_headless.so"
    subprocess.run(["cc", "-shared", "-fPIC", "-O2", "-Wall", "-Wextra", "-Werror",
                    str(ROOT / "scripts/emulator_sdl.c"), "-o", str(shim), "-ldl"], check=True)
    # -displayfd selecciona un servidor libre sin reutilizar el del usuario.
    read_fd, write_fd = os.pipe()
    server = None
    report = {}
    try:
        with (out / "xvfb.log").open("w") as log:
            server = subprocess.Popen([args.xvfb, "-displayfd", str(write_fd), "-screen", "0",
                                       "800x600x24", "-nolisten", "tcp"], pass_fds=(write_fd,),
                                      stdout=log, stderr=log)
        os.close(write_fd)
        with os.fdopen(read_fd) as pipe:
            display = ":" + pipe.readline().strip()
        assert display != ":", "Xvfb no pudo arrancar; revisa xvfb.log"
        env = dict(os.environ, DISPLAY=display, SDL_RENDER_DRIVER="software",
                   LD_PRELOAD=str(shim), SDL_AUDIODRIVER="dummy",
                   XDG_CONFIG_HOME=str(out / "config"), XDG_DATA_HOME=str(out / "data"))
        for name in args.examples:
            logpath = out / f"{name}.log"
            rom = ROOT / f"output/{name}.nds"
            if name == "digraph" and args.digraph_rom:
                rom = args.digraph_rom.resolve()
            rom_hash = hashlib.sha256(rom.read_bytes()).hexdigest()
            with logpath.open("w") as log:
                emulator = subprocess.Popen([args.emulator, "--disable-sound", "--num-cores", "1",
                                             "--scale", "1", "--nojoy", str(rom)],
                                            cwd=out, env=env, stdout=log, stderr=log)
            try:
                time.sleep(5)
                if emulator.poll() is not None:
                    raise RuntimeError(logpath.read_text())
                windows = subprocess.check_output(["xdotool", "search", "--onlyvisible", "--pid",
                                                   str(emulator.pid)], env=env).splitlines()
                window = windows[-1].decode()

                def capture(label):
                    path = out / f"{name}-{label}.png"
                    subprocess.run(["import", "-window", window, str(path)], env=env, check=True)
                    image = Image.open(path).convert("RGB")
                    assert image.size == (256, 384), image.size
                    return image

                first = capture("first")
                if name == "drawing":
                    expected = Image.open(ROOT / "output/preview/top.ppm").convert("RGB")
                    assert ImageChops.difference(first.crop((0, 0, 256, 192)), expected).getbbox() is None
                    time.sleep(1)
                    second = capture("moving")
                    bounds = ImageChops.difference(first, second).getbbox()
                    assert bounds and bounds[1] >= 192, "El cuadro inferior no se mueve"
                    subprocess.run(["xdotool", "mousemove", "--window", window, "80", "240",
                                    "mousedown", "1"], env=env, check=True)
                    time.sleep(1)
                    touched = capture("touch")
                    subprocess.run(["xdotool", "mouseup", "1"], env=env, check=True)
                    yellow = (247, 198, 82)
                    points = [(x, y) for y in range(192, 384) for x in range(256)
                              if touched.getpixel((x, y)) == yellow]
                    assert len(points) == 64, "El cuadro táctil no tiene el tamaño esperado"
                    x, y = min(p[0] for p in points), min(p[1] for p in points)
                    # La calibración del firmware puede redondear un píxel.
                    assert abs(x - 80) <= 2 and abs(y - 240) <= 2, "No se recibió el lápiz"
                    report[name] = {"top_matches_host": True, "animated": True,
                                    "touch": True, "touch_position": [x, y - 192]}
                elif name == "hello":
                    subprocess.run(["xdotool", "windowfocus", window, "keydown", "x"], env=env, check=True)
                    time.sleep(0.2)
                    subprocess.run(["xdotool", "keyup", "x"], env=env, check=True)
                    time.sleep(0.5)
                    second = capture("button-a")
                    assert first.getpixel((128, 288)) == (24, 49, 82)
                    assert second.getpixel((128, 288)) == (24, 140, 107), "No se recibió A (tecla X del emulador)"
                    report[name] = {"boot": True, "button_a": True}
                elif name == "hardware":
                    time.sleep(1)
                    second = capture("rotated")
                    assert ImageChops.difference(first.crop((0,0,256,192)), second.crop((0,0,256,192))).getbbox(), "La geometría 3D no cambia"
                    subprocess.run(["xdotool", "mousemove", "--window", window, "80", "332", "mousedown", "1"], env=env, check=True)
                    time.sleep(1)
                    touched = capture("sprite-touch")
                    subprocess.run(["xdotool", "mouseup", "1"], env=env, check=True)
                    region = (70, 322, 112, 372)
                    assert ImageChops.difference(second.crop(region), touched.crop(region)).getbbox(), "El sprite no respondió al lápiz"
                    report[name] = {"boot": True, "animated_3d": True, "sprite_touch": True}
                elif name == "paint":
                    assert first.crop((0,0,256,192)).getbbox(), "La consola superior quedó oculta"
                    for i in range(5):
                        subprocess.run(["xdotool", "mousemove", "--window", window, str(60+i*10), str(270+i*10)], env=env, check=True)
                        if i == 0:
                            subprocess.run(["xdotool", "mousedown", "1"], env=env, check=True)
                        time.sleep(0.2)
                    subprocess.run(["xdotool", "mouseup", "1"], env=env, check=True)
                    time.sleep(0.5)
                    painted = capture("stroke")
                    region = (48, 256, 128, 330)
                    assert ImageChops.difference(first.crop(region), painted.crop(region)).getbbox(), "El canvas no recibió el trazo"
                    report[name] = {"boot": True, "touch_stroke": True}
                else:
                    assert len(first.crop((0,0,256,192)).getcolors(49152)) > 4, "No apareció el grafo"
                    assert len(first.crop((0,192,256,384)).getcolors(49152)) > 4, "No apareció la interfaz inferior"
                    report[name] = {"boot": True, "two_screens": True}
                assert hashlib.sha256(rom.read_bytes()).hexdigest() == rom_hash, "La ROM cambió durante la prueba"
                report[name]["rom_sha256"] = rom_hash
                print(json.dumps({name: report[name]}), flush=True)
            finally:
                stop(emulator)
        (out / ("checks-" + "-".join(args.examples) + ".json")).write_text(json.dumps(report, indent=2) + "\n")
    finally:
        stop(server)


def stop(process):
    if process is not None and process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


if __name__ == "__main__":
    main()
