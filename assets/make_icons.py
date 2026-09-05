#!/usr/bin/env python3
"""Regenerate assets/icon.ico and assets/icon_32.png from assets/src/.

Needs Pillow (`apt install python3-pil` or `pip install pillow`).
The 16px ico layer comes from src16.png (the hand-tuned small variant);
every other size is Lanczos-downscaled from src64.png. The .ico uses
PNG-compressed entries (Vista+), one layer per size: 16/24/32/48/64.
"""

import io
import struct
from pathlib import Path

from PIL import Image

HERE = Path(__file__).parent


def png_bytes(image: Image.Image) -> bytes:
    buf = io.BytesIO()
    image.save(buf, format="PNG", optimize=True)
    return buf.getvalue()


def main() -> None:
    src64 = Image.open(HERE / "src" / "src64.png").convert("RGBA")
    src16 = Image.open(HERE / "src" / "src16.png").convert("RGBA")

    layers = [(16, png_bytes(src16))]
    for size in (24, 32, 48, 64):
        layers.append((size, png_bytes(src64.resize((size, size), Image.LANCZOS))))

    out = struct.pack("<HHH", 0, 1, len(layers))
    offset = 6 + 16 * len(layers)
    blobs = []
    for size, data in layers:
        out += struct.pack("<BBBBHHII", size, size, 0, 0, 1, 32, len(data), offset)
        blobs.append(data)
        offset += len(data)
    for blob in blobs:
        out += blob
    (HERE / "icon.ico").write_bytes(out)

    src64.resize((32, 32), Image.LANCZOS).save(HERE / "icon_32.png", optimize=True)
    print(f"wrote icon.ico ({len(out)} bytes, {len(layers)} layers) and icon_32.png")


if __name__ == "__main__":
    main()
