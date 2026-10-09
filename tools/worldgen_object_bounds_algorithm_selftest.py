#!/usr/bin/env python3
"""Regression tests for guide-safe tree/rock render-bound certification.

This test is source-byte independent: Full Gate separately hash-verifies the real
ElizaWy PNGs. Here we lock the geometry rule that a long 1-2 px disconnected
slot-edge guide is provenance, while short/irregular edge artwork remains art.
"""
from __future__ import annotations

from certify_worldgen_object_bounds import alpha_bbox


def rgba_grid(width: int, height: int):
    return [[(0, 0, 0, 0) for _ in range(width)] for _ in range(height)]


def paint(rows, x0, y0, x1, y1, rgba=(1, 2, 3, 255)):
    for y in range(y0, y1):
        for x in range(x0, x1):
            rows[y][x] = rgba


def main() -> int:
    # Reproduce the failure class visible on trees_summer.png: a long, thin,
    # disconnected slot-edge stroke plus legitimate central object alpha.
    rows = rgba_grid(32, 32)
    paint(rows, 0, 2, 1, 29)       # guide-like vertical stroke
    paint(rows, 7, 6, 25, 27)      # object body
    bbox, retained, edge, discarded, components = alpha_bbox(rows, [0, 0, 32, 32])
    assert bbox == [7, 6, 25, 27], bbox
    assert retained == 18 * 21, retained
    assert edge == 0, edge
    assert discarded == 27, discarded
    assert any(component["guideLike"] for component in components)

    # A small piece of legitimate artwork can touch a slot edge; it must not be
    # stripped merely because it reaches the border.
    rows = rgba_grid(32, 32)
    paint(rows, 0, 12, 6, 18)
    paint(rows, 5, 10, 22, 24)
    bbox, retained, _edge, discarded, components = alpha_bbox(rows, [0, 0, 32, 32])
    assert bbox == [0, 10, 22, 24], bbox
    assert retained > 0
    assert discarded == 0, discarded
    assert not any(component["guideLike"] for component in components)

    # A long two-pixel horizontal provenance stroke is also rejected.
    rows = rgba_grid(40, 24)
    paint(rows, 2, 22, 37, 24)
    paint(rows, 9, 3, 31, 20)
    bbox, _retained, _edge, discarded, _components = alpha_bbox(rows, [0, 0, 40, 24])
    assert bbox == [9, 3, 31, 20], bbox
    assert discarded == 35 * 2, discarded

    print("WORLDGEN OBJECT BOUNDS ALGORITHM SELFTEST: PASS / edge guides stripped; legitimate edge art retained")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
