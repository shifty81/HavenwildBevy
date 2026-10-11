# M2D132B - M2D114 Windows rustfmt gate repair

Source 0.10.58 -> 0.10.59.

The M2D114 static contract demanded exact single-line strings for Bridge, Panel, Select and Browse Objects UI calls. Windows Full Gate runs rustfmt which formats those calls across lines, despite retaining identical semantics. The assertions now use whitespace-tolerant regular expressions and still require the same concrete rail actions.

Evidence: debug bundle 20261010-185204-710727, phase M2D114. The bundled Windows-formatted main.rs was used locally to reproduce the failure. After the correction the M2D114 test passes against that same source.

No game/editor behavior, hybrid proof geometry or original ElizaWy assets changed. Windows Full Gate must be rerun. No claim of cargo compilation or Vulkan runtime certification.
