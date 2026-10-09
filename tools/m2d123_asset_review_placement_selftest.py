#!/usr/bin/env python3
"""Check reviewed-object placement semantics without depending on Rust whitespace.

M2D125 repairs a false gate triggered by source-format-dependent string checks.
This contract continues to reject unknown templates, unaudited alpha regions,
missing source bindings, bridge misuse, and missing persistent world changes.
Native Cargo tests independently exercise actual object placement.
"""
import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from m2d123_alpha_source_review import TARGET, manifest, validate


def ensure(condition, message):
    if not condition:
        raise AssertionError(message)


def compact(text):
    return re.sub(r"\s+", "", text)


def verify_world_placement(source, mutation_probes=False):
    """Inspect only the placement method; unrelated occurrences cannot satisfy the gate."""
    method = re.search(r"\bpub\s+fn\s+place_reviewed_template\s*\(", source)
    ensure(method is not None, "Missing WorldDocument::place_reviewed_template")
    tail = source[method.start():]
    boundary = re.search(r"\n\s*///\s*Builds a persistent authored bridge|\n\s*pub\s+fn\s+", tail[30:])
    implementation = tail[:boundary.start() + 30] if boundary else tail[:5500]
    normalized = compact(implementation)
    requirements = {
        "reviewed template authority": "authority.object_templates",
        "template ID lookup": ".find(",
        "unknown template rejection": "Unknown/unreviewedcompleteobjecttemplate",
        "valid object/structure layer": '"objects"|"structures"',
        "nonempty template geometry": "template.parts.is_empty()",
        "bridge routed to dedicated tool": 'tag=="bridge"',
        "full footprint bounds": "template.footprint_cells",
        "canonical source check": "authority.canonical_region(&source).is_none()",
        "rejection for missing canonical region": "Reviewedtemplatelostitscanonicalsourcebinding",
        "persistent authored object": "self.authored_objects.push(WorldObject",
        "world revision update": "self.bump();",
    }
    missing = [label for label, token in requirements.items() if token not in normalized]
    ensure(not missing, "Reviewed placement implementation incomplete: " + ", ".join(missing))
    ensure("alpha_candidates" not in implementation,
           "Unreviewed alpha candidates must never enter place_reviewed_template")

    if mutation_probes:
          # Deliberately damage this method and verify critical safeguards are detected.
        method_start = source.index("pub fn place_reviewed_template(")
        for label in ("canonical source check", "bridge routed to dedicated tool",
                      "persistent authored object", "world revision update"):
            marker = requirements[label]
            normalized_tail = compact(source[method_start:])
            ensure(marker in normalized_tail, "Mutation probe prerequisite missing: " + label)
            damaged_tail = normalized_tail.replace(marker, "", 1)
            try:
                verify_world_placement(source[:method_start] + damaged_tail)
            except AssertionError:
                pass
            else:
                raise AssertionError("Missing safety guard went undetected: " + label)
        # A harmless formatting change must not fail the same semantic test.
        reformatted = source.replace("authority.object_templates.iter()",
                                     "authority\n  .object_templates\n  .iter()", 1)
        ensure("authority\n  .object_templates" in reformatted,
               "Formatting resilience probe could not be constructed")
        # Avoid recursive probes: check the normalized requirement directly.
        ensure("authority.object_templates" in compact(reformatted[method_start:]),
               "Whitespace normalization regression")
    return hashlib.sha256(source.encode("utf-8")).hexdigest()


def main():
    report = json.loads(TARGET.read_text(encoding="utf-8"))
    original_manifest = manifest()
    ensure(len(original_manifest) == 320, "Expected original 320-file PNG authority")
    ensure(not validate(report, original_manifest), "M2D123 census failed validation")
    ensure(report["automaticApprovals"] == 0 and report["originalPixelsModified"] is False,
           "Alpha census must remain read-only and promote zero objects")
    ensure(sum(len(s["isolatedGroups"]) for s in report["sheets"]) == 238,
           "Isolated group census drift")
    ensure(sum(s["crowdedGroups"] for s in report["sheets"]) == 138,
           "Crowded groups census drift")
    ensure(sum(any(s["unalignedEdgesPx"]) for s in report["sheets"]) == 5,
           "Non-32px source edge tracking drift")
    index = json.loads((ROOT / "content/assets/authority/elizawy_runtime_index.v1.json").read_text(encoding="utf-8"))
    ensure(len(index["objectTemplates"]) == 28, "Reviewed object template count drift")

    for key, value in (
        ("sourceSha256", "0" * 64),
        ("sourcePath", "Objects/not-in-original.png"),
        ("isolatedGroups", [{"rectPx": [0, 0, 65, 32], "occupiedCells": 1}]),
    ):
        corrupt = json.loads(json.dumps(report))
        corrupt["sheets"][0][key] = value
        ensure(bool(validate(corrupt, original_manifest)),
               "Corrupted alpha candidate evidence accepted: " + key)

    review = (ROOT / "src/asset_review.rs").read_text(encoding="utf-8")
    ui = (ROOT / "src/main.rs").read_text(encoding="utf-8")
    world = (ROOT / "src/world_doc.rs").read_text(encoding="utf-8")
    for token in ("Original-alpha census incomplete", "source_sha256 != hash",
                  "Unrecognized candidate source", "Duplicate original-alpha source entry"):
        ensure(token in review, "Missing alpha review validation: " + token)
    for token in ("mod asset_review;", "asset_review::AlphaCandidateIndex::load(",
                  "state.alpha_candidates.sheet(&path)",
                  '"Unreviewed alpha regions (inspect only)"',
                  '"Place reviewed complete objects"',
                  "state.world_doc.place_reviewed_template(",
                  "world_template_placement: Option<String>",
                  "state.world_template_placement.take().is_some()",
                  "checkpoint_world(state);"):
        ensure(compact(token) in compact(ui), "Missing reviewed-object UI binding: " + token)
    sha = verify_world_placement(world, mutation_probes=True)
    print("M2D125 M2D123 PLACEMENT CONTRACT: PASS / reviewed templates guarded; alpha evidence read-only; "
          f"world_doc SHA256={sha[:16]} / 28 templates / 320 original PNG identities")


if __name__ == "__main__":
    main()
