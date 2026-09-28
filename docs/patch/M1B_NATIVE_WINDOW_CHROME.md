# M1B: Native Windows chrome enforcement

Follow-up to M1 after screenshot showed a second, inert ForgeGUI title bar.

- Force `decorations: true` independent of `STUDIO_NATIVE_FRAME`; old environment settings cannot silently restore broken custom chrome.
- Remove the custom title-bar panel and its non-operational snap handler from Studio's client area.
- Retain the earlier canvas input-unblocking fix, editor and mapping data.
- This is source-only: FULL GATE and interactive Windows drag/minimize/maximize/close still require local certification.

Apply through the governed PCC inbox; restart Studio after applying and building. If the native OS title bar still does not appear, capture the Studio launch log and inspect the actual executable/launcher used.
