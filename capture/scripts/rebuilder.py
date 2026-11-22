"""Replaces the builder chunk (crJs) of carrier PNGs with the current figma/builder.js.

usage: python3 rebuilder.py <carrier.figma.png>...
"""
import pathlib
import struct
import sys
import zlib

BUILDER = (pathlib.Path(__file__).resolve().parent.parent / "figma" / "builder.js").read_bytes()


def chunk(kind: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


for path in map(pathlib.Path, sys.argv[1:]):
    data = path.read_bytes()
    out, position = bytearray(data[:8]), 8
    while position < len(data):
        (length,) = struct.unpack(">I", data[position:position + 4])
        kind = data[position + 4:position + 8]
        end = position + 12 + length
        out += chunk(kind, BUILDER) if kind == b"crJs" else data[position:end]
        position = end
    path.write_bytes(bytes(out))
    print(path.name)
