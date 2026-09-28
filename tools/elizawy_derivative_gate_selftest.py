#!/usr/bin/env python3
"""M2C4 gate checks: report flags and non-authoritative provenance are fail-closed."""
from __future__ import annotations
import json
from copy import deepcopy
from pathlib import Path
import tempfile
from verify_elizawy_derivative_evidence import verify_report, EXPECTED
from reconcile_elizawy_tiled import KNOWN_SHA


def main() -> int:
    common={s:{'state':'original_sha_verified_rgba32_exact_match','uniqueCandidateCells':x[0],'ambiguousCandidateCells':x[1], 'unmatchedDerivativeOccupiedCells':x[2], 'derivativeMatchingCells':x[0]+x[1], 'certifiedRoles':0} for s,x in EXPECTED.items()}
    base={'schema':'havenwild.elizawy.external_tiled_concordance.v1','status':'external_derivative_evidence_only','archiveSha256':KNOWN_SHA,'matchesAuditedUpload':True,'errors':[],'historicalMapState':'sha256_verified','certifiedMappingsAdded':0,'runtimeChanged':False,'sourceFilesMutated':False,'pixelConcordance':{'matches':common},'buildingProfile':{'policy':'architecture_reference_only_not_terrain_authority'},'comparison':{'autumn':{'Terrain':{'missingCount':58,'extraCount':8,'changedCount':10},'Fences':{'missingEntireSet':True}},'winter':{'Terrain':{'semanticNamesIdentical':False}}}}
    assert not verify_report(base)
    for label, key, value in [('archive','archiveSha256','0'*64),('mutation','runtimeChanged',True),('promotion','certifiedMappingsAdded',1)]:
        d=deepcopy(base);d[key]=value;assert verify_report(d),label
    d=deepcopy(base);d['pixelConcordance']['matches']['winter']['state']='not_requested';assert verify_report(d)
    d=deepcopy(base);d['comparison']['autumn']['Fences']['missingEntireSet']=False;assert verify_report(d)
    print('M2C4 DERIVATIVE GATE SELFTEST: PASS / incomplete provenance, promotions, legacy Autumn and Winter semantics rejected')
    return 0
if __name__=='__main__':
    raise SystemExit(main())
