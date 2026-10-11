# Havenwild Bevy 0.10.48 — M2D130 Rust 2021 native build repair

This release preserves M2D128 Studio's Pixel Studio, Game Tools, Scene Navigator,
Inspector Asset Browser, canvas Layers and animation preview timeline, and the
M2D129 gate-test repairs. The source is standalone Havenwild.

## Windows instructions

1. For a current 0.10.47 checkout, put the M2D130 .pccpatch.zip at the project root
   and approve it in `PCC.cmd` (keep authored saves and hydrated assets).
2. Alternatively extract the M2D130 complete-source ZIP into a separate clean folder.
3. Confirm **0.10.48**, hydrate original core assets if needed, and run **Full Quality Gate**.
4. If green, run Vulkan Studio; verify dragging/clicking the Pixel Studio color wheel,
   shared Inspector Library and timeline playback in the generated world.

The repair removes Rust 2024-only let-chain syntax from 2021-edition code.
No asset mapping, world generation, scene data or PIE logic was altered.
Native Windows compilation is not verified by the source-only package audit.
