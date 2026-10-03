#!/usr/bin/env python3
"""Build Havenwild Bevy's single ElizaWy asset authority lane.

This tool never creates, edits, recolors, resamples, crops, or substitutes artwork.
It collapses existing source metadata/evidence into one deterministic source-controlled
registry consumed by editor/world-composition tooling.
"""
from __future__ import annotations
import argparse, hashlib, json, pathlib
from collections import Counter, defaultdict

SCHEMA = "havenwild.bevy.elizawy.asset_lane.v1"
OUT = pathlib.Path("content/assets/authority/elizawy_asset_lane.v1.json")
RUNTIME_OUT = pathlib.Path("content/assets/authority/elizawy_runtime_index.v1.json")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def region_key(path: str, rect) -> str:
    raw = f"{path}|{','.join(map(str, rect))}".encode("utf-8")
    return "hw.elizawy.region." + hashlib.sha256(raw).hexdigest()[:24]


def seasonal_variants(path: str) -> dict[str, str]:
    """Coordinate-equivalent seasonal sheets manually reviewed for M2D090.

    The Spring/Summer/Autumn/Winter terrain, tree, plant, wildflower and cliff
    sheets use the same canvas size and source layout. We still register every
    exact counterpart region in the canonical lane so runtime substitution
    remains source-addressed and fail-closed instead of assuming arbitrary files.
    """
    families = {
        "Terrain/terrain_summer.png": "Terrain/terrain_{season}.png",
        "Terrain/trees_summer.png": "Terrain/trees_{season}.png",
        "Terrain/plants_summer.png": "Terrain/plants_{season}.png",
        "Terrain/wildflowers_summer.png": "Terrain/wildflowers_{season}.png",
        "Terrain/cliff_summer.png": "Terrain/cliff_{season}.png",
    }
    pattern = families.get(path)
    if not pattern:
        return {}
    return {season: pattern.format(season=season) for season in ("spring", "autumn", "winter")}


def infer_worldgen_tags(label: str, parts: list[dict]) -> list[str]:
    text = (label + " " + " ".join(part["sourcePath"] for part in parts)).lower()
    tags: set[str] = set()
    for token, tag in [
        ("tree", "tree"), ("rock", "rock"), ("stone", "rock"),
        ("shrub", "shrub"), ("foliage", "foliage"),
        ("wildflower", "wildflower"), ("lily", "water_detail"),
        ("water-edge", "water_edge"), ("house", "house"),
    ]:
        if token in text:
            tags.add(tag)
    if any(tag in tags for tag in ("tree", "rock", "shrub", "foliage", "wildflower")):
        tags.add("nature")
    if "house" in tags:
        tags.update(("settlement", "large_structure"))
    if all(part["sourceRectPx"][2:] == [32, 32] for part in parts):
        tags.add("detail")
    return sorted(tags)


def build(root: pathlib.Path) -> dict:
    manifest_path = root / "content/catalog/core_source_manifest.json"
    historical_path = root / "content/mapping/recovered/region_candidates.v1.json"
    triage_path = root / "content/mapping/recovered/review_triage.v3.json"
    scene_path = root / "content/scenes/elizawy_mapping_certification.scene.json"
    seed_path = root / "content/scenes/summer_world.visual_seed.v1.json"
    summer_map_path = root / "content/terrain/recovered/native/lpc_terrain_summer_complete_map_32.native.v1.json"
    showcase_path = root / "content/worldgen/elizawy_showcase_templates.v1.json"

    manifest_raw = manifest_path.read_bytes()
    historical_raw = historical_path.read_bytes()
    triage_raw = triage_path.read_bytes()
    scene_raw = scene_path.read_bytes()
    seed_raw = seed_path.read_bytes()
    summer_map_raw = summer_map_path.read_bytes()
    showcase_raw = showcase_path.read_bytes()

    manifest = json.loads(manifest_raw)
    historical = json.loads(historical_raw)
    triage = json.loads(triage_raw)
    scene = json.loads(scene_raw)
    seed = json.loads(seed_raw)
    summer_map = json.loads(summer_map_raw)
    showcase = json.loads(showcase_raw)

    if manifest.get("schema") != "havenwild.bevy.asset_manifest.v1":
        raise SystemExit("unexpected core source manifest schema")
    if historical.get("schema") != "havenwild.bevy.elizawy.region_candidates.v1":
        raise SystemExit("unexpected historical region schema")
    if triage.get("schema") != "havenwild.bevy.elizawy.review_triage.v3":
        raise SystemExit("unexpected review triage schema")
    if scene.get("schema") != "havenwild.elizawy.scene.v1":
        raise SystemExit("unexpected scene schema")
    if seed.get("schema") != "havenwild.bevy.summer_visual_seed.v1":
        raise SystemExit("unexpected Summer seed schema")
    if summer_map.get("schema") != "havenwild.lpc_terrain_summer_complete_map.generated.v0_1":
        raise SystemExit("unexpected Native Summer complete-map schema")
    if summer_map.get("summary", {}).get("mappedNonTransparentCells") != 305:
        raise SystemExit("Native Summer complete map no longer proves 305/305 source cells")
    if showcase.get("schema") != "havenwild.worldgen.elizawy_showcase_templates.v1" or showcase.get("version") != 1:
        raise SystemExit("unexpected showcase template schema/version")

    images = []
    runtime_paths: set[str] = set()
    for entry in manifest["entries"]:
        if not entry["path"].endswith(".png"):
            continue
        runtime_paths.add(entry["path"])
        images.append({
            "assetId": entry["id"],
            "sourcePath": entry["path"],
            "sourceSha256": entry["sha256"],
            "bytes": entry["bytes"],
            "imageSizePx": entry["imageSizePx"],
            "family": entry["path"].split("/", 1)[0],
            "pack": entry.get("pack"),
            "runtimePlaceable": True,
            "sourceBytesPolicy": "immutable_hydrated_elizawy",
        })
    images.sort(key=lambda x: x["sourcePath"])
    manifest_by_path = {entry["path"]: entry for entry in manifest["entries"] if entry["path"].endswith(".png")}

    historical_sources = {}
    runtime_regions = defaultdict(lambda: {
        "historicalRegionIds": set(), "familyHints": set(), "seasons": set(),
        "matchStates": set(), "fixtureRoles": Counter(), "fixtureCount": 0,
        "objectTemplateUses": 0,
    })
    for entry in historical["entries"]:
        historical_sources[entry["regionId"]] = {
            "regionId": entry["regionId"],
            "sourcePath": entry["sourcePath"],
            "sourceCell": entry["sourceCell"],
            "sourceRectPx": entry["sourceRectPx"],
            "sourceImageSha256": entry["sourceImageSha256"],
            "sourcePixelSha256": entry["sourcePixelSha256"],
            "familyHint": entry["familyHint"],
            "transparent": entry["transparent"],
            "semanticState": entry["semanticReview"],
            "runtimeBinding": entry["editorRuntimeBinding"],
            "collisionState": entry["collision"],
            "seasonalCandidates": entry["seasonalCandidates"],
            "provenance": entry["sourceAddressEvidence"],
        }
        for candidate in entry["seasonalCandidates"]:
            atlas = candidate.get("canonicalAtlas")
            if atlas not in runtime_paths:
                continue
            for cell in candidate.get("exactCanonicalCells", []):
                rect = [cell[0] * 32, cell[1] * 32, 32, 32]
                rec = runtime_regions[(atlas, tuple(rect))]
                rec["historicalRegionIds"].add(entry["regionId"])
                rec["familyHints"].add(entry["familyHint"])
                rec["seasons"].add(candidate["season"])
                rec["matchStates"].add(candidate["canonicalMatchStatus"])


    # M2D081 promotes the complete Native Summer source inventory into the runtime
    # art authority. This does not assign gameplay semantics to every cell; it
    # merely makes every reviewed non-transparent 32x32 source cell selectable
    # and reusable by exact source address. Runtime meaning remains explicit.
    summer_atlas = "Terrain/terrain_summer.png"
    if summer_atlas not in runtime_paths:
        raise SystemExit("Summer atlas is missing from the canonical runtime lane")
    for cell in summer_map.get("cells", []):
        if not cell.get("nonTransparent"):
            continue
        rect = tuple(cell.get("pixelRect", []))
        if len(rect) != 4 or rect[2:] != (32, 32):
            raise SystemExit(f"invalid Summer complete-map cell rectangle: {rect}")
        rec = runtime_regions[(summer_atlas, rect)]
        group_id = cell.get("groupId")
        if group_id:
            rec["familyHints"].add(group_id)
        rec["seasons"].add("summer")
        rec["matchStates"].add("native_complete_map")

    for tile in scene["tiles"]:
        path = tile["source_asset"]
        rect = tuple(tile["source_rect"])
        if path not in runtime_paths:
            raise SystemExit(f"scene references source outside canonical runtime lane: {path}")
        rec = runtime_regions[(path, rect)]
        rec["fixtureRoles"][tile["role"]] += 1
        rec["fixtureCount"] += 1

    # Object-template source regions are also first-class runtime art authority.
    # The Summer visual seed contains larger exact regions (trees, structures,
    # foliage, rocks) that are not necessarily present in the historical 32x32
    # crosswalk or terrain fixture. They must be registered before templates are
    # emitted so every runtime placement can resolve through one canonical lane.
    for obj in seed["objects"]:
        for part in obj["parts"]:
            src = part["source"]
            path = src["source_asset"]
            rect = tuple(src["source_rect"])
            if path not in runtime_paths:
                raise SystemExit(f"seed object references source outside canonical runtime lane: {path}")
            runtime_regions[(path, rect)]["objectTemplateUses"] += 1

    # M2D090 adds a small, explicitly reviewed set of complete source sprites /
    # source sections for the Generated World showcase. These are source-address
    # declarations only; no PNG bytes are copied or modified.
    for template in showcase["templates"]:
        for part in template["parts"]:
            path = part["sourcePath"]
            rect = tuple(part["sourceRectPx"])
            entry = manifest_by_path.get(path)
            if entry is None:
                raise SystemExit(f"showcase template source absent from manifest: {path}")
            width, height = entry["imageSizePx"]
            x, y, w, h = rect
            if min(x, y, w, h) < 0 or w <= 0 or h <= 0 or x + w > width or y + h > height:
                raise SystemExit(f"showcase template source rectangle out of bounds: {path} {rect}")
            rec = runtime_regions[(path, rect)]
            rec["objectTemplateUses"] += 1
            rec["familyHints"].add("m2d090_showcase_review")
            rec["matchStates"].add("manual_source_region_review")

    # Register coordinate-equivalent seasonal counterparts for every canonical
    # Summer region currently in the runtime lane. This is what lets the renderer
    # use actual Spring/Autumn/Winter source pixels without inventing art or
    # silently treating arbitrary sheets as interchangeable.
    seasonal_seed = list(runtime_regions.items())
    for (path, rect), rec in seasonal_seed:
        variants = seasonal_variants(path)
        if not variants:
            continue
        for season, counterpart in variants.items():
            entry = manifest_by_path.get(counterpart)
            if entry is None:
                raise SystemExit(f"seasonal counterpart missing from manifest: {counterpart}")
            width, height = entry["imageSizePx"]
            x, y, w, h = rect
            if x + w > width or y + h > height:
                raise SystemExit(f"seasonal counterpart rectangle out of bounds: {counterpart} {rect}")
            target = runtime_regions[(counterpart, rect)]
            target["familyHints"].update(rec["familyHints"] | {"m2d090_coordinate_equivalent_seasonal_layout"})
            target["seasons"].add(season)
            target["matchStates"].add("manual_coordinate_equivalence")
            target["objectTemplateUses"] += rec["objectTemplateUses"]

    runtime_region_rows = []
    for (path, rect), rec in sorted(runtime_regions.items()):
        runtime_region_rows.append({
            "canonicalRegionId": region_key(path, rect),
            "sourceAssetId": f"elizawy:{path}",
            "sourcePath": path,
            "sourceRectPx": list(rect),
            "historicalRegionIds": sorted(rec["historicalRegionIds"]),
            "familyHints": sorted(rec["familyHints"]),
            "seasons": sorted(rec["seasons"]),
            "matchStates": sorted(rec["matchStates"]),
            "fixtureRoleEvidence": [
                {"role": role, "count": count}
                for role, count in sorted(rec["fixtureRoles"].items())
            ],
            "fixtureUseCount": rec["fixtureCount"],
            "objectTemplateUseCount": rec["objectTemplateUses"],
            "semanticAuthority": (
                "fixture_role_evidence_only" if rec["fixtureCount"]
                else "source_exact_visual_study_object_seed" if rec["objectTemplateUses"]
                else "historical_crosswalk_only"
            ),
            "runtimeArtAuthority": "exact_elizawy_source_region",
            "generatedArtwork": False,
        })

    templates = {}
    for obj in seed["objects"]:
        parts = []
        for part in obj["parts"]:
            src = part["source"]
            if src["source_asset"] not in runtime_paths:
                raise SystemExit(f"seed object references source outside canonical runtime lane: {src['source_asset']}")
            parts.append({
                "offsetPx": part["offset"],
                "canonicalRegionId": region_key(src["source_asset"], src["source_rect"]),
                "sourcePath": src["source_asset"],
                "sourceRectPx": src["source_rect"],
            })
        signature = json.dumps([obj["label"], obj["layer"], obj["footprint"], parts], sort_keys=True)
        template_id = "hw.elizawy.template." + hashlib.sha256(signature.encode()).hexdigest()[:20]
        row = templates.setdefault(template_id, {
            "templateId": template_id,
            "label": obj["label"],
            "layer": obj["layer"],
            "footprintCells": obj["footprint"],
            "parts": parts,
            "observedInstances": 0,
            "source": "content/scenes/summer_world.visual_seed.v1.json",
            "authority": "source_exact_visual_study",
            "worldgenEligible": True,
            "defaultCollision": "unassigned_use_asset_or_world_override",
            "worldgenTags": infer_worldgen_tags(obj["label"], parts),
        })
        row["observedInstances"] += 1

    for obj in showcase["templates"]:
        parts = []
        for part in obj["parts"]:
            path = part["sourcePath"]
            rect = part["sourceRectPx"]
            parts.append({
                "offsetPx": part["offsetPx"],
                "canonicalRegionId": region_key(path, rect),
                "sourcePath": path,
                "sourceRectPx": rect,
            })
        signature = json.dumps([obj["key"], obj["label"], obj["layer"], obj["footprintCells"], parts], sort_keys=True)
        template_id = "hw.elizawy.showcase." + hashlib.sha256(signature.encode()).hexdigest()[:20]
        if template_id in templates:
            raise SystemExit(f"duplicate showcase template identity: {obj['key']}")
        templates[template_id] = {
            "templateId": template_id,
            "label": obj["label"],
            "layer": obj["layer"],
            "footprintCells": obj["footprintCells"],
            "parts": parts,
            "observedInstances": 1,
            "source": str(showcase_path.relative_to(root)).replace('\\','/'),
            "authority": "manual_source_region_review",
            "worldgenEligible": True,
            "defaultCollision": "unassigned_use_asset_or_world_override",
            "worldgenTags": sorted(set(obj.get("worldgenTags", []))),
        }

    fixture_role_palette = defaultdict(list)
    for row in runtime_region_rows:
        for evidence in row["fixtureRoleEvidence"]:
            fixture_role_palette[evidence["role"]].append({
                "canonicalRegionId": row["canonicalRegionId"],
                "sourcePath": row["sourcePath"],
                "sourceRectPx": row["sourceRectPx"],
                "weight": evidence["count"],
            })

    # Derive an observed cardinal-neighbor topology palette from the existing
    # source-authored 40x28 Summer fixture. This does NOT certify a universal
    # autotile recipe; it simply lets world composition prefer source regions
    # seen with the same local role-neighbor pattern instead of random edges.
    width, height = scene["size"]
    tiles = scene["tiles"]
    topo_counts = defaultdict(Counter)
    directions = [(0, -1, 1), (1, 0, 2), (0, 1, 4), (-1, 0, 8)]  # N,E,S,W
    for y in range(height):
        for x in range(width):
            tile = tiles[y * width + x]
            role = tile["role"]
            mask = 0
            for dx, dy, bit in directions:
                nx, ny = x + dx, y + dy
                if 0 <= nx < width and 0 <= ny < height and tiles[ny * width + nx]["role"] == role:
                    mask |= bit
            key = (tile["source_asset"], tuple(tile["source_rect"]))
            topo_counts[(role, mask)][key] += 1
    fixture_topology_palette = {}
    for (role, mask), counts in sorted(topo_counts.items()):
        rows = []
        for (path, rect), weight in counts.most_common():
            rows.append({
                "canonicalRegionId": region_key(path, rect),
                "sourcePath": path,
                "sourceRectPx": list(rect),
                "weight": weight,
            })
        fixture_topology_palette[f"{role}:{mask:02x}"] = rows

    return {
        "schema": SCHEMA,
        "version": 1,
        "policy": {
            "artAuthority": "ElizaWy hydrated repository only",
            "generatedArtworkAllowed": False,
            "worldGenerationMeaning": "generate world composition/placement only; never generate or synthesize image assets",
            "sourcePixelsImmutable": True,
            "semanticPromotion": "explicit review only",
            "historicalEvidenceAutoPromotion": False,
            "runtimePlacementMustReferenceCanonicalSourceRegion": True,
        },
        "inputs": {
            "coreSourceManifest": {"path": str(manifest_path.relative_to(root)).replace('\\','/'), "sha256": sha256_bytes(manifest_raw)},
            "historicalRegionCandidates": {"path": str(historical_path.relative_to(root)).replace('\\','/'), "sha256": sha256_bytes(historical_raw)},
            "historicalReviewTriage": {"path": str(triage_path.relative_to(root)).replace('\\','/'), "sha256": sha256_bytes(triage_raw)},
            "summerFixture": {"path": str(scene_path.relative_to(root)).replace('\\','/'), "sha256": sha256_bytes(scene_raw)},
            "summerObjectSeed": {"path": str(seed_path.relative_to(root)).replace('\\','/'), "sha256": sha256_bytes(seed_raw)},
            "nativeSummerCompleteMap": {"path": str(summer_map_path.relative_to(root)).replace('\\','/'), "sha256": sha256_bytes(summer_map_raw)},
            "showcaseTemplates": {"path": str(showcase_path.relative_to(root)).replace('\\','/'), "sha256": sha256_bytes(showcase_raw)},
        },
        "counts": {
            "runtimeSourceImages": len(images),
            "historicalSourceRegions": len(historical_sources),
            "canonicalRuntimeRegions": len(runtime_region_rows),
            "fixtureRoleRegions": sum(1 for r in runtime_region_rows if r["fixtureUseCount"]),
            "objectTemplates": len(templates),
            "historicalReviewClusters": triage["clusterCount"],
        },
        "runtimeSourceImages": images,
        "historicalSourceRegions": [historical_sources[k] for k in sorted(historical_sources)],
        "canonicalRuntimeRegions": runtime_region_rows,
        "fixtureRolePalette": {role: rows for role, rows in sorted(fixture_role_palette.items())},
        "fixtureTopologyPalette": fixture_topology_palette,
        "objectTemplates": [templates[k] for k in sorted(templates)],
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=".")
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    root = pathlib.Path(args.root).resolve()
    out = root / OUT
    built = build(root)
    data = (json.dumps(built, indent=2, sort_keys=False) + "\n").encode("utf-8")
    runtime = {
        "schema": "havenwild.bevy.elizawy.runtime_index.v1",
        "version": 1,
        "policy": built["policy"],
        "lane": str(OUT).replace("\\", "/"),
        "laneSha256": sha256_bytes(data),
        "counts": built["counts"],
        "runtimeSourceImages": built["runtimeSourceImages"],
        "canonicalRuntimeRegions": built["canonicalRuntimeRegions"],
        "fixtureRolePalette": built["fixtureRolePalette"],
        "fixtureTopologyPalette": built["fixtureTopologyPalette"],
        "objectTemplates": built["objectTemplates"],
    }
    runtime_data = (json.dumps(runtime, indent=2, sort_keys=False) + "\n").encode("utf-8")
    runtime_out = root / RUNTIME_OUT
    if args.check:
        for check_path, expected in [(out, data), (runtime_out, runtime_data)]:
            if not check_path.is_file():
                raise SystemExit(f"missing generated authority lane artifact: {check_path}")
            if check_path.read_bytes() != expected:
                raise SystemExit(f"ElizaWy asset lane artifact is stale: {check_path}")
        print(f"ELIZAWY ASSET LANE: PASS / {built['counts']}")
        return 0
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(data)
    runtime_out.write_bytes(runtime_data)
    print(f"WROTE {out} and {runtime_out} / {built['counts']}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
