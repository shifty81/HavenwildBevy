#!/usr/bin/env python3
"""M2C4 PCC gate verification for local external-derivative evidence.

External archive is deliberately not a required source dependency at Full Gate:
if evidence is absent, report an explicit SKIP/NOT CERTIFIED, not a false mapping
certification. If present, malformed/unsafe evidence fails closed.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import sys
from export_elizawy_review_registry import validate_registry, DEFAULT
from reconcile_elizawy_tiled import KNOWN_SHA, SEASONS

REPORT = Path('.forgepy/recovered_mapping/external_tiled_concordance.v1.json')
EXPECTED = {
    'summer': (512, 29, 1985),
    'spring': (512, 29, 1985),
    'autumn': (512, 29, 1985),
    'winter': (306, 3, 2217),
}


def verify_report(report: dict) -> list[str]:
    errors = []
    if report.get('schema') != 'havenwild.elizawy.external_tiled_concordance.v1' or report.get('archiveSha256') != KNOWN_SHA or report.get('matchesAuditedUpload') is not True:
        errors.append('Report schema or pinned derivative archive identity mismatch')
    if report.get('errors') or report.get('status') != 'external_derivative_evidence_only':
        errors.append('Report is not a clean external evidence report')
    if report.get('historicalMapState') != 'sha256_verified':
        errors.append('Historical Summer mapping hash was not verified in report')
    if report.get('certifiedMappingsAdded') != 0 or report.get('runtimeChanged') is not False or report.get('sourceFilesMutated') is not False:
        errors.append('External evidence report unexpectedly claims runtime changes or certified mappings')
    matches = report.get('pixelConcordance', {}).get('matches', {})
    if set(matches) != set(SEASONS):
        errors.append('Report does not include all four seasons')
    for season in SEASONS:
        item = matches.get(season, {})
        if item.get('state') != 'original_sha_verified_rgba32_exact_match':
            errors.append(f'{season}: original RGBA and source SHA verification missing'); continue
        value = tuple(item.get(key) for key in ('uniqueCandidateCells', 'ambiguousCandidateCells', 'unmatchedDerivativeOccupiedCells'))
        if value != EXPECTED[season] or item.get('certifiedRoles') != 0:
            errors.append(f'{season}: candidate counts/certification differ from audited ZIP')
        if item.get('derivativeMatchingCells') != value[0] + value[1]:
            errors.append(f'{season}: matching cell subtotal incorrect')
    comp = report.get('comparison', {})
    autumn = comp.get('autumn', {})
    if (autumn.get('Terrain', {}).get('missingCount'), autumn.get('Terrain', {}).get('extraCount'), autumn.get('Terrain', {}).get('changedCount'), autumn.get('Fences', {}).get('missingEntireSet')) != (58, 8, 10, True):
        errors.append('Autumn incomplete Tiled metadata review flag has changed')
    if comp.get('winter', {}).get('Terrain', {}).get('semanticNamesIdentical') is not False:
        errors.append('Winter material-name caveat missing')
    if report.get('buildingProfile', {}).get('policy') != 'architecture_reference_only_not_terrain_authority':
        errors.append('External building sheet must remain architecture-only reference')
    return errors


def main() -> int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=Path(__file__).resolve().parents[1])
    args=parser.parse_args()
    root=args.root.resolve()
    path=root/REPORT
    registry=root/DEFAULT
    if not path.is_file():
        if registry.is_file():
            print('ELIZAWY EXTERNAL EVIDENCE: FAIL / registry exists without its parent external reconciliation',file=sys.stderr)
            return 2
        print('ELIZAWY EXTERNAL EVIDENCE: SKIP / local Tiled ZIP report not present; external derivative NOT CERTIFIED')
        return 0
    try:
        report=json.loads(path.read_text(encoding='utf-8'))
        errors=verify_report(report)
        if errors:
            raise ValueError('; '.join(errors[:10]))
        print('ELIZAWY EXTERNAL REPORT: PASS / four-season SHA+RGBA evidence, 0 promotions; this is not a runtime recipe certification')
        if registry.is_file():
            reg=json.loads(registry.read_text(encoding='utf-8'))
            errors=validate_registry(reg,root)
            if errors:raise ValueError('Full registry invalid: '+'; '.join(errors[:10]))
            if reg.get('archiveSha256') != report.get('archiveSha256'):
                raise ValueError('Registry and external report describe different archives')
            digest=hashlib.sha256(registry.read_bytes()).hexdigest()
            print(f'ELIZAWY M2C4 FULL REGISTRY: PASS / all occupied cells reviewed for candidate count consistency / SHA-256 {digest[:16]}... / 0 promotions')
        else:
            print('ELIZAWY M2C4 FULL REGISTRY: SKIP / no full per-cell export yet; NOT CERTIFIED')
        return 0
    except (ValueError,OSError,KeyError,TypeError,json.JSONDecodeError) as exc:
        print('ELIZAWY EXTERNAL EVIDENCE: FAIL / '+str(exc),file=sys.stderr)
        return 2

if __name__=='__main__':
    raise SystemExit(main())
