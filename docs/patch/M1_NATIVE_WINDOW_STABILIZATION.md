# M1 Windows host stabilization — checkpoint patch

This is the first independently testable component of the planned cumulative recovery. It does not implement the universal mapping registry or regenerate/publish flow.

- Native OS window decorations are ON by default. The previous opt-in `STUDIO_NATIVE_FRAME=1` remains compatible.
- `STUDIO_NATIVE_FRAME=0` explicitly opts into the existing experimental custom chrome. This mode is not yet certified for move/resize and should not be used for normal authoring.
- The earlier canvas input-unblock code is retained. No full-screen foreground resize area is reintroduced.
- The existing Summer scene, mapping, source and hydrated assets are untouched.

## Windows checkpoint

1. Apply via PCC governed update inbox, preserving a backup.
2. Run FULL GATE. The earlier DX12 `wgpu-hal` / `gpu-allocator` Windows-binding failure is **not** claimed fixed by this patch; attach the new gate evidence if it persists.
3. Launch Studio. Check native move, minimize, maximize, restore, edge resize and Windows Snap.
4. Verify toolbar/canvas/dock input, drag/drop, erase, Ctrl+S and reopen.

Do not modify registry cargo source or manually rewrite Cargo.lock.
