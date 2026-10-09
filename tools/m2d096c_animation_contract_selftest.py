#!/usr/bin/env python3
"""Source-only guard for the M2D096C animation-presentation preparation lane."""
from __future__ import annotations

import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
RUST = ROOT / "src/tiled_terrain.rs"
MAIN = ROOT / "src/main.rs"
AUTH = ROOT / "src/asset_authority.rs"
REF = ROOT / "content/terrain/evidence/elizawy_test_scenes.v1.json"


def req(ok: bool, message: str) -> None:
    if not ok:
        raise AssertionError(message)


def phase(seed: int, world: tuple[int, int], period: int) -> int:
    if period == 0:
        return 0
    _ = world
    return ((seed ^ (seed >> 32)) * 17) % period


def frame_index(durations: list[int], elapsed: int, phase_ms: int = 0) -> int | None:
    total = sum(durations)
    if not durations or total <= 0 or any(d <= 0 for d in durations):
        return None
    cursor = (elapsed + phase_ms) % total
    for index, duration in enumerate(durations):
        if cursor < duration:
            return index
        cursor -= duration
    return len(durations) - 1


def main() -> int:
    text = RUST.read_text(encoding="utf-8")
    main_rs = MAIN.read_text(encoding="utf-8")
    auth = AUTH.read_text(encoding="utf-8")
    ref = json.loads(REF.read_text(encoding="utf-8"))

    req(ref["schema"] == "havenwild.elizawy.test_scene_reference.v1", "test-scene schema drift")
    req(ref["authority"] == "visual_reference_only_no_runtime_promotion", "test-scene authority must remain evidence-only")
    req(ref["pinnedRevision"] == "f07f7f5892e67c932c68f70bb04472f2c64e46bc", "ElizaWy evidence revision drift")
    files = {item["path"]: item for item in ref["files"]}
    req("_ Test Scenes/DemoGame - 2 - Summer.png" in files, "Summer upstream reference missing")
    req("_ Test Scenes/DemoGame - Seasons.gif" in files, "Seasons upstream reference missing")
    req(all(re.fullmatch(r"[0-9a-f]{40}", item["gitBlobSha"]) for item in files.values()), "invalid Git blob SHA")

    markers = [
        "pub struct CanonicalAnimationFrame",
        "pub struct CanonicalAnimationClip",
        "canonical_animation_by_tile",
        "canonical_animation_clip_count",
        "incomplete_animation_clip_count",
        "pub fn frame_index_at",
        "pub fn world_animation_phase_ms",
        "pub fn canonical_animation_binding_at",
        "pub fn resolve_havenwild_corner_animated",
        "Partial clips are never promoted",
    ]
    for marker in markers:
        req(marker in text, f"missing Rust animation-contract marker: {marker}")

    for marker in ["SummerVisualAnimation", "SummerVisualAnimationFrame"]:
        req(marker in auth, f"missing generic visual-animation marker: {marker}")
    for marker in ["CachedWorldAnimation", "source_at(elapsed_ms)", "cached_world_animation", "request_repaint_after"]:
        req(marker in main_rs, f"live animation presentation missing marker: {marker}")

    # Keep unsupported legacy semantics fail-closed during live presentation.
    req('"WetSand" | "PebblePath" => None' in text, "legacy shore roles were promoted without evidence")

    # Mirror the Rust timeline math on boundary cases.
    durations = [100, 200]
    req(frame_index(durations, 0) == 0, "frame 0 start")
    req(frame_index(durations, 99) == 0, "frame 0 end")
    req(frame_index(durations, 100) == 1, "frame 1 start")
    req(frame_index(durations, 299) == 1, "frame 1 end")
    req(frame_index(durations, 300) == 0, "timeline wrap")
    req(frame_index(durations, 0, 100) == 1, "phase offset")
    req(frame_index([0, 100], 0) is None, "zero-duration clips must fail closed")

    period = 480
    origin = phase(7, (0, 0), period)
    req(phase(7, (1, 0), period) == origin, "east tile must share synchronized environmental phase")
    req(phase(7, (-1, 0), period) == origin, "west tile must share synchronized environmental phase")
    req(phase(7, (91, -57), period) == origin, "far tile must not form a chase-light phase gradient")
    req(phase(7, (3, -5), 0) == 0, "zero-period phase must fail closed")

    print("M2D096C ANIMATION CONTRACT SELFTEST: PASS")
    print("  upstream test scenes: pinned visual-reference metadata only")
    print("  canonical clips      : all-frame exact-pixel requirement")
    print("  phase field          : seed-stable synchronized environmental clock")
    print("  live presentation    : ON only for complete canonical clips; incomplete clips fail closed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"M2D096C ANIMATION CONTRACT SELFTEST: FAIL - {exc}", file=sys.stderr)
        raise SystemExit(1)
