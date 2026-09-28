# Havenwild Bevy PCC update inbox

Future incremental source patches may be dropped unextracted into either:

- `updates/inbox/*.zip`, or
- the project root as `*.pccpatch.zip`.

PCC only applies a ZIP when it contains a root `pcc_patch.json` with schema
`havenwild.bevy.pcc.patch.v1`. Every file must be declared with its SHA-256.
PCC stages and validates the patch, backs up replaced/deleted source files,
applies replacements atomically, archives the consumed ZIP, and records a receipt.

Patches are source-only. PCC rejects destinations under `assets/`, `reference/`,
`.git/`, `.pcc/`, `.forgepy/`, `target/`, and `artifacts/`, plus generated local
asset catalogs.
