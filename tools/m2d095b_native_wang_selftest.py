#!/usr/bin/env python3
"""Static + deterministic contract checks for M2D095B.

This is project-owned evidence only; native Rust/Cargo tests remain the compile authority.
"""
from __future__ import annotations

import argparse
import random
import re
import struct
from pathlib import Path
from source_version_contract import cargo_source_version, project_matches_cargo, source_at_least

FNV_OFFSET = 0xCBF29CE484222325
FNV_PRIME = 0x100000001B3
MASK64 = (1 << 64) - 1


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"M2D095B SELFTEST FAIL: {message}")


def stable_hash(seed: int, x: int, y: int, wang: tuple[int, ...]) -> int:
    value = FNV_OFFSET ^ seed
    payload = struct.pack("<i", x) + struct.pack("<i", y) + bytes(wang)
    for byte in payload:
        value ^= byte
        value = (value * FNV_PRIME) & MASK64
    return value


def corner_wang(corners: tuple[int, int, int, int]) -> tuple[int, ...]:
    nw, ne, sw, se = corners
    return (0, ne, 0, se, 0, sw, 0, nw)


def checkerboard(corners: tuple[int, int, int, int]) -> bool:
    nw, ne, sw, se = corners
    return nw != 0 and ne != 0 and nw == se and ne == sw and nw != ne


def binary_exact_set() -> set[tuple[int, ...]]:
    result: set[tuple[int, ...]] = set()
    for mask in range(16):
        corners = tuple(2 if mask & (1 << i) else 1 for i in range(4))
        if not checkerboard(corners):
            result.add(corner_wang(corners))
    return result


def mirror_solve(width: int, height: int, semantic: list[int], seed: int) -> bool:
    exact = binary_exact_set()
    visual = semantic.copy()

    def vi(x: int, y: int) -> int:
        return y * width + x

    def corners(x: int, y: int) -> tuple[int, int, int, int]:
        return (
            visual[vi(x, y)],
            visual[vi(x + 1, y)],
            visual[vi(x, y + 1)],
            visual[vi(x + 1, y + 1)],
        )

    def unresolved() -> int:
        return sum(
            corner_wang(corners(x, y)) not in exact
            for y in range(height - 1)
            for x in range(width - 1)
        )

    cells = (width - 1) * (height - 1)
    max_passes = max(cells * 4, 8)
    passes = 0
    while unresolved():
        if passes >= max_passes:
            return False
        changed = False
        for y in range(height - 1):
            for x in range(width - 1):
                c = corners(x, y)
                w = corner_wang(c)
                if w in exact:
                    continue
                if not checkerboard(c):
                    return False
                selector = stable_hash(seed, x, y, w) & 3
                vx, vy = ((x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1))[selector]
                index = vi(vx, vy)
                before = visual[index]
                visual[index] = c[1] if before == c[0] else c[0]
                changed |= visual[index] != before
        passes += 1
        if not changed:
            return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--stress", type=int, default=2000)
    args = parser.parse_args()
    root = args.root.resolve()

    cargo = (root / "Cargo.toml").read_text(encoding="utf-8")
    tiled = (root / "src/tiled_terrain.rs").read_text(encoding="utf-8")
    native_path = root / "src/tiled_terrain/native_wang.rs"
    require(native_path.is_file(), "missing src/tiled_terrain/native_wang.rs")
    native = native_path.read_text(encoding="utf-8")

    require(source_at_least(cargo, "0.10.13"), "integrated source predates 0.10.13")
    require(
        'tiled = { version = "=0.16.0", default-features = false }' in cargo,
        "tiled 0.16 parser dependency missing",
    )
    require("bevy_map_autotile" not in cargo, "external Bevy 0.18 Wang dependency still present")
    require("mod native_wang;" in tiled, "Tiled authority does not own native Wang module")
    require("solve_corner_material_grid" in tiled, "Tiled authority lacks native grid resolver")
    require("bevy_map_autotile" not in tiled + native, "source still imports external Wang crate")
    require(not re.search(r"then_some\([^\n]*-\s*1", tiled + native), "eager unsigned subtraction pattern returned")
    require("semantic_vertex_ids.to_vec()" in native, "solver must derive a visual copy from semantic vertices")
    require("Unconfigured non-checkerboard Corner topology" in native, "unknown sparse topology must fail closed")

    rng = random.Random(0x48574E47)
    for index in range(max(0, args.stress)):
        width = rng.randint(2, 18)
        height = rng.randint(2, 18)
        semantic = [rng.randint(1, 2) for _ in range(width * height)]
        seed = rng.getrandbits(64)
        require(mirror_solve(width, height, semantic, seed), f"mirror solver failed stress case {index}")

    print(f"M2D095B NATIVE WANG SELFTEST: PASS / {args.stress} deterministic stress grids")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
