# Havenwild M2D100 — complete working-source rollup

Baseline: user-uploaded `HWlatest.zip` (Havenwild `0.10.19` source snapshot).

- M2D099 eight-file source patch integrated directly, with manifest SHA-256 validation. The M2D099 base becomes `0.10.20` and this character-authority rollup is `0.10.21`; the earlier patch should not be applied again.
- 28 `assets/elizawy/**/Credits.txt` files restored from LF to the exact original CRLF (same pinned SHA-256 and size), with local backup removed from the distributed bundle after validation. Never rewrite these credits as normal LF text.
- Added `.gitattributes` override for immutable LPC credit files so a future Git checkout cannot change their bytes.
- The submitted canonical `Characters.zip` is now part of the rollup at `assets/optional/Characters.zip` (299,252,779 bytes; SHA-256 `0de481a33c458d600fb3adaed043dcd953f58dec436dac0ca241c7dc5a0368f9`); its 63,991 original source entries and credit records remain unmodified.
- The existing character library hydrator must verify the whole source archive before use. An explicit `--verify` checks each extracted member's CRC32, including when its size is unchanged. Member writes are atomic.
- The complete compressed archive is retained as provenance rather than duplicating all 63,991 individual files inside the handoff. Extract/hydrate through `PCC.cmd assets hydrate-characters` after unpacking.
- The original mixed-project packaging was superseded by the M2D118 standalone source boundary; the unrelated files are excluded.
- Original M2D099 patch is archived under `updates/archive/` for provenance, **not** in the pending root or inbox.

Verification performed separately before distribution includes pinned archive source SHA-256, core source image/credit SHA-256, Python regression suite, exact character archive extraction/CRC recheck, asset generation source check, and package-integrity audit. These results do not constitute a new Windows Cargo Full Gate or visual renderer acceptance.

Source distribution omits `.forgepy/` machine-specific logs, recovered credit backups, and hydrated `assets/elizawy/Characters/` duplicates; PCC reconstructs the character tree from the pinned included ZIP. Existing machine-specific state in a different working checkout is not overwritten by this recovery bundle.

Delivery note: the full source rollup is distributed in four numbered ZIP volumes because the single artifact was too large for reliable delivery. See `docs/rollup/M2D100_SPLIT_ARCHIVE_INSTALL.md` and run the included `ASSEMBLE_CHARACTER_LIBRARY.cmd` before hydration.
