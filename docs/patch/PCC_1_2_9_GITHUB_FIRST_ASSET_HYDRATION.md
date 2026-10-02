# PCC 1.2.9 — GitHub-first asset hydration

Fresh/source-only Havenwild deployments no longer require an existing local ElizaWy asset mirror before Full Gate can proceed.

## Authority order

1. Exact pinned upstream: `ElizaWy/LPC@f07f7f5892e67c932c68f70bb04472f2c64e46bc`.
2. Checked-in `core_source_manifest.json` byte-count + SHA-256 identity validation.
3. Remembered/local source trees only as fallback when pinned GitHub hydration cannot complete.
4. Explicit `--local-only` remains available as an operator recovery override.

## Gate behavior

If core assets are absent, Full Gate now invokes the normal asset sync lane, which attempts the pinned GitHub revision first. Already-correct local files are proven against the manifest and skipped without a network request; missing or invalid files are fetched and verified. Only if that path fails does ForgePY look for a remembered/local fallback source.

The Assets menu now makes this hierarchy explicit: normal hydrate/repair is GitHub-first, manual local fallback is separate, and forced pinned-upstream verification/repair remains available.
