# M2D095E — Regression contract repair / source 0.10.14

The 0.10.13 Windows Full Gate failed in M2D080 because its source-text assertion required the pre-M2D095D one-argument Studio title format. The runtime title remained valid and package-versioned; M2D095D intentionally added an optional DX12-safe suffix.

This pass:
- makes the M2D080 title check semantic and rustfmt-tolerant;
- changes historical milestone version checks from exact-current-version to milestone-or-newer;
- requires project `sourceVersion` to match Cargo where both are available;
- preserves the 0.10.13 Vulkan-first / fixed-size DX12-safe runtime policy;
- preserves `tiled 0.16`, Jaidyn authority, and the Havenwild-native sparse Wang solver unchanged.

No asset coordinates, terrain mappings, worldgen behavior, renderer topology, or gameplay semantics are changed.
