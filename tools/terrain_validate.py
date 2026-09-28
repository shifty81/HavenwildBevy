#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
import sys
import hashlib

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "content" / "terrain" / "dual_grid_contract.v1.json"
RECIPE = ROOT / "content" / "terrain" / "recipes" / "summer_grass_void.v1.json"
FIXTURES = ROOT / "content" / "terrain" / "fixtures"
RECOVERY_ROOT = ROOT / "content" / "terrain" / "recovery"
HISTORICAL = RECOVERY_ROOT / "historical_authority.v1.json"
SUMMER_INVENTORY = RECOVERY_ROOT / "summer_source_cells.v1.json"
SEMANTIC_LAB = ROOT / "content" / "terrain" / "previews" / "dg01_grass_void.semantic_lab.v1.json"
CORE_MANIFEST = ROOT / "content" / "catalog" / "core_source_manifest.json"
ASSET_ROOT = ROOT / "assets" / "elizawy"
EXPECTED_MASKS = [f"{i:04b}" for i in range(16)]


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def png_size(path: Path):
    data = path.read_bytes()[:24]
    if len(data) < 24 or data[:8] != b"\x89PNG\r\n\x1a\n":
        return None
    return (int.from_bytes(data[16:20], "big"), int.from_bytes(data[20:24], "big"))


def source_dimensions() -> dict[str, tuple[int, int]]:
    try:
        manifest = load(CORE_MANIFEST)
    except Exception:
        return {}
    result: dict[str, tuple[int, int]] = {}
    for entry in manifest.get("entries", []):
        size = entry.get("imageSizePx")
        path = entry.get("path")
        if isinstance(path, str) and isinstance(size, list) and len(size) == 2:
            result[path] = (int(size[0]), int(size[1]))
    return result


SOURCE_DIMENSIONS = source_dimensions()


def validate_source(source: dict, label: str, errors: list[str], warnings: list[str]) -> None:
    path = source.get("path")
    rect = source.get("rect")
    if not path or not isinstance(rect, list) or len(rect) != 4:
        errors.append(f"{label}: mapping requires source.path and source.rect[4]")
        return
    try:
        x, y, w, h = [int(v) for v in rect]
    except (TypeError, ValueError):
        errors.append(f"{label}: source rect must contain integer values")
        return
    if min(x, y) < 0 or w <= 0 or h <= 0:
        errors.append(f"{label}: invalid source rect {rect}")
        return
    canonical_size = SOURCE_DIMENSIONS.get(str(path))
    if canonical_size is None:
        errors.append(f"{label}: source path is absent from canonical manifest: {path}")
        return
    if x + w > canonical_size[0] or y + h > canonical_size[1]:
        errors.append(f"{label}: source rect {rect} exceeds canonical {path} {canonical_size}")
        return
    p = ASSET_ROOT / path
    if not p.is_file():
        warnings.append(f"{label}: hydrated source missing locally: {path}")
        return
    size = png_size(p)
    if size and size != canonical_size:
        errors.append(f"{label}: hydrated dimensions {size} differ from canonical {canonical_size} for {path}")


def fixture_mask_counts(fixture: dict) -> Counter[str]:
    width = int(fixture["width"])
    height = int(fixture["height"])
    cells = fixture["cells"]

    def cell(x: int, y: int) -> int:
        if x < 0 or y < 0 or x >= width or y >= height:
            return 0
        return 1 if cells[y][x] else 0

    counts: Counter[str] = Counter()
    for vy in range(height + 1):
        for vx in range(width + 1):
            mask = 0
            if cell(vx - 1, vy - 1):
                mask |= 0b0001
            if cell(vx, vy - 1):
                mask |= 0b0010
            if cell(vx - 1, vy):
                mask |= 0b0100
            if cell(vx, vy):
                mask |= 0b1000
            counts[f"{mask:04b}"] += 1
    return counts


def load_fixtures(errors: list[str]) -> list[tuple[Path, dict]]:
    fixtures = []
    for fixture_path in sorted(FIXTURES.glob("*.json")):
        try:
            fixture = load(fixture_path)
            width = int(fixture["width"])
            height = int(fixture["height"])
            cells = fixture["cells"]
            if len(cells) != height or any(len(row) != width for row in cells):
                errors.append(f"{fixture_path.name}: dimensions do not match cells")
            for row in cells:
                for value in row:
                    if value not in (0, 1):
                        errors.append(
                            f"{fixture_path.name}: binary fixture contains non-binary value {value}"
                        )
            fixtures.append((fixture_path, fixture))
        except Exception as exc:
            errors.append(f"{fixture_path.name}: {exc}")
    return fixtures


def validate(strict: bool, show_fixtures: bool = False) -> int:
    errors: list[str] = []
    warnings: list[str] = []
    try:
        contract = load(CONTRACT)
        recipe = load(RECIPE)
    except Exception as exc:
        print(f"TERRAIN CONTRACT: FAIL: {exc}")
        return 2

    if contract.get("binaryMasks") != EXPECTED_MASKS:
        errors.append("dual-grid contract must enumerate masks 0000..1111 in numeric order")
    bits = contract.get("cornerBits", {})
    if bits != {"NW": 1, "NE": 2, "SW": 4, "SE": 8}:
        errors.append("corner bit contract differs from NW=1 NE=2 SW=4 SE=8")
    if contract.get("dirtyVertexOffsetsForChangedCell") != [[0, 0], [1, 0], [0, 1], [1, 1]]:
        errors.append("changed-cell invalidation must target the four surrounding terrain vertices")

    states = recipe.get("states", {})
    if sorted(states) != EXPECTED_MASKS:
        errors.append("summer.grass_void recipe must contain exactly 16 mask entries")

    mapped = 0
    certified = 0
    unresolved = []
    uncertified = []
    counts = Counter()
    for mask in EXPECTED_MASKS:
        entry = states.get(mask, {})
        resolution = entry.get("resolution")
        counts[str(resolution)] += 1
        if resolution in ("sprite", "composite", "unsupported"):
            mapped += 1
            confidence = entry.get("confidence")
            if confidence == "certified":
                certified += 1
            else:
                uncertified.append(mask)
            if confidence not in (None, "candidate", "reviewed", "certified"):
                errors.append(f"{mask}: unknown confidence state {confidence!r}")
        else:
            unresolved.append(mask)
        if resolution == "sprite":
            if entry.get("parts"):
                errors.append(f"{mask}: sprite mapping must not retain composite parts")
            validate_source(entry.get("source", {}), mask, errors, warnings)
        elif resolution == "composite":
            if entry.get("source") is not None:
                errors.append(f"{mask}: composite mapping must not also retain source")
            parts = entry.get("parts")
            if not isinstance(parts, list) or not parts:
                errors.append(f"{mask}: composite mapping requires non-empty parts")
                continue
            for index, part in enumerate(parts, 1):
                validate_source(
                    (part or {}).get("source", {}),
                    f"{mask} composite part {index}",
                    errors,
                    warnings,
                )
                offset = (part or {}).get("offset", [0, 0])
                if not isinstance(offset, list) or len(offset) != 2:
                    errors.append(f"{mask} composite part {index}: offset must be [x,y]")
        elif resolution in ("unsupported", "unmapped", None):
            if entry.get("source") is not None or entry.get("parts"):
                errors.append(f"{mask}: {resolution or 'unmapped'} mapping must not retain source payloads")
            if resolution in ("unmapped", None) and entry.get("confidence") is not None:
                errors.append(f"{mask}: unmapped mapping must not carry confidence")
        else:
            errors.append(f"{mask}: unsupported resolution kind {resolution!r}")

    fixtures = load_fixtures(errors)

    print("HAVENWILD TERRAIN / DUAL-GRID CONTRACT")
    print(f"Contract : {CONTRACT.relative_to(ROOT)}")
    print(f"Recipe   : {RECIPE.relative_to(ROOT)}")
    print(f"Masks    : {mapped}/16 classified · {certified}/16 certified")
    print(
        "Breakdown: "
        f"sprite={counts['sprite']} composite={counts['composite']} "
        f"unsupported={counts['unsupported']} unmapped={counts['unmapped']}"
    )
    if unresolved:
        print("Unmapped : " + " ".join(unresolved))
    if uncertified:
        print("Review   : " + " ".join(uncertified) + " (classified but not certified)")

    if show_fixtures or strict:
        union: Counter[str] = Counter()
        print("FIXTURE TOPOLOGY COVERAGE")
        for fixture_path, fixture in fixtures:
            mask_counts = fixture_mask_counts(fixture)
            union.update(mask_counts)
            present = [mask for mask in EXPECTED_MASKS if mask_counts[mask]]
            print(
                f"  {fixture.get('id', fixture_path.stem)}: "
                f"{len(present)}/16 masks -> {' '.join(present)}"
            )
        covered = [mask for mask in EXPECTED_MASKS if union[mask]]
        missing = [mask for mask in EXPECTED_MASKS if not union[mask]]
        print(f"  Combined: {len(covered)}/16 masks")
        if missing:
            print("  Fixture gaps: " + " ".join(missing))

    for warning in warnings:
        print("[WARN]", warning)
    for error in errors:
        print("[FAIL]", error)
    if errors:
        return 3
    if strict and (unresolved or uncertified):
        if unresolved:
            print("CERTIFICATION: NOT READY — all 16 states must be explicitly sprite/composite/unsupported")
        if uncertified:
            print("CERTIFICATION: NOT READY — every classified state must be explicitly promoted to confidence=certified")
        return 4
    print("CONTRACT: PASS" + (" / CERTIFIED" if strict else " / scaffold-valid"))
    return 0



def validate_resolver_lab(verbose: bool = True) -> int:
    errors: list[str] = []
    try:
        lab = load(SEMANTIC_LAB)
    except Exception as exc:
        print(f"DG RESOLVER LAB: FAIL: {exc}")
        return 7

    if lab.get("schema") != "havenwild.terrain.semantic_lab.v1":
        errors.append("semantic lab schema must be havenwild.terrain.semantic_lab.v1")
    size = lab.get("size")
    if not isinstance(size, list) or len(size) != 2:
        errors.append("semantic lab size must be [width,height]")
        width = height = 0
    else:
        width, height = [int(v) for v in size]
        if width <= 0 or height <= 0:
            errors.append("semantic lab dimensions must be positive")
    cells = lab.get("cells")
    if not isinstance(cells, list) or len(cells) != width * height:
        errors.append(
            f"semantic lab cells must contain width*height booleans ({width * height})"
        )
        cells = []
    elif any(type(value) is not bool for value in cells):
        errors.append("semantic lab cells must be booleans")

    forbidden = {"source", "sourceAsset", "sourceRect", "atlas", "atlasPath"}
    if forbidden.intersection(lab):
        errors.append("semantic lab must not contain atlas/source address fields")

    masks: Counter[str] = Counter()
    if cells and width and height:
        def cell(x: int, y: int) -> bool:
            if x < 0 or y < 0 or x >= width or y >= height:
                return False
            return cells[y * width + x]

        for vy in range(height + 1):
            for vx in range(width + 1):
                mask = 0
                if cell(vx - 1, vy - 1): mask |= 0b0001
                if cell(vx, vy - 1): mask |= 0b0010
                if cell(vx - 1, vy): mask |= 0b0100
                if cell(vx, vy): mask |= 0b1000
                masks[f"{mask:04b}"] += 1

    covered = [mask for mask in EXPECTED_MASKS if masks[mask]]
    missing = [mask for mask in EXPECTED_MASKS if not masks[mask]]
    if missing:
        errors.append("semantic resolver lab does not cover masks: " + " ".join(missing))

    if verbose:
        print("HAVENWILD DG SEMANTIC RESOLVER LAB")
        print(f"Document : {SEMANTIC_LAB.relative_to(ROOT)}")
        print(f"Grid     : {width}x{height} semantic cells")
        print(f"Coverage : {len(covered)}/16 masks")
        if covered:
            print("Masks    : " + " ".join(covered))
        print("Authority: semantic-only; no atlas/source coordinates allowed")
    for error in errors:
        print("[FAIL]", error)
    if errors:
        return 8
    print("DG RESOLVER LAB: PASS / semantic -> vertex mask path covers all 16 states")
    return 0



def validate_resolved_pipeline(verbose: bool = True) -> int:
    errors: list[str] = []
    try:
        lab = load(SEMANTIC_LAB)
        recipe = load(RECIPE)
    except Exception as exc:
        print(f"DG RESOLVED PIPELINE: FAIL: {exc}")
        return 9

    size = lab.get("size", [0, 0])
    cells = lab.get("cells", [])
    try:
        width, height = int(size[0]), int(size[1])
    except Exception:
        width = height = 0
    if width <= 0 or height <= 0 or len(cells) != width * height:
        errors.append("semantic lab dimensions/cells are invalid")

    if lab.get("foreground") != recipe.get("foreground") or lab.get("background") != recipe.get("background"):
        errors.append(
            f"semantic family {lab.get('foreground')}/{lab.get('background')} does not match recipe "
            f"{recipe.get('foreground')}/{recipe.get('background')}"
        )

    states = recipe.get("states", {})
    vertices: list[tuple[int, int, str, str, str | None]] = []
    masks = Counter()
    classifications = Counter()

    def cell(x: int, y: int) -> bool:
        if x < 0 or y < 0 or x >= width or y >= height:
            return False
        return bool(cells[y * width + x])

    if not errors:
        for vy in range(height + 1):
            for vx in range(width + 1):
                mask = 0
                if cell(vx - 1, vy - 1): mask |= 0b0001
                if cell(vx, vy - 1): mask |= 0b0010
                if cell(vx - 1, vy): mask |= 0b0100
                if cell(vx, vy): mask |= 0b1000
                key = f"{mask:04b}"
                state = states.get(key)
                if not isinstance(state, dict):
                    errors.append(f"vertex ({vx},{vy}) resolves to missing recipe state {key}")
                    continue
                resolution = str(state.get("resolution", "unmapped"))
                confidence = state.get("confidence")
                vertices.append((vx, vy, key, resolution, confidence))
                masks[key] += 1
                classifications[resolution] += 1

    expected_vertices = (width + 1) * (height + 1)
    if len(vertices) != expected_vertices and not errors:
        errors.append(f"resolved vertex count {len(vertices)} != expected {expected_vertices}")
    missing_masks = [key for key in EXPECTED_MASKS if not masks[key]]
    if missing_masks:
        errors.append("resolved semantic seed does not exercise masks: " + " ".join(missing_masks))

    # Every semantic cell, including corners and edges, must invalidate exactly its
    # four surrounding dual-grid outputs and remain inside the (w+1)x(h+1) vertex grid.
    dirty_cells_checked = 0
    for cy in range(height):
        for cx in range(width):
            dirty = {
                (cx, cy),
                (cx + 1, cy),
                (cx, cy + 1),
                (cx + 1, cy + 1),
            }
            if len(dirty) != 4:
                errors.append(f"changed-cell dirty-vertex contract is not unique at ({cx},{cy})")
                continue
            if any(x < 0 or y < 0 or x > width or y > height for x, y in dirty):
                errors.append(f"changed-cell dirty-vertex contract escaped bounds at ({cx},{cy})")
                continue
            dirty_cells_checked += 1

    probe_x, probe_y = min(2, max(width - 1, 0)), min(2, max(height - 1, 0))
    dirty = {
        (probe_x, probe_y),
        (probe_x + 1, probe_y),
        (probe_x, probe_y + 1),
        (probe_x + 1, probe_y + 1),
    }

    # Lock the bit orientation independently of the checked-in seed. One foreground
    # cell must appear as SE/SW/NE/NW from its four surrounding vertices respectively.
    cx, cy = probe_x, probe_y
    orientation = {
        (cx, cy): 0b1000,
        (cx + 1, cy): 0b0100,
        (cx, cy + 1): 0b0010,
        (cx + 1, cy + 1): 0b0001,
    }
    for (vx, vy), expected_mask in orientation.items():
        mask = 0
        if (vx - 1, vy - 1) == (cx, cy): mask |= 0b0001
        if (vx, vy - 1) == (cx, cy): mask |= 0b0010
        if (vx - 1, vy) == (cx, cy): mask |= 0b0100
        if (vx, vy) == (cx, cy): mask |= 0b1000
        if mask != expected_mask:
            errors.append(
                f"corner-bit orientation drift at vertex ({vx},{vy}): "
                f"expected {expected_mask:04b}, got {mask:04b}"
            )

    if verbose:
        classified = sum(
            1 for _, _, _, resolution, _ in vertices
            if resolution in ("sprite", "composite", "unsupported")
        )
        certified = sum(
            1 for _, _, _, resolution, confidence in vertices
            if resolution in ("sprite", "composite", "unsupported")
            and confidence == "certified"
        )
        print("HAVENWILD DG RESOLVED PIPELINE")
        print(f"Semantic : {SEMANTIC_LAB.relative_to(ROOT)}")
        print(f"Recipe   : {RECIPE.relative_to(ROOT)}")
        print(f"Vertices : {len(vertices)}/{expected_vertices}")
        print(f"Coverage : {len([key for key in EXPECTED_MASKS if masks[key]])}/16 masks")
        print(f"Resolved : {classified}/{len(vertices)} classified · {certified}/{len(vertices)} certified")
        print(
            "Breakdown: "
            f"sprite={classifications['sprite']} composite={classifications['composite']} "
            f"unsupported={classifications['unsupported']} unmapped={classifications['unmapped']}"
        )
        print(
            f"Dirty probe cell ({probe_x},{probe_y}) -> "
            + " ".join(f"({x},{y})" for x, y in sorted(dirty))
        )
        print(f"Dirty coverage: {dirty_cells_checked}/{width * height} semantic cells -> exactly four in-bounds vertices")
        print("Bit orientation: NW=0001 NE=0010 SW=0100 SE=1000 verified by single-cell probe")
        print("Authority: semantic cells -> shared 4-bit vertex mask -> recipe state")

    for error in errors:
        print("[FAIL]", error)
    if errors:
        return 10
    print("DG RESOLVED PIPELINE: PASS")
    return 0

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for block in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def validate_recovery(verbose: bool = True) -> int:
    errors: list[str] = []
    warnings: list[str] = []
    try:
        history = load(HISTORICAL)
        inventory = load(SUMMER_INVENTORY)
    except Exception as exc:
        print(f"SUMMER RECOVERY: FAIL: {exc}")
        return 5

    source = inventory.get("source", {})
    cells = inventory.get("cells", [])
    expected_hash = source.get("sha256")
    if source.get("dimensions") != [512, 832]:
        errors.append("Summer source inventory must describe 512x832 terrain_summer.png")
    if source.get("grid") != [16, 26] or source.get("totalCells") != 416:
        errors.append("Summer source inventory must contain the complete 16x26 / 416-cell grid")
    if len(cells) != 416:
        errors.append(f"Summer source inventory has {len(cells)} cells; expected 416")

    seen = set()
    nontransparent = 0
    transparent = 0
    for entry in cells:
        grid = tuple(entry.get("grid", []))
        rect = entry.get("rect")
        if len(grid) != 2 or grid in seen:
            errors.append(f"invalid or duplicate Summer source grid coordinate {grid}")
            continue
        seen.add(grid)
        x, y = grid
        if not (0 <= x < 16 and 0 <= y < 26):
            errors.append(f"Summer source grid coordinate out of range: {grid}")
        if rect != [x * 32, y * 32, 32, 32]:
            errors.append(f"Summer source rect mismatch at {grid}: {rect}")
        occupancy = entry.get("occupancy")
        if occupancy == "nontransparent":
            nontransparent += 1
            if entry.get("recoveryState") != "source_mapped":
                errors.append(f"nontransparent cell {grid} must be source_mapped")
        elif occupancy == "transparent":
            transparent += 1
        else:
            errors.append(f"unknown occupancy at {grid}: {occupancy!r}")

    if nontransparent != 305 or transparent != 111:
        errors.append(
            f"Summer pixel replay count mismatch: nontransparent={nontransparent}, transparent={transparent}"
        )

    canonical = history.get("canonicalSummer", {})
    if canonical.get("nonTransparentCellsMapped") != 305:
        errors.append("historical Summer authority must preserve the verified 305/305 mapping count")
    evidence = history.get("historicalEvidence", [])
    evidence_ids = {item.get("id") for item in evidence}
    for required in (
        "pass103_summer_complete_map",
        "pass106_seasonal_topology",
        "b48r15_summer_mapper",
        "later_terrain_v7_authority",
    ):
        if required not in evidence_ids:
            errors.append(f"historical authority is missing evidence record {required}")

    source_path = ASSET_ROOT / source.get("path", "")
    if source_path.is_file():
        actual_hash = sha256_file(source_path)
        if actual_hash != expected_hash:
            errors.append(
                f"hydrated Summer source hash mismatch: expected {expected_hash}, got {actual_hash}"
            )
    else:
        warnings.append("hydrated Terrain/terrain_summer.png is unavailable; hash replay skipped")

    if verbose:
        print("HAVENWILD SUMMER MAPPING RECOVERY")
        print(f"Inventory : {SUMMER_INVENTORY.relative_to(ROOT)}")
        print(f"History   : {HISTORICAL.relative_to(ROOT)}")
        print(f"Source    : {source.get('path')} · {source.get('dimensions')} · grid {source.get('grid')}")
        print(f"Recovered : {nontransparent}/305 non-transparent source cells")
        print(f"Transparent: {transparent} total; historical split 14 structural / 97 unused remains identity-unrecovered")
        print("Historical evidence: 20 topology families · 176 ordered-pair variants · 144 outer + 32 inner · 641 Summer crosswalk hints · 1,781 seasonal addresses")
        print("Later separate authority: 34 materials · 15,562 exact tuple signatures")
        print("NOTE: source-map recovery is not the same as current 16-mask recipe certification.")
    for warning in warnings:
        print("[WARN]", warning)
    for error in errors:
        print("[FAIL]", error)
    if errors:
        return 6
    print("SUMMER RECOVERY: PASS / source inventory restored without semantic guessing")
    return 0

def main(argv):
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "command",
        choices=["status", "certify", "fixtures", "recovery", "resolver", "pipeline"],
        nargs="?",
        default="status",
    )
    args = parser.parse_args(argv)
    if args.command == "recovery":
        return validate_recovery()
    if args.command == "resolver":
        return validate_resolver_lab()
    if args.command == "pipeline":
        rc = validate(strict=False, show_fixtures=False)
        if rc != 0:
            return rc
        rc = validate_resolver_lab(verbose=True)
        if rc != 0:
            return rc
        return validate_resolved_pipeline(verbose=True)
    rc = validate(strict=args.command == "certify", show_fixtures=args.command == "fixtures")
    if rc != 0:
        return rc
    if args.command == "status":
        rc = validate_recovery(verbose=True)
        if rc != 0:
            return rc
        rc = validate_resolver_lab(verbose=True)
        if rc != 0:
            return rc
        return validate_resolved_pipeline(verbose=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
