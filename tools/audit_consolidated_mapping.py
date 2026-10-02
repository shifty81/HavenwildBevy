#!/usr/bin/env python3
"""Static, zero-promotion validation of the source-controlled historical mapping.
Does NOT require workstation art or claim semantic/DG/runtime certification.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / 'content' / 'mapping' / 'recovered'

def load(name: str):
    return json.loads((BASE / name).read_text(encoding='utf-8'))

def audit() -> None:
    source = load('source_registry.v1.json')
    regions = load('region_candidates.v1.json')
    triage = load('review_triage.v3.json')
    ledger = load('review_ledger.v1.json')
    report = load('consolidation_audit.v1.json')
    raw = (BASE / 'elizawy_all_seasons_split_source_crosswalk_b48r9_v0_1.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest() == report['exactSourceCrosswalkSha256']
    crosswalk = json.loads(raw)
    assert source['schema'] == 'havenwild.bevy.elizawy.source_registry.candidate.v1'
    assert regions['schema'] == 'havenwild.bevy.elizawy.region_candidates.v1'
    assert len(source['sourceSheets']) == source['sourceCount'] == 40 == report['sourceSheets']
    assert len(regions['entries']) == regions['regionCount'] == report['sourceRegions'] == 1781
    assert source['runtimeEnabled'] is False and regions['runtimeEnabled'] is False
    assert report['semanticMappingsPromoted'] == 0 and report['currentLocalAssetHashesChecked'] == 0
    assert len(crosswalk['sourceSheets']) == 40
    sheets = {sheet['path']: sheet for sheet in source['sourceSheets']}
    seen = set()
    statuses = {'one': 0, 'multiple': 0, 'none': 0}
    for region in regions['entries']:
        rid = region['regionId']
        assert rid not in seen
        seen.add(rid)
        sheet = sheets[region['sourcePath']]
        x, y, w, h = region['sourceRectPx']
        assert w == h == 32 and x == region['sourceCell'][0] * 32 and y == region['sourceCell'][1] * 32
        assert 0 <= x and x + w <= sheet['pixelDimensions'][0]
        assert 0 <= y and y + h <= sheet['pixelDimensions'][1]
        assert region['sourceImageSha256'] == sheet['sourceByteSha256']
        assert region['sourceAddressEvidence'] == 'b48r9_verified_historical'
        assert region['semanticReview'] == 'not_promoted'
        assert region['editorRuntimeBinding'] == region['collision'] == 'not_certified'
        assert region['currentSourceBytesVerified'] is False and region['selection'] is None
        for target in region['seasonalCandidates']:
            status = target['canonicalMatchStatus']
            assert status in statuses
            n = len(target['exactCanonicalCells'])
            assert (status == 'none' and n == 0) or (status == 'one' and n == 1) or (status == 'multiple' and n >= 2)
            statuses[status] += 1
    assert statuses == report['targetMatchCounts'], (statuses, report['targetMatchCounts'])
    assert triage['schema'] == 'havenwild.bevy.elizawy.review_triage.v3'
    assert triage['authority'] == 'historical_evidence_only'
    assert len(triage['clusters']) == triage['clusterCount'] == 284
    assert sum(c['memberCount'] for c in triage['clusters']) == triage['rowCount'] == 787
    assert all(c['approved'] is False and c['selectedAtlasCell'] is None for c in triage['clusters'])
    assert ledger['regionApprovals'] == [] and ledger['runtimeCertifications'] == []
    assert report['sourceAddressHistoricallyVerified'] == 1781
    print(f"CONSOLIDATED ELIZAWY: PASS / {len(sheets)} sheets / {len(seen)} regions / {sum(statuses.values())} season-target comparisons")
    print(f"REVIEW: {triage['clusterCount']} clusters / {triage['rowCount']} comparison records / 0 promotions / local PNG verification pending")

if __name__ == '__main__':
    audit()
