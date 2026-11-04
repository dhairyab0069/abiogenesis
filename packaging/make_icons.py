#!/usr/bin/env python3
"""Draw the Abiogenesis app icon and generate platform icons.

Outputs: packaging/icon-1024.png, packaging/windows/app.ico and
packaging/macos/AppIcon.icns. The .icns step needs macOS (iconutil).
Requires Pillow. Run: python3 packaging/make_icons.py
"""
import math
import platform
import shutil
import subprocess
import tempfile
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

HERE = Path(__file__).resolve().parent
S = 1024


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def background():
    top, bot = (0x1F, 0x24, 0x28), (0x08, 0x09, 0x0A)
    grad = Image.new("RGB", (S, S))
    px = grad.load()
    for y in range(S):
        c = lerp(top, bot, y / (S - 1))
        for x in range(S):
            px[x, y] = c
    mask = Image.new("L", (S, S), 0)
    inset = 40  # leaves the margin macOS expects around the squircle
    ImageDraw.Draw(mask).rounded_rectangle(
        (inset, inset, S - inset, S - inset), radius=190, fill=255
    )
    img = grad.convert("RGBA")
    img.putalpha(mask)
    return img, mask


def circle(d, c, r, fill):
    d.ellipse((c[0] - r, c[1] - r, c[0] + r, c[1] + r), fill=fill)


def build():
    img, mask = background()

    # faint chain of small linked circles, bottom-left to upper-right
    chain = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(chain)
    pts = [(170, 760), (270, 700), (350, 770), (450, 720), (540, 790), (640, 740)]
    col = (170, 200, 215, 90)
    for a, b in zip(pts, pts[1:]):
        d.line((a, b), fill=col, width=9)
    for p in pts:
        circle(d, p, 24, col)
    img = Image.alpha_composite(img, chain)

    # glowing molecules: (x, y, radius, colour)
    mols = [
        (400, 400, 150, (40, 220, 235)),    # cyan
        (680, 330, 105, (130, 235, 120)),   # light green
        (690, 640, 130, (230, 70, 200)),    # magenta
        (330, 650, 80, (250, 200, 50)),     # gold
    ]
    glow = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(glow)
    for x, y, r, c in mols:
        circle(d, (x, y), int(r * 1.7), c + (150,))
    glow = glow.filter(ImageFilter.GaussianBlur(60))
    img = Image.alpha_composite(img, glow)

    solid = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(solid)
    for x, y, r, c in mols:
        circle(d, (x, y), r, c + (255,))
        hi = lerp(c, (255, 255, 255), 0.55)
        circle(d, (x - r // 3, y - r // 3), r // 4, hi + (255,))
    img = Image.alpha_composite(img, solid)

    # keep everything inside the rounded square
    img.putalpha(ImageChops.multiply(img.getchannel("A"), mask))
    return img


def main():
    master = build()
    master.save(HERE / "icon-1024.png")

    (HERE / "windows").mkdir(exist_ok=True)
    master.save(
        HERE / "windows" / "app.ico",
        sizes=[(n, n) for n in (16, 24, 32, 48, 64, 128, 256)],
    )

    if platform.system() != "Darwin":
        print("Skipping AppIcon.icns: iconutil is macOS-only.")
        return
    (HERE / "macos").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        iconset = Path(tmp) / "AppIcon.iconset"
        iconset.mkdir()
        for base in (16, 32, 128, 256, 512):
            for scale in (1, 2):
                px = base * scale
                name = f"icon_{base}x{base}{'@2x' if scale == 2 else ''}.png"
                master.resize((px, px), Image.LANCZOS).save(iconset / name)
        subprocess.run(
            ["iconutil", "-c", "icns", str(iconset), "-o",
             str(HERE / "macos" / "AppIcon.icns")],
            check=True,
        )
    print("Wrote icon-1024.png, windows/app.ico, macos/AppIcon.icns")


if __name__ == "__main__":
    main()
