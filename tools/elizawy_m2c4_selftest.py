#!/usr/bin/env python3
"""M2C4 no-network/no-Cargo safety and structural review-registry tests."""
from __future__ import annotations
from copy import deepcopy
from export_elizawy_review_registry import SCHEMA, SEASONS, STATUS, validate_registry


def main() -> int:
    def fixture():
        seasons = {}
        for name in SEASONS:
            cells = []
            for tid in range(2526):
                cells.append({
                    'derivativeTileId': tid, 'derivativeCell': [tid % 64, tid // 64],
                    'matchStatus': STATUS[2], 'originalCandidates': [],
                    'reviewState': 'unreviewed', 'mappingCertified': False,
                })
            seasons[name] = {
                'originalSourceSha256': 'a' * 64,
                'originalPixelState': 'original_sha_verified_rgba32_exact_match',
                'summary': {'occupiedDerivativeCells': 2526, 'uniqueCandidateCells': 0,
                            'ambiguousCandidateCells': 0, 'unmatchedDerivativeCells': 2526},
                'cells': cells,
            }
        return {'schema': SCHEMA, 'authority': 'evidence_only_zero_promotions',
                'archiveSha256': __import__('reconcile_elizawy_tiled').KNOWN_SHA,
                'historicalMapState': 'sha256_verified', 'seasons': seasons,
                'certifiedMappingsAdded': 0, 'runtimeChanged': False, 'sourceFilesMutated': False}
    good = fixture()
    assert not validate_registry(good)
    cases = [
        ('certification', lambda d: d.update(certifiedMappingsAdded=1)),
        ('role promoted', lambda d: d['seasons']['summer']['cells'][0].update(mappingCertified=True)),
        ('tile id corruption', lambda d: d['seasons']['spring']['cells'][0].update(derivativeTileId=4096)),
        ('wrong count', lambda d: d['seasons']['winter']['summary'].update(unmatchedDerivativeCells=999)),
        ('missing season', lambda d: d['seasons'].pop('autumn')),
        ('wrong hash', lambda d: d.update(archiveSha256='0'*64)),
        ('false unique', lambda d: d['seasons']['summer']['cells'][0].update(matchStatus=STATUS[0])),
    ]
    for label, mutate in cases:
        doc = deepcopy(good)
        mutate(doc)
        assert validate_registry(doc), f'{label} was accepted'
    print('M2C4 SELFTEST: PASS / complete row identity, summaries, seasonal scope, reject unsafe mutations/promotions')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
