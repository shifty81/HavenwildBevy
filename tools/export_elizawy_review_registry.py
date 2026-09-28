#!/usr/bin/env python3
"""M2C4: complete, read-only ElizaWy/Tiled per-cell review evidence export.

This file deliberately generates NO certified terrain recipes, runtime tile IDs,
source images, or semantic promotions. Existing original ElizaWy art and the fixed
recovered Summer map remain separate authorities. The external TSX is evidence only.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import hashlib
import io
import json
from pathlib import Path
import sys
import zipfile

from reconcile_elizawy_tiled import (
    KNOWN_SHA, SEASONS, get_manifest, get_map, reconcile, season_from_name,
    sha, tile_definitions, wang_data,
)

SCHEMA = "havenwild.elizawy.tiled_review_registry.v1"
DEFAULT = Path(".forgepy/recovered_mapping/elizawy_tiled_review_registry.v1.json")
STATUS = ("pixel_unique_candidate", "pixel_ambiguous_candidates", "no_original_terrain_cell_match")


def sha_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def _tile_metadata(tsx_blob: bytes) -> tuple[dict, dict]:
    sets = wang_data(tsx_blob)
    inverse = defaultdict(dict)
    for name, value in sets.items():
        for tile_id, signature in value['assignments'].items():
            inverse[tile_id][name] = {
                "type": value['type'], "signature": signature,
                "colors": value['colors'],
                "authority": "external_tiled_evidence_only",
            }
    return inverse, tile_definitions(tsx_blob)


def validate_registry(document: dict, root: Path | None = None) -> list[str]:
    errors = []
    if document.get('schema') != SCHEMA:
        return ['Unexpected review registry schema']
    if document.get('authority') != 'evidence_only_zero_promotions' or document.get('certifiedMappingsAdded') != 0 or document.get('runtimeChanged') is not False or document.get('sourceFilesMutated') is not False:
        errors.append('Registry must be non-authoritative and preserve zero-mutation contract')
    if document.get('archiveSha256') != KNOWN_SHA or document.get('historicalMapState') != 'sha256_verified':
        errors.append('Archive or original map provenance is not verified')
    if set(document.get('seasons', {})) != set(SEASONS):
        errors.append('Exactly four seasonal evidence entries are required')
    for season, profile in document.get('seasons', {}).items():
        records = profile.get('cells', [])
        if not isinstance(records, list):
            errors.append(f'{season}: missing cell rows'); continue
        ids = set()
        counts = Counter()
        for c in records:
            tileid = c.get('derivativeTileId'); coord = c.get('derivativeCell')
            if not isinstance(tileid, int) or isinstance(tileid, bool) or not 0 <= tileid < 4096 or coord != [tileid % 64, tileid // 64] or tileid in ids:
                errors.append(f'{season}: invalid/duplicate derivative tile id or coordinate'); break
            ids.add(tileid)
            status = c.get('matchStatus')
            candidates = c.get('originalCandidates')
            if status not in STATUS or not isinstance(candidates, list) or ((status == STATUS[0]) != (len(candidates) == 1)) or ((status == STATUS[1]) != (len(candidates) > 1)) or ((status == STATUS[2]) != (len(candidates) == 0)):
                errors.append(f'{season}: invalid candidate/status relationship'); break
            for match in candidates:
                pos = match.get('sourceCell')
                if not isinstance(pos, list) or len(pos) != 2 or not all(isinstance(x, int) and not isinstance(x, bool) for x in pos) or not (0 <= pos[0] < 16 and 0 <= pos[1] < 26):
                    errors.append(f'{season}: out-of-bounds original source cell'); break
            if c.get('reviewState') != 'unreviewed' or c.get('mappingCertified') is not False:
                errors.append(f'{season}: promotion/approval detected'); break
            counts[status] += 1
        summary = profile.get('summary', {})
        expected = {
            'occupiedDerivativeCells': len(records),
            'uniqueCandidateCells': counts[STATUS[0]],
            'ambiguousCandidateCells': counts[STATUS[1]],
            'unmatchedDerivativeCells': counts[STATUS[2]],
        }
        for key, count in expected.items():
            if summary.get(key) != count:
                errors.append(f'{season}: {key} count does not match actual rows')
        if len(records) != 2526 or len(ids) != 2526:
            errors.append(f'{season}: expected 2526 occupied derivative cells from pinned archive')
        if not profile.get('originalSourceSha256') or profile.get('originalPixelState') != 'original_sha_verified_rgba32_exact_match':
            errors.append(f'{season}: original image provenance missing')
    if root is not None:
        # Validation of the locally-generated file requires the original recovered
        # index and actual source to be available; never trust a copied stale report.
        map_doc, map_state = get_map(root, None)
        if map_doc is None or map_state != 'sha256_verified':
            errors.append('No valid local SHA-verified recovered historical source map')
        manifest = get_manifest(root)
        for season, profile in document.get('seasons', {}).items():
            key = f'Terrain/terrain_{season}.png'
            if profile.get('originalSourceSha256') != manifest.get(key, {}).get('sha256'):
                errors.append(f'{season}: original source hash no longer agrees with source manifest')
        # The archive is optional at gate time, but the recorded metadata cannot
        # be treated as independent evidence if its documented hash changes.
    return errors


def build_registry(archive: Path, root: Path, original_root: Path, summer_map: Path | None = None) -> dict:
    # Run the complete existing archive/member/TSX provenance checks first.
    report = reconcile(archive, root, original_root, summer_map, strict_archive_sha=True)
    if report['errors'] or not report['matchesAuditedUpload']:
        raise ValueError('Prior M2C3 reconciliation failed: ' + '; '.join(report['errors']))
    if report.get('historicalMapState') != 'sha256_verified':
        raise ValueError('Historical source map was not SHA verified')
    source_map, source_state = get_map(root, summer_map)
    if source_state != 'sha256_verified' or source_map is None:
        raise ValueError('Recovered Summer source map unavailable or hash mismatch')
    evidence = {tuple(c['cell']): c for c in source_map['cells']}
    manifest = get_manifest(root)
    try:
        from PIL import Image
    except ImportError as exc:
        raise ValueError('Pillow is required for the full per-cell image concordance') from exc
    out = {
        'schema': SCHEMA, 'authority': 'evidence_only_zero_promotions',
        'originalSourceRevision': report['originalSourceRevision'],
        'archiveSha256': sha_file(archive), 'historicalMapSha256': sha_file(summer_map) if summer_map else __import__('reconcile_elizawy_tiled').SUMMER_MAP_SHA,
        'historicalMapState': source_state, 'seasonProfilesSource': 'TSX Wang/animation/collision are derivative evidence; never overwrite native source roles',
        'upstreamWarnings': report['warnings'], 'seasons': {},
        'certifiedMappingsAdded': 0, 'runtimeChanged': False, 'sourceFilesMutated': False,
    }
    with zipfile.ZipFile(archive) as z:
        by_season = {season_from_name(name): name for name in z.namelist() if season_from_name(name)}
        for season in SEASONS:
            original_rel = f'Terrain/terrain_{season}.png'
            original = original_root / original_rel
            expected = manifest.get(original_rel, {}).get('sha256')
            if not original.is_file() or not expected or sha_file(original) != expected:
                raise ValueError(f'{season}: original source missing or not hash verified')
            original_image = Image.open(original).convert('RGBA')
            derivative_image = Image.open(io.BytesIO(z.read(f'lpc-tileset-terrain-{season}.png'))).convert('RGBA')
            if original_image.size != (512, 832) or derivative_image.size != (2048, 2048):
                raise ValueError(f'{season}: unexpected dimensions')
            source_hashes = defaultdict(list)
            for y in range(26):
                for x in range(16):
                    cell = original_image.crop((x * 32, y * 32, x * 32 + 32, y * 32 + 32))
                    if cell.getchannel('A').getbbox():
                        source_hashes[sha(cell.tobytes())].append([x, y])
            reverse_wang, definitions = _tile_metadata(z.read(by_season[season]))
            cells = []
            count = Counter()
            for tile_id in range(4096):
                x, y = tile_id % 64, tile_id // 64
                tile = derivative_image.crop((x * 32, y * 32, x * 32 + 32, y * 32 + 32))
                if not tile.getchannel('A').getbbox():
                    continue
                matches = source_hashes.get(sha(tile.tobytes()), [])
                match_status = STATUS[0] if len(matches) == 1 else STATUS[1] if matches else STATUS[2]
                original_candidates = []
                for cell_coord in matches:
                    hist = evidence.get(tuple(cell_coord), {})
                    original_candidates.append({
                        'sourceCell': cell_coord, 'sourceSheet': original_rel,
                        'historicalSummerGroupId': hist.get('groupId'),
                        'historicalSummerRole': hist.get('role'),
                        'historicalSummerRoleIsNotSeasonCertification': True,
                    })
                definition = definitions.get(tile_id, {})
                cells.append({
                    'derivativeTileId': tile_id, 'derivativeCell': [x, y],
                    'matchStatus': match_status, 'originalCandidates': original_candidates,
                    'tiledWangEvidence': reverse_wang.get(tile_id, {}),
                    'tiledTileType': definition.get('tileType'),
                    'tiledHasCollisionDefinition': bool(definition.get('hasCollision')),
                    'tiledAnimationFrames': definition.get('frames', []),
                    'reviewState': 'unreviewed', 'mappingCertified': False,
                })
                count[match_status] += 1
            match_stats = report['pixelConcordance']['matches'][season]
            if match_stats['state'] != 'original_sha_verified_rgba32_exact_match':
                raise ValueError(f'{season}: prior pixel comparison not successful')
            expected_count = (match_stats['uniqueCandidateCells'], match_stats['ambiguousCandidateCells'], match_stats['unmatchedDerivativeOccupiedCells'])
            actual_count = tuple(count[k] for k in STATUS)
            if expected_count != actual_count:
                raise ValueError(f'{season}: detailed per-cell counts disagree with M2C3 report {actual_count} != {expected_count}')
            out['seasons'][season] = {
                'tsxFile': by_season[season], 'tsxSha256': report['seasonProfiles'][season]['tsxSha256'],
                'derivativeImageSha256': report['seasonProfiles'][season]['imageSha256'],
                'originalSourceSha256': expected,
                'originalPixelState': match_stats['state'],
                'summary': {
                    'occupiedDerivativeCells': len(cells),
                    'uniqueCandidateCells': count[STATUS[0]],
                    'ambiguousCandidateCells': count[STATUS[1]],
                    'unmatchedDerivativeCells': count[STATUS[2]],
                    'candidateMappingsCertified': 0,
                },
                'cells': cells,
            }
    failures = validate_registry(out, root if summer_map is None else None)
    if failures:
        raise ValueError('Registry self-validation failed: ' + '; '.join(failures[:10]))
    return out


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    p.add_argument('--output', type=Path)
    p.add_argument('--verify', type=Path, help='Verify an existing registry without the third-party ZIP or Pillow')
    p.add_argument('--original-root', type=Path, help='ForgePY-hydrated original source root (contains Terrain/)')
    p.add_argument('--summer-map', type=Path, help='Optional fixed-SHA historical Summer map for isolated testing')
    p.add_argument('--strict-archive-sha', action='store_true', help='Documentary flag; strict hash verification is always enforced')
    p.add_argument('archive', nargs='?', type=Path, help='External original ZIP, never copied into source/patch')
    a = p.parse_args(argv)
    root = a.root.resolve()
    try:
        if a.verify:
            path = a.verify.resolve()
            document = json.loads(path.read_text(encoding='utf-8'))
            errors = validate_registry(document, root)
            if errors:
                raise ValueError('; '.join(errors[:10]))
            print('ELIZAWY M2C4 REGISTRY VERIFY: PASS / full per-cell evidence / 0 promotions / no archive required')
            return 0
        if not a.archive or not a.original_root:
            p.error('export requires archive and --original-root; --verify requires a registry JSON')
        output = a.output or root / DEFAULT
        doc = build_registry(a.archive.resolve(), root, a.original_root.resolve(), a.summer_map.resolve() if a.summer_map else None)
        output.parent.mkdir(parents=True, exist_ok=True)
        # No original source/recipe mutation; only generated local .forgepy output by default.
        output.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
        print('ELIZAWY M2C4 REVIEW REGISTRY: PASS / four seasons / 0 certified / read-only')
        for season in SEASONS:
            s = doc['seasons'][season]['summary']
            print(f'  {season:7s} occupied={s["occupiedDerivativeCells"]} unique={s["uniqueCandidateCells"]} ambiguous={s["ambiguousCandidateCells"]} unmatched={s["unmatchedDerivativeCells"]}')
        print(f'REGISTRY: {output}')
        return 0
    except (ValueError, OSError, KeyError, TypeError, json.JSONDecodeError, zipfile.BadZipFile) as exc:
        print(f'ELIZAWY M2C4: FAIL / {exc}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
