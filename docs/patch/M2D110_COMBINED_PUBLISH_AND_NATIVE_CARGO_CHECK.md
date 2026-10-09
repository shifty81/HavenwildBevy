# Havenwild Bevy M2D110 — combined Git preflight and E0308 native-build repair

**Upgrade:** installed source `0.10.29` → `0.10.30` (PCC `1.3.5`, ForgePY `0.4.10` unchanged).

This archive **supersedes** the separately issued `Havenwild_Bevy_M2D109_Git_Publish_Preflight_Fix_From_0.10.29.pccpatch.zip`. Install only the combined patch. Do not queue both archives simultaneously.

## Native Full Gate failure confirmed

The user's Full Gate at source `0.10.29` stopped in `cargo check` with Rust E0308 at `src/main.rs`:

```
expected `&mut Image`, found `AssetMut<'_, Image>`
```

`Assets<Image>::get_mut` returns Bevy's change-tracked `AssetMut<Image>` guard, rather than a direct `&mut Image`. The source now binds that guard as mutable (`let Some(mut image)`) and passes `&mut image` to the premultiplication function. Rust's mutable deref coercion supplies the expected `&mut Image`, preserving asset change tracking, alpha conversion, and original source-art immutability. A dedicated pre-Cargo source selftest guards against this exact regression.

## Included M2D109 Git preflight changes

- Scope `.gitignore` to exclude local character hydration, `target/`, `.pcc/`, `.forgepy/`, generated packages and artifacts while keeping project-owned source paths available.
- Print phases **before** the first potentially expensive publish-readiness Git query; print eight-second progress heartbeat and completion time.
- Keep Git publish certification, changed-file hashing, binary diff, `origin/main` ahead check, path authorization, and explicit approval unchanged.
- Continue to run M2D109 Git validation regression tests in PCC Full Gate and static audit.

## Installation and verification

1. Remove the old M2D109 `.pccpatch.zip` from the project root/update inbox if still queued. Do not delete working-tree files or local Git history.
2. Place **only** this combined M2D110 `.pccpatch.zip` at the `HWlatest` project root and approve through `PCC.cmd`.
3. Verify `Source : 0.10.30`; run Full Quality Gate again. Note: rustfmt may normalize source formatting automatically, and Rust build/test still need Windows native verification.
4. After a Full Gate PASS, use menu 2 and observe preflight activity; no GitHub push occurs until explicit authorization.

### Validation scope

Package integrity, PCC manifest compatibility, simulated apply and source-level checks are verified outside Windows. **Cargo check, the native build, and the live GitHub publish path have not been run in the portable environment.**
