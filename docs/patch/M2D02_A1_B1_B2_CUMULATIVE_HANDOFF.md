# Havenwild Bevy — M2D02 A1+B1+B2 cumulative root-drop PCC handoff

## Supported starting point
- M2D02-A (Lean UI) must already be installed. Includes its M2D02-A1 keyboard API repair, M2D02-B1 scene-v2 draft foundation, and M2D02-B2 translucent canvas rulers in **one ZIP**.
- This also works when A1 was already applied. Do not additionally apply separate A1, B1 or B2 archives after this cumulative patch.
- This update leaves the original ElizaWy art, workstation M2C4 registry, existing authored scene v1, recipe authority, Cargo.lock and saved user layout untouched. It only changes the listed source/tool/docs paths.
- PCC may identify the source-version triplet as unchanged (0.5.6/0.4.6/1.2.6): review the unique cumulative patch ID in the update inbox.

## Install
1. Remove older **pending** A1/B1/B2 `.pccpatch.zip` files from the project root and `updates/inbox`, or keep them elsewhere. Do not manually extract patch ZIPs.
2. Drop `Havenwild_Bevy_M2D02_A1_B1_B2_Cumulative_From_A.pccpatch.zip` into your Havenwild Bevy root.
3. PCC > 11 > 1 verify **one** cumulative patch is READY; PCC > 11 > 2 apply. PCC > 1 Full Quality Gate, then PCC > 3 DX12 run.
4. Check default Select tool, ruler bands overlay within canvas, View toggle/F11, Scene > Preview separate v2 scene draft and optional explicit draft creation.

## Truthful limits
- B1's v2 draft is independently previewed/saved and **not** activated as the live scene renderer yet. The v1 scene stays original.
- Regeneration, v2 live canvas, real layered ownership, whole-object editing and whole-scene Pixel mode are future phases. No source content is auto-certified.
- Local Python audits and PCC dry-runs do not establish Windows Cargo formatting, compilation, native UI or visual output. The Windows Full Gate is required.
