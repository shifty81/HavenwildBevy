# M2D096A — Tiled live renderer cutover / source 0.10.16

M2D096A promotes the previously certified Jaidyn/Tiled terrain authority from an isolated parser/test lane into Havenwild's live core-terrain resolution path without promoting derivative artwork to art authority.

## Live path

The following consumers now share the same optional `CanonicalTiledTerrainAuthority`:

- Paint/sample source resolution;
- near dual-grid terrain rendering;
- Generated World source materialization used by far/source-backed rendering;
- World Generator preview and regenerate operations.

The configured Jaidyn Summer TSX supplies Wang material names, exact authored Wang assignments, and animation metadata. A configured derivative tile may render only when its 32x32 RGBA pixels match an exact tile in Havenwild's canonical ElizaWy Terrain source catalog. Missing concordance fails closed to the prior certified source-backed resolver.

## First promoted semantics

Evidence-backed Havenwild roles map as follows:

- `Grass` -> `Grass`
- `MudBank` -> `Dirt`
- `Sand` -> `Sand`
- `RiverWater` -> `Shallow Water`
- `DeepWater` -> `Deep Water`

`WetSand` and `PebblePath` are deliberately not translated. They remain on the previous resolver until configured TSX and original ElizaWy scene/demo evidence establish their exact identity.

## Animation readiness

`TiledTileMetadata` now retains exact animation frame tile IDs and durations rather than only a frame count. Canonical concordance also measures how many referenced animation frames map back to original ElizaWy pixels. Runtime animation presentation remains disabled in M2D096A; this pass only preserves the authored data needed for the next shoreline/water animation pass.

## Generated World migration

`WORLD_GENERATOR_ID` advances to `havenwild.source_backed.generated_world.v11`. Existing generated chunk source bindings from v9 are therefore rematerialized through the new authority while authored world overrides remain governed by the existing persistence rules.

## Hydration and failure policy

Full Quality Gate now verifies the pinned configured Jaidyn package and hydrates it when missing. Studio startup loads the bridge opportunistically. If configured assets cannot be loaded or a source tile is not canonically matched, Havenwild logs the condition and retains the prior source-backed resolver instead of crashing or drawing unverified derivative pixels.

## Renderer/backend policy

Vulkan remains the recommended normal resizable Studio backend. DX12 remains available through the guarded fixed-size/diagnostic paths introduced in M2D095D.

## Native validation boundary

Static/source-side milestone tests certify the integration wiring, canonical-art guard, semantic fail-closed rules, and native Wang stress behavior. Windows Full Quality Gate remains the authority for Rust compilation, all-target tests, and live interactive visual acceptance.
