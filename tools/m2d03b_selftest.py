#!/usr/bin/env python3
"""Cargo-independent PNG-alpha regression fixture; not a native Studio/PIE certificate."""
from __future__ import annotations
import importlib.util
import struct
from pathlib import Path
import zlib

source = Path(__file__).with_name('inspect_elizawy_source_alpha.py')
spec = importlib.util.spec_from_file_location('alpha_tested',source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

def chunk(tag,payload):
    return struct.pack('>I',len(payload))+tag+payload+struct.pack('>I',zlib.crc32(tag+payload)&0xffffffff)

# Four original RGBA samples: opaque nonblack, fully clear, opaque black, partially transparent.
scan = b'\x00' + bytes([11,22,33,255, 0,0,0,0]) + b'\x00' + bytes([0,0,0,255, 5,6,7,128])
png = (module.SIGNATURE+chunk(b'IHDR',struct.pack('>IIBBBBB',2,2,8,6,0,0,0))
    +chunk(b'IDAT',zlib.compress(scan))+chunk(b'IEND',b''))
w,h,rows = module.rgba_rows(png)
assert (w,h)==(2,2)
report = module.region_report(rows,[0,0,2,2])
assert report['pixels']==4 and report['fully_opaque']==2 and report['fully_transparent']==1
assert report['partially_transparent']==1 and report['opaque_exact_black']==1
assert module.region_report(rows,[0,1,1,1])['opaque_exact_black']==1
try:
    module.region_report(rows,[1,1,2,1])
except ValueError:
    pass
else:
    raise AssertionError('out-of-bounds crop must fail')
print('M2D03-B ALPHA SELFTEST: PASS / 8-bit RGBA exact pixels, alpha categories, black-is-not-transparent, bounded regions')
