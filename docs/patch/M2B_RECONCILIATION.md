# M2B — historical mapping reconciliation (diagnostic-only)

Run `python tools/reconcile_mapping_evidence.py` after the existing
`PCC.cmd terrain recover --source "C:\\path\\to\\older\\Havenwild"` workflow.
It reads the M2A inventory and local recovery index and writes
`.forgepy/recovered_mapping/reconciliation_report.v1.json`.
Raw copied evidence is SHA-256 checked; external paths are NOT read or trusted.
Filename hints are candidates only, never semantic imports or certification.
No existing source, scene, recipe or UI is overwritten. No raw evidence is packaged.
