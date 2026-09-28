#!/usr/bin/env python3
"""Read-only reconciliation report: historical artifact evidence vs current inventory."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = Path("content/catalog/mapping_inventory.v1.json")
INDEX = Path(".forgepy/recovered_mapping/index.json")
OUTPUT = Path(".forgepy/recovered_mapping/reconciliation_report.v1.json")
HINTS = {
    "pass103_summer_complete_map": ("lpc_terrain_summer_complete_map_32.json",),
    "pass106_seasonal_topology": ("lpc_seasonal_terrain_topology_v0_1.json",),
    "pass114_exact_tuples_snapshot": ("lpc_mapped_terrain_v7_32.json",),
    "b48r15_summer_mapper": ("summer", "crosswalk"),
}
def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda:f.read(1024*1024),b""): h.update(block)
    return h.hexdigest()
def report(root: Path) -> dict:
    inventory = json.loads((root/INVENTORY).read_text(encoding="utf-8"))
    if inventory.get("schema") != "havenwild.asset_mapping.inventory.v1":
        raise ValueError("unsupported inventory schema")
    index_path = root/INDEX
    records = []
    if index_path.is_file():
        idx = json.loads(index_path.read_text(encoding="utf-8"))
        if idx.get("schema") != "havenwild.terrain.legacy_mapping_recovery.local.v1":
            raise ValueError("unsupported recovery index schema")
        records = idx["records"]
    checked=[]
    for rec in records:
        rel = str(rec.get("relativePath",""))
        local = rec.get("localPath")
        state="indexed_external_unverified"
        if local:
            p = (root/local).resolve()
            base = (root/".forgepy/recovered_mapping/raw").resolve()
            if not p.is_relative_to(base):
                state="unsafe_local_path"
            elif not p.is_file():
                state="local_copy_missing"
            elif digest(p)!=rec.get("sha256"):
                state="local_copy_hash_mismatch"
            else:
                state="local_copy_hash_verified"
        checked.append({"relativePath":rel,"sha256":rec.get("sha256"),"evidenceState":state,
                        "possibleHistoricalMatches":[h["id"] for h in inventory["historicalEvidence"]
                        if any(x in rel.lower() for x in HINTS.get(h["id"],()))]})
    return {"schema":"havenwild.mapping.reconciliation_report.v1",
      "authority":"diagnostic_only_no_semantic_import_or_promotion",
      "inventorySource":str(INVENTORY),"recoveryIndex":str(INDEX),
      "recoveryIndexPresent":index_path.is_file(),"records":checked,
      "historicalEvidence":[{"id":h["id"],"currentImportState":h["currentImportState"],
        "candidateArtifacts":sum(h["id"] in r["possibleHistoricalMatches"] for r in checked),
        "status":"candidate_evidence_only" if any(h["id"] in r["possibleHistoricalMatches"] for r in checked)
        else "raw_artifact_not_identified"} for h in inventory["historicalEvidence"]],
      "certifiedMappingsAdded":0}
def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--root",type=Path,default=ROOT)
    ap.add_argument("--output",type=Path)
    a=ap.parse_args()
    try:
        result=report(a.root)
        out=a.output or a.root/OUTPUT
        out.parent.mkdir(parents=True,exist_ok=True)
        out.write_text(json.dumps(result,indent=2)+"\n",encoding="utf-8")
        print(f"RECONCILIATION: PASS / {len(result['records'])} indexed artifacts / {len(result['historicalEvidence'])} historical evidence records / 0 promoted")
        print(f"REPORT: {out}")
        return 0
    except (OSError,ValueError,KeyError,TypeError) as exc:
        print(f"RECONCILIATION: FAIL / {exc}")
        return 2
if __name__=="__main__": raise SystemExit(main())
