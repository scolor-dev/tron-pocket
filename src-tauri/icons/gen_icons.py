#!/usr/bin/env python3
"""Generate placeholder app icons (PNG/ICO/ICNS) without external deps.

Run once during scaffolding. Replace the generated files with real artwork
before shipping - this only produces a simple red-coin placeholder.
"""
import struct
import zlib
from pathlib import Path

BG = (239, 0, 39, 255)  # TRON red
FG = (255, 255, 255, 255)

OUT = Path(__file__).parent


def make_png(size: int) -> bytes:
    cx = cy = size / 2
    r = size * 0.36
    raw = bytearray()
    for y in range(size):
        raw.append(0)  # filter type 0 (None)
        for x in range(size):
            dx, dy = x + 0.5 - cx, y + 0.5 - cy
            color = FG if (dx * dx + dy * dy) <= r * r else BG
            raw.extend(color)

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    idat = zlib.compress(bytes(raw), 9)
    return sig + chunk(b"IHDR", ihdr) + chunk(b"IDAT", idat) + chunk(b"IEND", b"")


def make_ico(sizes: list[int], path: Path) -> None:
    pngs = [make_png(s) for s in sizes]
    header = struct.pack("<HHH", 0, 1, len(sizes))
    entries = b""
    offset = len(header) + 16 * len(sizes)
    for s, png in zip(sizes, pngs):
        dim = 0 if s >= 256 else s
        entries += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(png), offset)
        offset += len(png)
    path.write_bytes(header + entries + b"".join(pngs))


def make_icns(sizes_to_tags: dict[int, bytes], path: Path) -> None:
    body = b""
    for size, tag in sizes_to_tags.items():
        png = make_png(size)
        body += tag + struct.pack(">I", len(png) + 8) + png
    header = b"icns" + struct.pack(">I", len(body) + 8)
    path.write_bytes(header + body)


for size, name in [(32, "32x32.png"), (128, "128x128.png"), (256, "128x128@2x.png")]:
    (OUT / name).write_bytes(make_png(size))

# Also keep a top-level icon.png (used by some bundlers/tooling).
(OUT / "icon.png").write_bytes(make_png(512))

make_ico([16, 32, 48, 256], OUT / "icon.ico")
make_icns(
    {
        16: b"icp4",
        32: b"icp5",
        128: b"ic07",
        256: b"ic08",
        512: b"ic09",
    },
    OUT / "icon.icns",
)

print("done")
