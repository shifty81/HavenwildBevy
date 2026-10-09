# M2D100 complete Havenwild source rollup — four volumes

The complete source and user-supplied 299,252,779-byte LPC character archive could not be delivered as a single attachment, so M2D100 is distributed as four independent numbered ZIPs. **Download and extract all four** into the same destination so they merge into the single directory `Havenwild_Bevy_M2D100_Full_Source_0.10.21/`. Do not drop these full source volumes into the PCC update inbox; they are not `.pccpatch` updates.

1. Extract `Havenwild_Bevy_M2D100_Full_Source_0.10.21_PART_1_of_4.zip` into a clean directory. This contains all source, the M2D099 changes already applied, and first character archive volume.
2. Extract `_PART_2_of_4.zip`, `_PART_3_of_4.zip`, and `_PART_4_of_4.zip` over the same destination, merging the top-level project folder. They contain only numbered character archive byte pieces.
3. Run `ASSEMBLE_CHARACTER_LIBRARY.cmd` at the project root. It requires all four parts. It verifies each volume's SHA-256 and the entire reconstructed `Characters.zip` against the original pinned core manifest before atomic installation. It refuses to overwrite an existing mismatched character archive. Parts are deliberately preserved for recovery.
4. Run `PCC.cmd assets hydrate-characters`, then `PCC.cmd audit` and `PCC.cmd full`. This unpacks the now-restored character artwork into its normal local library and validates builds. Studio/Vulkan gameplay inspection remains required.

A `Characters.zip.partXX` is **not** an ordinary ZIP and should not be opened separately. The source-integrity manifest `docs/rollup/M2D100_SOURCE_FILE_SHA256.json` references the reconstructed `assets/optional/Characters.zip`; it becomes complete after assembly. If you want to recover disk space after assembly and hydration succeed, the four `.partXX` files can be removed manually, but no cleanup is automatic.
