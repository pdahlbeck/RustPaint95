# RustPaint95

A clean-room reimplementation of **Microsoft Paint as shipped with Windows 95**, written in Rust.

## Non-negotiable goal

Historical fidelity, not a modern drawing application with a retro skin. Match the original Windows 95 MS Paint user interface, drawing behavior, keyboard shortcuts, mouse semantics, dialogs, file-format restrictions, and documented quirks. When a detail is uncertain, verify it against an original Windows 95 installation before implementing it.

**Not affiliated with Microsoft.** The implementation must use original source code and independently produced graphical assets; do not commit Microsoft binaries, icons, bitmaps, fonts, or other copyrighted assets.

## Strict compatibility rules

- The application has one bitmap canvas, **no layers**.
- **Three levels of Undo**, no unlimited history.
- Historically accurate selection behavior, text, geometric primitives, paintbrushes, fill, eraser, magnifier, airbrush, curves, foreground/background colors and right-click semantics.
- BMP workflows and color depths matching the original; support Paint's historic PAL color-palette files where applicable.
- No added modern file formats or enhancements in the user-facing application. In particular **no native PNG export**.
- Do not silently 'fix' historic behavior, even if it seems unintuitive.
- Rendering is pixel-accurate with no invented antialiasing.
- Menu structure, toolbar, palette, bitmap workspace, status bar, dialogs, system colors, sizing and focus behavior are to be compared against an original installation.

## Proposed architecture

- `paint_core`: deterministic pixel drawing, tools, selection, undo (cap = 3), image operations, BMP and PAL formats.
- `paint_ui`: Windows 95 look and behavior via custom pixel-aligned widgets and a platform-neutral window/event layer.
- `platform`: filesystem, clipboard, pointer and keyboard translation, printing integration where available.
- `tests`: golden pixel outputs and behavioral comparison against reference sessions.

Rust is the implementation language. Platform APIs or dependencies may be used for displaying the window and opening native files, but never to change historically visible behavior.

## Reference and verification protocol

1. Establish a specific Windows 95 Paint build as ground truth, including its release/version, OS language, display DPI and color depth.
2. Capture screenshots of every menu, dialog and tool-options state in a Windows 95 virtual machine.
3. Record tool behavior with left and right mouse buttons, unusual input cases, clipping, coordinate handling, keyboard shortcuts, zoom levels, palette changes, file import/export and undo.
4. Build reproducible reference BMPs and scripted pointer actions.
5. Compare screenshots **pixel by pixel** and generated BMP bytes or decoded pixels, documenting accepted platform differences.

Do not confuse the Windows 95 program with Windows 98, XP or browser reimplementations.

## Planned milestones

- [ ] M0: version-pinned historical reference corpus and compatibility test specification.
- [ ] M1: classic UI shell, window controls and menu structure.
- [ ] M2: canvas plus the 16 original tools and mouse semantics.
- [ ] M3: selections, text, image operations, zoom and three-level undo.
- [ ] M4: correct BMP and PAL behavior, clipboard, dialogs and printing.
- [ ] M5: visual/behavioral regression suite on Windows, macOS and Linux.

## Current status

**Specification only.** This repository is not yet an implemented Paint clone. Faithfulness is a testable target, not a claim of completed equivalence.
