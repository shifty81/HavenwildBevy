# M2D095D - DX12 surface guard / source 0.10.13

Source 0.10.13 follows a fully GREEN 0.10.12 Full Gate. The observed live failure occurred only after the DX12 Studio window was resized/reconfigured: `ResizeBuffers` returned `0x887A0001` and WGPU reported `surface configuration failed: window is in use`.

This pass does not alter the Jaidyn/Tiled terrain authority. It changes Windows run policy only:

- normal Studio run: Vulkan, resizable;
- DX12 safe run: native frame, fixed-size window;
- DX12 resize remains explicit diagnostic-only;
- verbose GPU modes remain available;
- Terrain startup hydration logging now counts only `Terrain/` sheets.

The source keeps Bevy 0.19.0 and tiled 0.16.0.
