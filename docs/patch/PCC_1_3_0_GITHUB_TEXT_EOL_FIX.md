# PCC 1.3.0 — GitHub text identity normalization

Fresh-source asset hydration remains GitHub-first at the pinned ElizaWy/LPC commit `f07f7f5892e67c932c68f70bb04472f2c64e46bc`.

The historical Havenwild core-source manifest was captured from a Windows Git working tree. Its 28 `Credits.txt` records therefore contain CRLF byte counts/hashes, while GitHub raw/blob endpoints return the repository's LF bytes. Binary assets were never affected.

PCC 1.3.0 / ForgePY 0.4.8 fixes this without weakening binary verification:

- binary PNG/GIF assets still require exact byte count and SHA-256 identity;
- `Credits.txt` files first try exact identity;
- if exact identity differs, only newline representation is canonicalized to CRLF and compared with the historical manifest;
- any content change beyond line endings still fails closed;
- the exact GitHub LF blob is written locally, preserving a true upstream mirror;
- full core-manifest verification uses the same compatibility rule so hydrated LF files remain certified.

The asset hydration self-test now certifies the GitHub-first ordering, binary fail-closed behavior, LF/CRLF compatibility, tamper rejection, and local fallback policy.
