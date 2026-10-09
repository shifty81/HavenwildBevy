#!/usr/bin/env python3
"""M2D096E: keep flowers/wildflowers in the Objects/vegetation composition layer."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def req(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D096E VEGETATION LAYER SELFTEST: FAIL - {message}")


def main() -> int:
    seed = json.loads((ROOT / "content/scenes/summer_world.visual_seed.v1.json").read_text(encoding="utf-8"))
    runtime = json.loads((ROOT / "content/assets/authority/elizawy_runtime_index.v1.json").read_text(encoding="utf-8"))
    world = (ROOT / "src/world_doc.rs").read_text(encoding="utf-8")
    main_rs = (ROOT / "src/main.rs").read_text(encoding="utf-8")

    flowers = [o for o in seed.get("objects", []) if o.get("id", "").startswith("summer.flowers.")]
    req(len(flowers) == 7, f"expected 7 source-exact Summer wildflower fixtures, got {len(flowers)}")
    req(all(o.get("layer") == "objects" for o in flowers), "Summer wildflowers must live on Objects")

    vegetation_images = [
        image for image in runtime.get("runtimeSourceImages", [])
        if "flower" in image.get("sourcePath", "").lower()
    ]
    req(vegetation_images, "flower source images missing from runtime authority")
    req(all(image.get("semanticCategory") == "Vegetation" for image in vegetation_images),
        "flower source images are not semantically classified as Vegetation")

    templates = [
        t for t in runtime.get("objectTemplates", [])
        if "wildflower" in set(t.get("worldgenTags") or [])
    ]
    req(templates, "runtime authority has no wildflower template")
    req(all(t.get("layer") == "objects" for t in templates), "wildflower runtime templates must use Objects")
    req(all(t.get("worldgenEligible") is True for t in templates), "wildflower template lost worldgen eligibility")

    req('kind == "wildflower" || has_tag("wildflower")' in world, "generated wildflower routing missing")
    req('kind == "wildflower" || has_tag("wildflower") {\n        "objects".into()' in world,
        "generated wildflowers are not routed to Objects")
    req('"Objects / vegetation / forageables"' in main_rs, "Objects rail is not the vegetation authoring destination")
    req('"Vegetation", "Structure"' in main_rs, "Source Browser has no Vegetation semantic filter")
    req('entry.semantic_category == family' in main_rs, "Source Browser ignores semantic source categories")

    print("M2D096E VEGETATION LAYER SELFTEST: PASS")
    print(f"  source fixture : {len(flowers)} wildflower placements -> Objects")
    print(f"  runtime        : {len(templates)} wildflower template(s) -> Objects / worldgen eligible")
    print(f"  source browser : {len(vegetation_images)} flower source image(s) -> Vegetation semantic category")
    print("  source family  : Terrain path provenance preserved; placement semantics are vegetation Objects")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
