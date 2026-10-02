#!/usr/bin/env python3
"""Read-only original PNG region transparency report, with no Pillow dependency.

Only supported non-interlaced 8-bit PNG types are reported. Unsupported encodings
fail explicitly; no pixels, masks, atlas positions, or scene drafts are modified.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import struct
import sys
import zlib

ROOT = Path(__file__).resolve().parents[1]
SIGNATURE = b"\x89PNG\r\n\x1a\n"


class UnsupportedPNG(ValueError):
    pass


def rgba_rows(payload: bytes):
    if not payload.startswith(SIGNATURE):
        raise UnsupportedPNG("Not a PNG")
    offset = 8
    ihdr = None
    palette = None
    transparency = None
    compressed = bytearray()
    while offset + 12 <= len(payload):
        length = struct.unpack_from(">I", payload, offset)[0]
        offset += 4
        tag = payload[offset:offset+4]
        offset += 4
        if length > len(payload) - offset - 4:
            raise UnsupportedPNG("Truncated PNG chunk")
        data = payload[offset:offset+length]
        crc = struct.unpack_from(">I", payload, offset+length)[0]
        if zlib.crc32(tag+data) & 0xFFFFFFFF != crc:
            raise UnsupportedPNG(f"CRC mismatch in {tag!r}")
        offset += length + 4
        if tag == b"IHDR":
            if ihdr is not None or len(data) != 13:
                raise UnsupportedPNG("Invalid IHDR")
            width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", data)
            if depth != 8 or color not in (0, 2, 3, 4, 6) or compression or filtering or interlace:
                raise UnsupportedPNG("Unsupported PNG: requires non-interlaced, 8-bit gray/RGB/indexed/gray-alpha/RGBA")
            if not width or not height or width*height > 16_000_000:
                raise UnsupportedPNG("Invalid/oversized PNG")
            ihdr = width, height, color
        elif tag == b"PLTE": palette = data
        elif tag == b"tRNS": transparency = data
        elif tag == b"IDAT": compressed.extend(data)
        elif tag == b"IEND": break
    if ihdr is None:
        raise UnsupportedPNG("Missing IHDR")
    width, height, color = ihdr
    channels = {0:1, 2:3, 3:1, 4:2, 6:4}[color]
    if color == 3 and (palette is None or len(palette)%3 or len(palette)>768):
        raise UnsupportedPNG("Indexed PNG missing valid PLTE")
    stride = width*channels
    try:
        raw = zlib.decompress(compressed, max_length=height*(stride+1)+1)
    except TypeError:
        raw = zlib.decompress(compressed)
    except zlib.error as error:
        raise UnsupportedPNG(f"Invalid IDAT zlib stream: {error}") from error
    if len(raw) != height*(stride+1):
        raise UnsupportedPNG("Unexpected decompressed scanline size")
    prev = bytearray(stride)
    out = []
    position = 0
    rgb_transparent = None
    gray_transparent = None
    if transparency is not None and color == 2 and len(transparency) == 6:
        rgb_transparent = tuple(transparency[index+1] for index in (0, 2, 4))
    if transparency is not None and color == 0 and len(transparency) == 2:
        gray_transparent = transparency[1]
    for _ in range(height):
        mode = raw[position]
        position += 1
        row = bytearray(raw[position:position+stride])
        position += stride
        if mode not in (0, 1, 2, 3, 4):
            raise UnsupportedPNG(f"Unsupported PNG scanline filter {mode}")
        for idx in range(stride):
            a = row[idx-channels] if idx >= channels else 0
            b = prev[idx]
            c = prev[idx-channels] if idx >= channels else 0
            if mode == 1: predictor = a
            elif mode == 2: predictor = b
            elif mode == 3: predictor = (a+b)//2
            elif mode == 4:
                pa = abs(b-c)
                pb = abs(a-c)
                pc = abs(a+b-2*c)
                predictor = a if pa <= pb and pa <= pc else b if pb <= pc else c
            else: predictor = 0
            row[idx] = (row[idx]+predictor)&0xff
        pixels = []
        for ix in range(width):
            chunk = row[ix*channels:(ix+1)*channels]
            if color == 6: rgba = tuple(chunk)
            elif color == 4: rgba = (chunk[0], chunk[0], chunk[0], chunk[1])
            elif color == 2:
                rgb = tuple(chunk)
                rgba = (*rgb, 0 if rgb == rgb_transparent else 255)
            elif color == 0:
                v = chunk[0]
                rgba = (v, v, v, 0 if v == gray_transparent else 255)
            else:
                index = chunk[0]
                if index*3+3 > len(palette):
                    raise UnsupportedPNG("Palette index exceeds PLTE")
                rgba = (*palette[index*3:index*3+3], transparency[index] if transparency is not None and index<len(transparency) else 255)
            pixels.append(rgba)
        out.append(pixels)
        prev = row
    return width, height, out


def region_report(rows, rect):
    x, y, w, h = rect
    if x<0 or y<0 or w<=0 or h<=0 or y+h>len(rows) or x+w>len(rows[0]):
        raise ValueError("Source region exceeds original PNG dimensions")
    selected = (pixel for row in rows[y:y+h] for pixel in row[x:x+w])
    result = {"pixels": w*h, "fully_transparent": 0, "partially_transparent": 0,
              "fully_opaque": 0, "opaque_exact_black": 0, "opaque_near_black": 0}
    for r,g,b,a in selected:
        if a == 0: result["fully_transparent"] += 1
        elif a == 255:
            result["fully_opaque"] += 1
            if (r,g,b) == (0,0,0): result["opaque_exact_black"] += 1
            if max(r,g,b) <= 32: result["opaque_near_black"] += 1
        else: result["partially_transparent"] += 1
    result["interpretation"] = (
        "All pixels are opaque: any dark pixels are from the original source, not uncovered lower scene."
        if result["fully_opaque"] == result["pixels"] else
        "Source has transparency: it should reveal the underlying authored layer if one is present."
    )
    return result


def inspect(asset: str, rect):
    manifest = json.loads((ROOT/"content/catalog/core_source_manifest.json").read_text(encoding="utf-8"))
    originals = {entry["path"]:entry for entry in manifest["entries"] if entry["path"].lower().endswith(".png")}
    if asset not in originals:
        raise ValueError(f"Asset is not in the canonical original PNG catalog: {asset}")
    path = ROOT/"assets/elizawy"/asset
    data = path.read_bytes()
    meta = originals[asset]
    if hashlib.sha256(data).hexdigest() != meta["sha256"] or len(data) != meta["bytes"]:
        raise ValueError(f"Original source hash/size mismatch; refusing alpha analysis of {asset}")
    width,height,rows = rgba_rows(data)
    if [width,height] != meta["imageSizePx"]:
        raise ValueError(f"Original source PNG dimensions mismatch: {asset}")
    result = region_report(rows,rect)
    result.update({"asset":asset,"rect":rect,"original_sha256":meta["sha256"],"source_size":[width,height],"source_mutated":False})
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--asset",required=True,help="Exact catalog path, e.g. Terrain/plants_summer.png")
    parser.add_argument("--rect",type=int,nargs=4,required=True,metavar=("X","Y","WIDTH","HEIGHT"))
    args = parser.parse_args(argv)
    try: print(json.dumps(inspect(args.asset,args.rect),indent=2))
    except (ValueError, OSError, zlib.error) as error:
        print("SOURCE ALPHA REPORT: FAIL:",error,file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
