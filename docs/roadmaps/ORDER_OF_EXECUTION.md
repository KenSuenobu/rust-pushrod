# Order of Execution

The complete plan of action for Pushrod, Chassis, and `pushrod-extra-widgets`: **every open issue in the exact order to do it**, one step at a time, from the first Pushrod commit to Chassis v2 and the extra widgets. Each step is one piece of work, one pull request, and one video.

**268 steps:** 256 issues and 12 checkpoints (releases and phase demos). The order is generated from the roadmaps and checked: every step comes after everything it depends on.

## How to use this document

- **Work top to bottom.** Don't skip ahead: later steps assume earlier ones are merged.
- **One step, one pull request, one video.** Each step lists what it delivers, which earlier steps it builds on, and an on-camera checklist taken from the issue's acceptance criteria. Show each item working in the video, and the step is done.
- **Done steps** keep their number (marked *done*), so the series numbering never shifts; their videos can be recaps.
- **Checkpoints** are episodes of their own: a release or the end of a phase, demonstrating everything built so far.
- **Every step ships its documentation** (Docusaurus pages, full doc headers, changelog entry), as CONTRIBUTING.md requires.
- **Suggested video title:** `Pushrod #<step>: <title>` (or `Chassis #<step>: ...`, `Extra Widgets #<step>: ...`), so the series sorts in order on YouTube.

Sources: [Pushrod roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md), [Chassis roadmap](../ROADMAP_CHASSIS.md) (Action Plan), [extra widgets roadmap](../ROADMAP_EXTRA_WIDGETS.md), [GitHub projects](../PROJECTS.md). When a roadmap changes, regenerate this file.

## Overview

| Part | Steps | Result |
|---|---|---|
| [Part 1: Pushrod v0.1, dialog applications](#part-1-pushrod-v01-dialog-applications) | 1 to 56 (56) | Everything needed to build and ship real dialog-style applications with Pushrod, ending with the first crates.io release. |
| [Part 2: Pushrod v0.2, forms and resources](#part-2-pushrod-v02-forms-and-resources) | 57 to 70 (14) | Forms, form files, and resources by ID (the runtime foundation for Chassis), plus input methods, mnemonics, accessibility, and specs. |
| [Part 3: Pushrod Showcase, the interactive demo of every widget](#part-3-pushrod-showcase-the-interactive-demo-of-every-widget) | 71 to 78 (8) | One application that shows every widget and core feature live, with a property panel, an event log, and source code. |
| [Part 4: Chassis M1: core without UI](#part-4-chassis-m1-core-without-ui) | 79 to 84 (6) | Project model, resource access, code generation, and a command-line generator, all testable without a window. |
| [Part 5: Chassis M2: shell and project](#part-5-chassis-m2-shell-and-project) | 85 to 115 (31) | The main window, three panes, Designer/Code tabs, commands, and the project drawer with resource editors. |
| [Part 6: Chassis M3: form designer](#part-6-chassis-m3-form-designer) | 116 to 132 (17) | Design surface, palette, drag and drop, selection, snapping, containers, layouts, and undo. |
| [Part 7: Chassis M4: inspector, events, and code](#part-7-chassis-m4-inspector-events-and-code) | 133 to 155 (23) | Property inspector, events menu and indicators, handler creation, and the code view. |
| [Part 8: Chassis M5: build, run, and release (MVP)](#part-8-chassis-m5-build-run-and-release-mvp) | 156 to 161 (6) | Build and run from Chassis, CI, documentation, and the sample project. |
| [Part 9: Chassis example projects](#part-9-chassis-example-projects) | 162 to 170 (9) | Example projects built with Chassis once it reaches the MVP: small, complete applications that exercise its features and that anyone can build and test against. |
| [Part 10: Chassis v2](#part-10-chassis-v2) | 171 to 206 (36) | The second release: settings, menu editor, form preview, multi-selection, error list, templates, packaging, code intelligence, localization, form inheritance, bindings, project search, multiple windows, accessibility checks, source control, and more build targets, and an AI assistant through MCP. |
| [Part 11: Extra widgets 1 and 2 (the pushrod-extra-widgets MVP)](#part-11-extra-widgets-1-and-2-the-pushrod-extra-widgets-mvp) | 207 to 233 (27) | The rest of the `pushrod-extra-widgets` crate's first two milestones: the popover and gallery, then everyday controls, pickers, lists, panels, and dialogs. |
| [Part 12: Extra widgets 3, specialized views and assistants](#part-12-extra-widgets-3-specialized-views-and-assistants) | 234 to 252 (19) | Specialized views and assistants in `pushrod-extra-widgets`. |
| [Part 13: Remaining Pushrod backlog](#part-13-remaining-pushrod-backlog) | 253 to 268 (16) | Pushrod issues not needed by Chassis, in roadmap order. |

## Step index

| Step | Issue | Title |
|---|---|---|
| ✅ 1 | [#130](https://github.com/KenSuenobu/rust-pushrod/issues/130) | pushrod: [1.21] Split the standard widgets into the pushrod-widgets crate (done) |
| ✅ 2 | [#69](https://github.com/KenSuenobu/rust-pushrod/issues/69) | pushrod: [7.1] CI: build, test, clippy, fmt, doc with SDL2 (done) |
| ✅ 3 | [#122](https://github.com/KenSuenobu/rust-pushrod/issues/122) | pushrod: [7.8] Documentation site: build pushrod-docs in CI and publish it (done) |
| ✅ 4 | [#93](https://github.com/KenSuenobu/rust-pushrod/issues/93) | pushrod: [9.1] Event vocabulary: modern event names and structure (done) |
| ✅ 5 | [#106](https://github.com/KenSuenobu/rust-pushrod/issues/106) | pushrod: [10.2] Backend-neutral paint API (SDL2 backend by default) (done) |
| ✅ 6 | [#19](https://github.com/KenSuenobu/rust-pushrod/issues/19) | pushrod: [1.1] Widget behavior flags and visual states (done) |
| ✅ 7 | [#6](https://github.com/KenSuenobu/rust-pushrod/issues/6) | pushrod: [1.2] Event masks for the run loop and for widgets (done) |
| ✅ 8 | [#109](https://github.com/KenSuenobu/rust-pushrod/issues/109) | pushrod: [9.10] Engine events: every internal action Pushrod performs emits an event (done) |
| ✅ 9 | [#124](https://github.com/KenSuenobu/rust-pushrod/issues/124) | pushrod: [9.11] Event-loop style: keep parity with callbacks (done) |
| ✅ 10 | [#7](https://github.com/KenSuenobu/rust-pushrod/issues/7) | pushrod: [1.3] Broadcast events to all interested widgets (done) |
| ✅ 11 | [#22](https://github.com/KenSuenobu/rust-pushrod/issues/22) | pushrod: [1.6] Callback registry for every widget event (done) |
| ✅ 12 | [#78](https://github.com/KenSuenobu/rust-pushrod/issues/78) | pushrod: [1.13] Custom application events with payloads (done) |
| ✅ 13 | [#111](https://github.com/KenSuenobu/rust-pushrod/issues/111) | pushrod: [1.18] Text layout engine: Parley for shaping, bidi, line breaking, and font fallback (with an SDL2 compatibility gate) (done) |
| ✅ 14 | [#3](https://github.com/KenSuenobu/rust-pushrod/issues/3) | pushrod: [1.7] GPU text: font cache with a glyph atlas texture (done) |
| ✅ 15 | [#71](https://github.com/KenSuenobu/rust-pushrod/issues/71) | pushrod: [7.3] Widget gallery example (done) |
| ✅ 16 | [#94](https://github.com/KenSuenobu/rust-pushrod/issues/94) | pushrod: [9.2] Event propagation: capture, target, and bubble phases with handled/ignored (done) |
| ✅ 17 | [#96](https://github.com/KenSuenobu/rust-pushrod/issues/96) | pushrod: [9.4] Reusable input controllers: click, drag, scroll, hover, long-press, key (done) |
| ✅ 18 | [#20](https://github.com/KenSuenobu/rust-pushrod/issues/20) | pushrod: [1.4] Keyboard focus, Tab order, and Return/Escape (done) |
| ✅ 19 | [#104](https://github.com/KenSuenobu/rust-pushrod/issues/104) | pushrod: [1.17] Per-state appearance: colors, border, and font for each visual state (done) |
| ✅ 20 | [#289](https://github.com/KenSuenobu/rust-pushrod/issues/289) | pushrod: [1.26] Text truncation: shorten text that doesn't fit with an ellipsis (done) |
| ✅ 21 | [#290](https://github.com/KenSuenobu/rust-pushrod/issues/290) | pushrod: [1.27] Wrapped text: multi-line labels that reflow (done) |
| ✅ 22 | [#21](https://github.com/KenSuenobu/rust-pushrod/issues/21) | pushrod: [1.5] Timers and idle callbacks (done) |
| ✅ 23 | [#134](https://github.com/KenSuenobu/rust-pushrod/issues/134) | pushrod: [10.6] Backend crates: keep SDL out of the core and the widgets (done) |
| ✅ 24 | [#138](https://github.com/KenSuenobu/rust-pushrod/issues/138) | pushrod: [1.25] Errors: return Result instead of panicking (done) |
| ✅ 25 | [#137](https://github.com/KenSuenobu/rust-pushrod/issues/137) | pushrod: [1.24] Fonts: shared font handles and a built-in default font (done) |
| ✅ 26 | [#136](https://github.com/KenSuenobu/rust-pushrod/issues/136) | pushrod: [1.23] Public API polish before publishing: builders, prelude, and stability (done) |
| ✅ 27 | [#36](https://github.com/KenSuenobu/rust-pushrod/issues/36) | pushrod: [2.10] Custom paint widget (closure-drawn) (done) |
| ✅ 28 | [#27](https://github.com/KenSuenobu/rust-pushrod/issues/27) | pushrod: [2.1] Boxed label widget (done) |
| ✅ 29 | [#83](https://github.com/KenSuenobu/rust-pushrod/issues/83) | pushrod: [2.14] Separator line (done) |
| ✅ 30 | [#41](https://github.com/KenSuenobu/rust-pushrod/issues/41) | pushrod: [3.3] Group box (titled frame) (done) |
| ✅ 31 | [#33](https://github.com/KenSuenobu/rust-pushrod/issues/33) | pushrod: [2.7] Image widget (done) |
| ✅ 32 | [#28](https://github.com/KenSuenobu/rust-pushrod/issues/28) | pushrod: [2.2] Glyph button (small single-character button) (done) |
| ✅ 33 | [#29](https://github.com/KenSuenobu/rust-pushrod/issues/29) | pushrod: [2.3] Button behaviors: default, cancel, toggle, activate-on-press, dismiss dialog (done) |
| ✅ 34 | [#30](https://github.com/KenSuenobu/rust-pushrod/issues/30) | pushrod: [2.4] Check box with indeterminate state (done) |
| ✅ 35 | [#31](https://github.com/KenSuenobu/rust-pushrod/issues/31) | pushrod: [2.5] Radio buttons and radio groups (done) |
| ✅ 36 | [#48](https://github.com/KenSuenobu/rust-pushrod/issues/48) | pushrod: [4.1] Progress bar (done) |
| ✅ 37 | [#40](https://github.com/KenSuenobu/rust-pushrod/issues/40) | pushrod: [3.2] Slider (done) |
| ✅ 38 | [#32](https://github.com/KenSuenobu/rust-pushrod/issues/32) | pushrod: [2.6] Text field: single-line entry with input masks, validation, and numeric mode (done) |
| ✅ 39 | [#56](https://github.com/KenSuenobu/rust-pushrod/issues/56) | pushrod: [5.1] Clip children to their parent's bounds (done) |
| ✅ 40 | [#57](https://github.com/KenSuenobu/rust-pushrod/issues/57) | pushrod: [5.2] Composite widgets: build widgets from parts that applications can retrieve and modify (done) |
| ✅ 41 | [#116](https://github.com/KenSuenobu/rust-pushrod/issues/116) | pushrod: [5.5] Layout manager foundation: Layout trait, size hints, measure and arrange (done) |
| ✅ 42 | [#117](https://github.com/KenSuenobu/rust-pushrod/issues/117) | pushrod: [5.6] Row and column layouts (done) |
| ✅ 43 | [#118](https://github.com/KenSuenobu/rust-pushrod/issues/118) | pushrod: [5.7] Form layout: aligned label and field rows (done) |
| ✅ 44 | [#39](https://github.com/KenSuenobu/rust-pushrod/issues/39) | pushrod: [3.1] Scroll bar (done) |
| ✅ 45 | [#58](https://github.com/KenSuenobu/rust-pushrod/issues/58) | pushrod: [5.3] Container: a scrollable viewport onto fixed-size content (done) |
| ✅ 46 | [#70](https://github.com/KenSuenobu/rust-pushrod/issues/70) | pushrod: [7.2] Golden-image rendering tests (done) |
| ✅ 47 | [#115](https://github.com/KenSuenobu/rust-pushrod/issues/115) | pushrod: [7.7] Performance benchmarks and budgets (done) |
| ✅ 48 | [#25](https://github.com/KenSuenobu/rust-pushrod/issues/25) | pushrod: [1.10] High-DPI rendering and window resize (done) |
| ✅ 49 | [#139](https://github.com/KenSuenobu/rust-pushrod/issues/139) | pushrod: [7.9] Easy installation: bundled SDL, static linking, and platform guides (done) |
| ✅ 50 | [#23](https://github.com/KenSuenobu/rust-pushrod/issues/23) | pushrod: [1.8] Edit the widget tree at run time: remove, reparent, reorder (done) |
| ✅ 51 | [#66](https://github.com/KenSuenobu/rust-pushrod/issues/66) | pushrod: [6.7] Modal dialogs: run a form until a dismissing control is chosen (done) |
| ✅ 52 | [#35](https://github.com/KenSuenobu/rust-pushrod/issues/35) | pushrod: [2.9] Alert dialog (done) |
| ✅ 53 | [#140](https://github.com/KenSuenobu/rust-pushrod/issues/140) | pushrod: [7.10] Reference application: a real app built only with Pushrod (done for v0.1; form-file version waits for #63) |
| ✅ 54 | [#72](https://github.com/KenSuenobu/rust-pushrod/issues/72) | pushrod: [7.4] Guide: writing widgets and composites (done; registry chapter waits for #61) |
| ✅ 55 | [#73](https://github.com/KenSuenobu/rust-pushrod/issues/73) | pushrod: [7.5] Changelog, Cargo metadata, and crates.io release (done; publishing runs when v0.1.0 is tagged) |
| **56** | checkpoint | **Pushrod v0.1 released** |
| ✅ 57 | [#114](https://github.com/KenSuenobu/rust-pushrod/issues/114) | pushrod: [7.6] Module boundaries and written specs with conformance tests (done; the form file format spec waits for #62) |
| ✅ 58 | [#112](https://github.com/KenSuenobu/rust-pushrod/issues/112) | pushrod: [1.19] Input method (IME) support: composition text and candidate window (done for TextField; the text area and a real-IME check by hand are pending) |
| ✅ 59 | [#76](https://github.com/KenSuenobu/rust-pushrod/issues/76) | pushrod: [1.12] Keyboard mnemonics: underlined shortcut letters |
| ✅ 60 | [#60](https://github.com/KenSuenobu/rust-pushrod/issues/60) | pushrod: [6.1] Widget registry and factory |
| ✅ 61 | [#61](https://github.com/KenSuenobu/rust-pushrod/issues/61) | pushrod: [6.2] Property introspection: list, get, and set widget properties by name |
| 62 | [#80](https://github.com/KenSuenobu/rust-pushrod/issues/80) | pushrod: [1.14] Accessibility: expose widgets to screen readers |
| 63 | [#62](https://github.com/KenSuenobu/rust-pushrod/issues/62) | pushrod: [6.3] Form file format (binary, versioned) |
| 64 | [#63](https://github.com/KenSuenobu/rust-pushrod/issues/63) | pushrod: [6.4] Form loader: build a form from a file with stable IDs |
| 65 | [#64](https://github.com/KenSuenobu/rust-pushrod/issues/64) | pushrod: [6.5] Form writer: save a widget subtree to a file |
| 66 | [#65](https://github.com/KenSuenobu/rust-pushrod/issues/65) | pushrod: [6.6] Generate ID constants from a form file |
| 67 | [#154](https://github.com/KenSuenobu/rust-pushrod/issues/154) | pushrod: [6.11] Resource kinds: strings, images, icons, files, menus, and alerts |
| 68 | [#155](https://github.com/KenSuenobu/rust-pushrod/issues/155) | pushrod: [6.12] Typed resource IDs: generated constants per resource kind |
| 69 | [#153](https://github.com/KenSuenobu/rust-pushrod/issues/153) | pushrod: [6.10] Resource manager: load resource sets and get resources by ID |
| **70** | checkpoint | **Pushrod v0.2 released: the Pushrod MVP is complete** |
| 71 | [#295](https://github.com/KenSuenobu/rust-pushrod/issues/295) | pushrod: [12.1] Showcase application shell |
| 72 | [#296](https://github.com/KenSuenobu/rust-pushrod/issues/296) | pushrod: [12.2] Showcase pages for every standard widget, with a completeness test |
| 73 | [#297](https://github.com/KenSuenobu/rust-pushrod/issues/297) | pushrod: [12.3] Showcase property panel: change any property live |
| 74 | [#298](https://github.com/KenSuenobu/rust-pushrod/issues/298) | pushrod: [12.4] Showcase event log |
| 75 | [#299](https://github.com/KenSuenobu/rust-pushrod/issues/299) | pushrod: [12.5] Showcase source view: the code for each page |
| 76 | [#300](https://github.com/KenSuenobu/rust-pushrod/issues/300) | pushrod: [12.6] Showcase pages for core features: text, layouts, focus, dialogs, forms, and resources |
| 77 | [#301](https://github.com/KenSuenobu/rust-pushrod/issues/301) | pushrod: [12.7] Showcase in CI: scripted tour, golden screenshots, and documentation images |
| **78** | checkpoint | **Pushrod Showcase released** |
| 79 | [#156](https://github.com/KenSuenobu/rust-pushrod/issues/156) | pushrod: [6.13] Event metadata in the widget registry |
| 80 | [#164](https://github.com/KenSuenobu/rust-pushrod/issues/164) | chassis: [2.1] Project model |
| 81 | [#165](https://github.com/KenSuenobu/rust-pushrod/issues/165) | chassis: [6.1] Code generation rules |
| 82 | [#166](https://github.com/KenSuenobu/rust-pushrod/issues/166) | chassis: [5.7] Event wiring generation |
| 83 | [#167](https://github.com/KenSuenobu/rust-pushrod/issues/167) | chassis: [6.5] Command-line generator |
| **84** | checkpoint | **Chassis M1: core without UI complete** |
| 85 | [#97](https://github.com/KenSuenobu/rust-pushrod/issues/97) | pushrod: [9.5] Actions and keyboard shortcuts: shared commands for menus, toolbars, and buttons |
| 86 | [#100](https://github.com/KenSuenobu/rust-pushrod/issues/100) | pushrod: [9.8] Drag and drop within the application |
| 87 | [#81](https://github.com/KenSuenobu/rust-pushrod/issues/81) | pushrod: [1.15] Undo and redo command stack |
| 88 | [#59](https://github.com/KenSuenobu/rust-pushrod/issues/59) | pushrod: [5.4] Autoresizing (keep margins or stretch on resize) |
| 89 | [#52](https://github.com/KenSuenobu/rust-pushrod/issues/52) | pushrod: [4.5] List view |
| 90 | [#51](https://github.com/KenSuenobu/rust-pushrod/issues/51) | pushrod: [4.4] Tab view |
| 91 | [#92](https://github.com/KenSuenobu/rust-pushrod/issues/92) | pushrod: [4.9] Table view |
| 92 | [#90](https://github.com/KenSuenobu/rust-pushrod/issues/90) | pushrod: [3.10] Expander (collapsible section) |
| 93 | [#37](https://github.com/KenSuenobu/rust-pushrod/issues/37) | pushrod: [2.11] Menu bar and drop-down menus (drawn by Pushrod) |
| 94 | [#125](https://github.com/KenSuenobu/rust-pushrod/issues/125) | pushrod: [2.21] Image button |
| 95 | [#84](https://github.com/KenSuenobu/rust-pushrod/issues/84) | pushrod: [2.15] Toolbar |
| 96 | [#85](https://github.com/KenSuenobu/rust-pushrod/issues/85) | pushrod: [2.16] Status bar |
| 97 | [#34](https://github.com/KenSuenobu/rust-pushrod/issues/34) | pushrod: [2.8] Icon widget (picture with caption) |
| 98 | [#38](https://github.com/KenSuenobu/rust-pushrod/issues/38) | pushrod: [2.12] File chooser dialog (drawn by Pushrod) |
| 99 | [#47](https://github.com/KenSuenobu/rust-pushrod/issues/47) | pushrod: [3.9] Split view (panes with draggable dividers) |
| 100 | [#131](https://github.com/KenSuenobu/rust-pushrod/issues/131) | pushrod: [1.22] Widget libraries: extend or replace pushrod-widgets |
| 101 | [#252](https://github.com/KenSuenobu/rust-pushrod/issues/252) | extra-widgets: [1.1] Create the pushrod-extra-widgets crate |
| 102 | [#254](https://github.com/KenSuenobu/rust-pushrod/issues/254) | extra-widgets: [1.3] Popover: a floating panel anchored to a control |
| 103 | [#257](https://github.com/KenSuenobu/rust-pushrod/issues/257) | extra-widgets: [5.1] Drawer (collapsible side panel) |
| 104 | [#305](https://github.com/KenSuenobu/rust-pushrod/issues/305) | extra-widgets: [6.9] Command palette |
| 105 | [#168](https://github.com/KenSuenobu/rust-pushrod/issues/168) | chassis: [1.1] Chassis crate and main window |
| 106 | [#169](https://github.com/KenSuenobu/rust-pushrod/issues/169) | chassis: [1.2] Three-pane workspace |
| 107 | [#170](https://github.com/KenSuenobu/rust-pushrod/issues/170) | chassis: [1.4] Commands, menus, and shortcuts |
| 108 | [#171](https://github.com/KenSuenobu/rust-pushrod/issues/171) | chassis: [1.3] Designer and Code tabs |
| 109 | [#172](https://github.com/KenSuenobu/rust-pushrod/issues/172) | chassis: [2.2] New, open, and save |
| 110 | [#173](https://github.com/KenSuenobu/rust-pushrod/issues/173) | chassis: [2.3] Project drawer |
| 111 | [#174](https://github.com/KenSuenobu/rust-pushrod/issues/174) | chassis: [2.4] Manage resources and IDs |
| 112 | [#175](https://github.com/KenSuenobu/rust-pushrod/issues/175) | chassis: [2.5] String table editor |
| 113 | [#176](https://github.com/KenSuenobu/rust-pushrod/issues/176) | chassis: [2.6] Images and icons |
| 114 | [#177](https://github.com/KenSuenobu/rust-pushrod/issues/177) | chassis: [2.8] Files |
| **115** | checkpoint | **Chassis M2: shell and project complete** |
| 116 | [#95](https://github.com/KenSuenobu/rust-pushrod/issues/95) | pushrod: [9.3] Event filters: observe or intercept another widget's events |
| 117 | [#105](https://github.com/KenSuenobu/rust-pushrod/issues/105) | pushrod: [10.1] Drawing modes: live or outline feedback while moving and resizing |
| 118 | [#121](https://github.com/KenSuenobu/rust-pushrod/issues/121) | pushrod: [5.10] Flow layout (wrapping rows) |
| 119 | [#77](https://github.com/KenSuenobu/rust-pushrod/issues/77) | pushrod: [2.13] Tooltips |
| 120 | [#68](https://github.com/KenSuenobu/rust-pushrod/issues/68) | pushrod: [6.9] Design-mode hooks for Chassis |
| 121 | [#42](https://github.com/KenSuenobu/rust-pushrod/issues/42) | extra-widgets: [2.6] Tool palette (grid of tool buttons) |
| 122 | [#256](https://github.com/KenSuenobu/rust-pushrod/issues/256) | extra-widgets: [4.9] Breadcrumb bar (path control) |
| 123 | [#304](https://github.com/KenSuenobu/rust-pushrod/issues/304) | extra-widgets: [4.14] Canvas view (zoomable surface of items) |
| 124 | [#178](https://github.com/KenSuenobu/rust-pushrod/issues/178) | chassis: [3.1] Design surface |
| 125 | [#179](https://github.com/KenSuenobu/rust-pushrod/issues/179) | chassis: [3.2] Widget palette |
| 126 | [#180](https://github.com/KenSuenobu/rust-pushrod/issues/180) | chassis: [3.4] Selection, move, and resize |
| 127 | [#181](https://github.com/KenSuenobu/rust-pushrod/issues/181) | chassis: [3.3] Drag and drop placement |
| 128 | [#182](https://github.com/KenSuenobu/rust-pushrod/issues/182) | chassis: [3.5] Snapping and alignment |
| 129 | [#183](https://github.com/KenSuenobu/rust-pushrod/issues/183) | chassis: [3.7] Clipboard and undo |
| 130 | [#184](https://github.com/KenSuenobu/rust-pushrod/issues/184) | chassis: [3.6] Containers, z-order, and tab order |
| 131 | [#185](https://github.com/KenSuenobu/rust-pushrod/issues/185) | chassis: [3.8] Layout managers in the designer |
| **132** | checkpoint | **Chassis M3: form designer complete** |
| 133 | [#101](https://github.com/KenSuenobu/rust-pushrod/issues/101) | pushrod: [9.9] Pointer cursor per widget |
| 134 | [#44](https://github.com/KenSuenobu/rust-pushrod/issues/44) | pushrod: [3.6] Choice button: pop-up list and click-to-cycle modes |
| 135 | [#88](https://github.com/KenSuenobu/rust-pushrod/issues/88) | pushrod: [2.19] Context menus |
| 136 | [#113](https://github.com/KenSuenobu/rust-pushrod/issues/113) | pushrod: [1.20] Layers and invalidation: cache hints, texture memory budget, incremental pipeline |
| 137 | [#142](https://github.com/KenSuenobu/rust-pushrod/issues/142) | pushrod: [11.1] Chassis: Code editor crate and editing core |
| 138 | [#143](https://github.com/KenSuenobu/rust-pushrod/issues/143) | pushrod: [11.2] Chassis: Code editor view: gutter, visible-line rendering, caret |
| 139 | [#144](https://github.com/KenSuenobu/rust-pushrod/issues/144) | pushrod: [11.3] Chassis: Code editor input: keyboard, mouse selection, clipboard, input methods |
| 140 | [#145](https://github.com/KenSuenobu/rust-pushrod/issues/145) | pushrod: [11.4] Chassis: Code editor undo and redo |
| 141 | [#146](https://github.com/KenSuenobu/rust-pushrod/issues/146) | pushrod: [11.5] Chassis: Code editor syntax highlighting with tree-sitter |
| 142 | [#149](https://github.com/KenSuenobu/rust-pushrod/issues/149) | pushrod: [11.8] Chassis: Code editor API for the Chassis designer |
| 143 | [#302](https://github.com/KenSuenobu/rust-pushrod/issues/302) | extra-widgets: [2.9] Drag value (numeric scrubber) |
| 144 | [#255](https://github.com/KenSuenobu/rust-pushrod/issues/255) | extra-widgets: [4.5] Property grid (name and value inspector) |
| 145 | [#186](https://github.com/KenSuenobu/rust-pushrod/issues/186) | chassis: [4.1] Inspector drawer |
| 146 | [#187](https://github.com/KenSuenobu/rust-pushrod/issues/187) | chassis: [4.2] Property grid |
| 147 | [#188](https://github.com/KenSuenobu/rust-pushrod/issues/188) | chassis: [4.3] Typed property editors |
| 148 | [#189](https://github.com/KenSuenobu/rust-pushrod/issues/189) | chassis: [5.4] Code view |
| 149 | [#190](https://github.com/KenSuenobu/rust-pushrod/issues/190) | chassis: [5.3] Create and open handlers |
| 150 | [#191](https://github.com/KenSuenobu/rust-pushrod/issues/191) | chassis: [4.4] Names and IDs |
| 151 | [#192](https://github.com/KenSuenobu/rust-pushrod/issues/192) | chassis: [5.1] Events context menu |
| 152 | [#193](https://github.com/KenSuenobu/rust-pushrod/issues/193) | chassis: [5.2] Inspector Events tab |
| 153 | [#194](https://github.com/KenSuenobu/rust-pushrod/issues/194) | chassis: [5.5] Code indicator on the design surface |
| 154 | [#195](https://github.com/KenSuenobu/rust-pushrod/issues/195) | chassis: [5.6] Keep handlers in sync |
| **155** | checkpoint | **Chassis M4: inspector, events, and code complete** |
| 156 | [#303](https://github.com/KenSuenobu/rust-pushrod/issues/303) | extra-widgets: [4.13] Log view (append-only output) |
| 157 | [#196](https://github.com/KenSuenobu/rust-pushrod/issues/196) | chassis: [6.2] Build and run |
| 158 | [#197](https://github.com/KenSuenobu/rust-pushrod/issues/197) | chassis: [7.1] CI and golden screenshots |
| 159 | [#198](https://github.com/KenSuenobu/rust-pushrod/issues/198) | chassis: [7.2] Chassis documentation |
| 160 | [#199](https://github.com/KenSuenobu/rust-pushrod/issues/199) | chassis: [7.3] Sample project |
| **161** | checkpoint | **Chassis MVP released** |
| 162 | [#208](https://github.com/KenSuenobu/rust-pushrod/issues/208) | chassis: [8.1] Hello, Chassis |
| 163 | [#209](https://github.com/KenSuenobu/rust-pushrod/issues/209) | chassis: [8.2] Temperature converter |
| 164 | [#210](https://github.com/KenSuenobu/rust-pushrod/issues/210) | chassis: [8.3] Tip calculator |
| 165 | [#211](https://github.com/KenSuenobu/rust-pushrod/issues/211) | chassis: [8.4] Settings dialog |
| 166 | [#212](https://github.com/KenSuenobu/rust-pushrod/issues/212) | chassis: [8.5] Image viewer |
| 167 | [#213](https://github.com/KenSuenobu/rust-pushrod/issues/213) | chassis: [8.6] Stopwatch |
| 168 | [#214](https://github.com/KenSuenobu/rust-pushrod/issues/214) | chassis: [8.7] Quiz |
| 169 | [#215](https://github.com/KenSuenobu/rust-pushrod/issues/215) | chassis: [8.8] Dice roller |
| **170** | checkpoint | **All example projects built** |
| 171 | [#98](https://github.com/KenSuenobu/rust-pushrod/issues/98) | pushrod: [9.6] Property bindings: keep two properties in sync |
| 172 | [#87](https://github.com/KenSuenobu/rust-pushrod/issues/87) | pushrod: [2.18] Multi-line text area |
| 173 | [#126](https://github.com/KenSuenobu/rust-pushrod/issues/126) | pushrod: [2.22] Link label |
| 174 | [#82](https://github.com/KenSuenobu/rust-pushrod/issues/82) | pushrod: [1.16] Multiple windows: separate OS windows, each with its own engine |
| 175 | [#67](https://github.com/KenSuenobu/rust-pushrod/issues/67) | pushrod: [6.8] Bind actions by name from form files |
| 176 | [#107](https://github.com/KenSuenobu/rust-pushrod/issues/107) | pushrod: [10.3] no_std + alloc core crate |
| 177 | [#108](https://github.com/KenSuenobu/rust-pushrod/issues/108) | pushrod: [10.4] Experimental: Atari ST / TOS backend |
| 178 | [#135](https://github.com/KenSuenobu/rust-pushrod/issues/135) | pushrod: [10.5] SDL3 backend (pushrod-sdl3) |
| 179 | [#308](https://github.com/KenSuenobu/rust-pushrod/issues/308) | extra-widgets: [4.15] Rich text view (formatted, read-only) |
| 180 | [#200](https://github.com/KenSuenobu/rust-pushrod/issues/200) | chassis: [1.5] Workspace settings |
| 181 | [#201](https://github.com/KenSuenobu/rust-pushrod/issues/201) | chassis: [2.7] Menu editor |
| 182 | [#202](https://github.com/KenSuenobu/rust-pushrod/issues/202) | chassis: [3.9] Form preview |
| 183 | [#203](https://github.com/KenSuenobu/rust-pushrod/issues/203) | chassis: [4.5] Multiple selection editing |
| 184 | [#204](https://github.com/KenSuenobu/rust-pushrod/issues/204) | chassis: [6.3] Error list |
| 185 | [#205](https://github.com/KenSuenobu/rust-pushrod/issues/205) | chassis: [6.4] Project templates |
| 186 | [#206](https://github.com/KenSuenobu/rust-pushrod/issues/206) | chassis: [7.4] Packaging and releases |
| 187 | [#216](https://github.com/KenSuenobu/rust-pushrod/issues/216) | chassis: [1.6] Find in project |
| 188 | [#217](https://github.com/KenSuenobu/rust-pushrod/issues/217) | chassis: [1.7] Multiple windows per project |
| 189 | [#218](https://github.com/KenSuenobu/rust-pushrod/issues/218) | chassis: [2.9] Localization |
| 190 | [#219](https://github.com/KenSuenobu/rust-pushrod/issues/219) | chassis: [2.10] Source control status |
| 191 | [#220](https://github.com/KenSuenobu/rust-pushrod/issues/220) | chassis: [3.10] Visual form inheritance |
| 192 | [#221](https://github.com/KenSuenobu/rust-pushrod/issues/221) | chassis: [3.11] Accessibility and usability checks |
| 193 | [#222](https://github.com/KenSuenobu/rust-pushrod/issues/222) | chassis: [4.6] Data bindings in the inspector |
| 194 | [#223](https://github.com/KenSuenobu/rust-pushrod/issues/223) | chassis: [5.8] Code intelligence with rust-analyzer |
| 195 | [#224](https://github.com/KenSuenobu/rust-pushrod/issues/224) | chassis: [6.6] Build targets, including the Atari ST |
| 196 | [#312](https://github.com/KenSuenobu/rust-pushrod/issues/312) | chassis: [9.1] Command catalog: every designer command by name, with typed arguments |
| 197 | [#313](https://github.com/KenSuenobu/rust-pushrod/issues/313) | chassis: [9.2] MCP server inside Chassis |
| 198 | [#314](https://github.com/KenSuenobu/rust-pushrod/issues/314) | chassis: [9.3] Assistant tools: read the project |
| 199 | [#315](https://github.com/KenSuenobu/rust-pushrod/issues/315) | chassis: [9.4] Assistant tools: edit forms and resources |
| 200 | [#316](https://github.com/KenSuenobu/rust-pushrod/issues/316) | chassis: [9.5] Assistant tools: events, handler code, and builds |
| 201 | [#317](https://github.com/KenSuenobu/rust-pushrod/issues/317) | chassis: [9.6] Assistant tool: a picture of the form |
| 202 | [#318](https://github.com/KenSuenobu/rust-pushrod/issues/318) | chassis: [9.7] Assistant activity, review mode, and safety |
| 203 | [#319](https://github.com/KenSuenobu/rust-pushrod/issues/319) | chassis: [9.8] Connect dialog and assistant guide |
| 204 | [#322](https://github.com/KenSuenobu/rust-pushrod/issues/322) | chassis: [9.10] Assistant settings: every check is configurable |
| 205 | [#320](https://github.com/KenSuenobu/rust-pushrod/issues/320) | chassis: [9.9] Assistant drawer: chat inside Chassis |
| **206** | checkpoint | **Chassis v2 released** |
| 207 | [#120](https://github.com/KenSuenobu/rust-pushrod/issues/120) | pushrod: [5.9] Stack layout (one child visible at a time) |
| 208 | [#49](https://github.com/KenSuenobu/rust-pushrod/issues/49) | pushrod: [4.2] Spin box |
| 209 | [#53](https://github.com/KenSuenobu/rust-pushrod/issues/53) | pushrod: [4.6] Combo box |
| 210 | [#253](https://github.com/KenSuenobu/rust-pushrod/issues/253) | extra-widgets: [1.2] Extra widgets gallery example |
| 211 | [#258](https://github.com/KenSuenobu/rust-pushrod/issues/258) | extra-widgets: [2.1] Menu button and split button |
| 212 | [#50](https://github.com/KenSuenobu/rust-pushrod/issues/50) | extra-widgets: [2.2] Segmented control |
| 213 | [#259](https://github.com/KenSuenobu/rust-pushrod/issues/259) | extra-widgets: [2.3] Stepper (up and down arrows) |
| 214 | [#127](https://github.com/KenSuenobu/rust-pushrod/issues/127) | extra-widgets: [2.4] Dial (rotary knob) |
| 215 | [#54](https://github.com/KenSuenobu/rust-pushrod/issues/54) | extra-widgets: [2.5] Level indicator (meter) |
| 216 | [#306](https://github.com/KenSuenobu/rust-pushrod/issues/306) | extra-widgets: [2.10] Range slider (two thumbs) |
| 217 | [#260](https://github.com/KenSuenobu/rust-pushrod/issues/260) | extra-widgets: [3.1] Calendar (month view) |
| 218 | [#91](https://github.com/KenSuenobu/rust-pushrod/issues/91) | extra-widgets: [3.2] Date picker |
| 219 | [#261](https://github.com/KenSuenobu/rust-pushrod/issues/261) | extra-widgets: [3.3] Time field and date-time field |
| 220 | [#46](https://github.com/KenSuenobu/rust-pushrod/issues/46) | extra-widgets: [3.4] Color well and color picker |
| 221 | [#129](https://github.com/KenSuenobu/rust-pushrod/issues/129) | extra-widgets: [3.5] Font chooser |
| 222 | [#262](https://github.com/KenSuenobu/rust-pushrod/issues/262) | extra-widgets: [3.6] Path field (file or folder with Browse) |
| 223 | [#307](https://github.com/KenSuenobu/rust-pushrod/issues/307) | extra-widgets: [3.10] Token field (tags in a field) |
| 224 | [#55](https://github.com/KenSuenobu/rust-pushrod/issues/55) | extra-widgets: [4.1] Tree view (outline) |
| 225 | [#263](https://github.com/KenSuenobu/rust-pushrod/issues/263) | extra-widgets: [4.6] Check list |
| 226 | [#264](https://github.com/KenSuenobu/rust-pushrod/issues/264) | extra-widgets: [4.7] Editable list (add, remove, reorder) |
| 227 | [#265](https://github.com/KenSuenobu/rust-pushrod/issues/265) | extra-widgets: [5.2] Accordion (one section open at a time) |
| 228 | [#266](https://github.com/KenSuenobu/rust-pushrod/issues/266) | extra-widgets: [5.3] Sidebar (navigation list) |
| 229 | [#267](https://github.com/KenSuenobu/rust-pushrod/issues/267) | extra-widgets: [6.1] Input dialog (ask for one value) |
| 230 | [#268](https://github.com/KenSuenobu/rust-pushrod/issues/268) | extra-widgets: [6.2] Progress dialog |
| 231 | [#269](https://github.com/KenSuenobu/rust-pushrod/issues/269) | extra-widgets: [6.3] Task dialog (alert with details and options) |
| 232 | [#309](https://github.com/KenSuenobu/rust-pushrod/issues/309) | extra-widgets: [6.10] Recent files menu |
| **233** | checkpoint | **pushrod-extra-widgets MVP released** |
| 234 | [#270](https://github.com/KenSuenobu/rust-pushrod/issues/270) | extra-widgets: [2.7] Ruler |
| 235 | [#271](https://github.com/KenSuenobu/rust-pushrod/issues/271) | extra-widgets: [2.8] Shortcut field (key combination recorder) |
| 236 | [#272](https://github.com/KenSuenobu/rust-pushrod/issues/272) | extra-widgets: [3.7] Image well |
| 237 | [#273](https://github.com/KenSuenobu/rust-pushrod/issues/273) | extra-widgets: [3.8] New password (enter, confirm, strength) |
| 238 | [#274](https://github.com/KenSuenobu/rust-pushrod/issues/274) | extra-widgets: [3.9] Character picker |
| 239 | [#45](https://github.com/KenSuenobu/rust-pushrod/issues/45) | extra-widgets: [4.2] Column browser |
| 240 | [#275](https://github.com/KenSuenobu/rust-pushrod/issues/275) | extra-widgets: [4.3] Icon grid view |
| 241 | [#276](https://github.com/KenSuenobu/rust-pushrod/issues/276) | extra-widgets: [4.4] Data grid (editable table) |
| 242 | [#277](https://github.com/KenSuenobu/rust-pushrod/issues/277) | extra-widgets: [4.10] Filter field (live filter for lists, tables, and trees) |
| 243 | [#278](https://github.com/KenSuenobu/rust-pushrod/issues/278) | extra-widgets: [4.8] Dual list (list builder) |
| 244 | [#279](https://github.com/KenSuenobu/rust-pushrod/issues/279) | extra-widgets: [4.11] Rule editor (query builder) |
| 245 | [#280](https://github.com/KenSuenobu/rust-pushrod/issues/280) | extra-widgets: [4.12] Image viewer (zoom and pan) |
| 246 | [#281](https://github.com/KenSuenobu/rust-pushrod/issues/281) | extra-widgets: [5.4] Document tabs (closable and reorderable) |
| 247 | [#282](https://github.com/KenSuenobu/rust-pushrod/issues/282) | extra-widgets: [6.4] Wizard (step-by-step assistant) |
| 248 | [#283](https://github.com/KenSuenobu/rust-pushrod/issues/283) | extra-widgets: [6.5] Settings dialog |
| 249 | [#284](https://github.com/KenSuenobu/rust-pushrod/issues/284) | extra-widgets: [6.6] About dialog |
| 250 | [#285](https://github.com/KenSuenobu/rust-pushrod/issues/285) | extra-widgets: [6.7] Find and replace bar |
| 251 | [#286](https://github.com/KenSuenobu/rust-pushrod/issues/286) | extra-widgets: [6.8] Keyboard shortcuts editor |
| **252** | checkpoint | **Extra widgets 3 released** |
| 253 | [#99](https://github.com/KenSuenobu/rust-pushrod/issues/99) | pushrod: [9.7] Callback handles: disconnect, block, and unblock |
| 254 | [#26](https://github.com/KenSuenobu/rust-pushrod/issues/26) | pushrod: [1.11] Application lifecycle: startup and close-request callbacks |
| 255 | [#119](https://github.com/KenSuenobu/rust-pushrod/issues/119) | pushrod: [5.8] Grid layout |
| 256 | [#86](https://github.com/KenSuenobu/rust-pushrod/issues/86) | pushrod: [2.17] Switch (on/off slide toggle) |
| 257 | [#89](https://github.com/KenSuenobu/rust-pushrod/issues/89) | pushrod: [2.20] Message bar (non-modal inline notice) |
| 258 | [#128](https://github.com/KenSuenobu/rust-pushrod/issues/128) | pushrod: [4.10] Busy spinner |
| 259 | [#291](https://github.com/KenSuenobu/rust-pushrod/issues/291) | pushrod: [2.23] Selectable label text |
| 260 | [#292](https://github.com/KenSuenobu/rust-pushrod/issues/292) | pushrod: [2.24] Count badge on buttons, tabs, and rows |
| 261 | [#293](https://github.com/KenSuenobu/rust-pushrod/issues/293) | pushrod: [4.11] Empty-state text for list, table, and other views |
| 262 | [#294](https://github.com/KenSuenobu/rust-pushrod/issues/294) | pushrod: [4.12] Arithmetic in number fields |
| 263 | [#24](https://github.com/KenSuenobu/rust-pushrod/issues/24) | pushrod: [1.9] Dirty-rectangle redraw |
| 264 | [#147](https://github.com/KenSuenobu/rust-pushrod/issues/147) | pushrod: [11.6] Chassis: Code editor find |
| 265 | [#148](https://github.com/KenSuenobu/rust-pushrod/issues/148) | pushrod: [11.7] Chassis: Code editor auto-indent and bracket matching |
| 266 | [#150](https://github.com/KenSuenobu/rust-pushrod/issues/150) | pushrod: [11.9] Chassis: Code editor documentation, example, and tests |
| 267 | [#74](https://github.com/KenSuenobu/rust-pushrod/issues/74) | pushrod: [8.1] Theme for colors and metrics (future) |
| 268 | [#75](https://github.com/KenSuenobu/rust-pushrod/issues/75) | pushrod: [8.2] Skins: image-based widget rendering (future) |

## Part 1: Pushrod v0.1, dialog applications

Everything needed to build and ship real dialog-style applications with Pushrod, ending with the first crates.io release.

### Step 1: Split the standard widgets into the pushrod-widgets crate

**Issue:** [#130](https://github.com/KenSuenobu/rust-pushrod/issues/130) `pushrod: [1.21] Split the standard widgets into the pushrod-widgets crate`

**Delivers:** Make Pushrod a Cargo workspace: `pushrod` is the core, and the standard widgets move to their own crate, `pushrod-widgets`.

**Status:** done (issue closed). Its video can be a recap of the finished work.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [ ] `cargo test --workspace`, clippy, and docs pass
- [ ] The core builds and runs with no widget crate (root widget only)
- [ ] `pushrod-widgets` uses only public core API
- [ ] Docs, README, and examples show both dependencies

### Step 2: CI: build, test, clippy, fmt, doc with SDL2

**Issue:** [#69](https://github.com/KenSuenobu/rust-pushrod/issues/69) `pushrod: [7.1] CI: build, test, clippy, fmt, doc with SDL2`

**Delivers:** Every PR is built and tested on Linux, macOS, and Windows.

**Status:** done (issue closed). Its video can be a recap of the finished work.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [ ] Runs on PRs and main
- [ ] Fails on warnings, formatting, or tests
- [ ] README badge
- [ ] Sample-data check flags real-looking e-mail addresses and user home paths

### Step 3: Documentation site: build pushrod-docs in CI and publish it

**Issue:** [#122](https://github.com/KenSuenobu/rust-pushrod/issues/122) `pushrod: [7.8] Documentation site: build pushrod-docs in CI and publish it`

**Delivers:** Build the Docusaurus site on every PR (broken links fail) and publish it to GitHub Pages, with the API reference linked.

**Builds on:** step 2.

**Show on video:**

- [x] PRs fail when the docs build fails
- [x] `main` deploys to GitHub Pages
- [x] API reference published and linked
- [x] Missing widget pages are reported
- [x] Checked code samples compile in CI

**Status:** done (issue closed). Its video can be a recap of the finished work.

### Step 4: Event vocabulary: modern event names and structure

**Issue:** [#93](https://github.com/KenSuenobu/rust-pushrod/issues/93) `pushrod: [9.1] Event vocabulary: modern event names and structure`

**Delivers:** Define Pushrod's event enum and naming once; rename the PR #10 events.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [x] `pushrod::Event` with the variants above; `EventKind`; `widget()`
- [x] PR #10 behavior and tests ported to the new names
- [x] `Button` uses `PointerPressed/Released/Entered/Exited`
- [x] Rustdoc lists every event with when it is emitted

**Status:** done (issue closed). Its video can be a recap of the finished work.

### Step 5: Backend-neutral paint API (SDL2 backend by default)

**Issue:** [#106](https://github.com/KenSuenobu/rust-pushrod/issues/106) `pushrod: [10.2] Backend-neutral paint API (SDL2 backend by default)`

**Delivers:** Widgets paint through a small Painter trait instead of an SDL2 canvas, so other backends become possible without touching widgets.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [x] All widgets paint through `Painter`; no widget imports `sdl2::render`
- [x] SDL2 backend renders pixel-identical goldens
- [x] A headless test backend records paint calls for unit tests
- [x] Cache policy is backend-selected

**Status:** done (issue closed). Its video can be a recap of the finished work.

### Step 6: Widget behavior flags and visual states

**Issue:** [#19](https://github.com/KenSuenobu/rust-pushrod/issues/19) `pushrod: [1.1] Widget behavior flags and visual states`

**Delivers:** Behavior flags (toggle, default, cancel, exclusive, focusable, ...) and visual states (pressed, checked, disabled, ...) on every widget.

**Builds on:** step 4.

**Mockups:** [1.1-1.png](../mockups/1.1-1.png)

**Show on video:**

- [x] `Behavior` and `VisualState` exist on `WidgetCore` with default trait accessors
- [x] `set_hidden`/`is_hidden` are implemented on `HIDDEN` (no API break)
- [x] Disabled widgets are skipped by input routing and focus
- [x] `paint::state_overlay` draws DISABLED, FOCUSED, and INDETERMINATE after `paint()`, using a cached GPU dither texture
- [x] Tests for flags, states, and disabled routing

**Status:** done (issue closed). Its video can be a recap of the finished work.

### Step 7: Event masks for the run loop and for widgets

**Issue:** [#6](https://github.com/KenSuenobu/rust-pushrod/issues/6) `pushrod: [1.2] Event masks for the run loop and for widgets`

**Delivers:** Filter events for the whole run loop and per widget, so only events in the mask are handled.

**Builds on:** step 4.

**Show on video:**

- [ ] Run-loop mask filters before routing and callbacks; changes apply on the next event
- [ ] Masked SDL event types are disabled in SDL and re-enabled when the mask changes
- [ ] Close requests are always delivered
- [ ] Idle loop blocks on `wait_event_timeout` (near-zero idle CPU)
- [ ] Widgets never receive events outside their mask; a `Frame` receives nothing while a button still clicks

### Step 8: Engine events: every internal action Pushrod performs emits an event

**Issue:** [#109](https://github.com/KenSuenobu/rust-pushrod/issues/109) `pushrod: [9.10] Engine events: every internal action Pushrod performs emits an event`

**Delivers:** A system-wide EngineEvent for each internal action (mode changes, focus, modals, menus, tree changes, forms, drags, timers, window changes), broadcast to widgets and callbacks.

**Builds on:** step 4, step 7.

**Show on video:**

- [ ] `EngineEvent` enum with the initial variants above (those whose features exist are emitted now; the rest are added by their issues)
- [ ] Broadcast after completion to masked widgets and `on_engine_event`
- [ ] Kind filter for widgets and callbacks
- [ ] No re-entrant loops (engine events never trigger internal actions on their own)
- [ ] Guide and PR checklist state the rule

### Step 9: Event-loop style: keep parity with callbacks

**Issue:** [#124](https://github.com/KenSuenobu/rust-pushrod/issues/124) `pushrod: [9.11] Event-loop style: keep parity with callbacks`

**Delivers:** Everything delivered to callbacks is also returned by next_event, and both styles can be combined.

**Builds on:** step 4.

**Show on video:**

- [ ] Every event kind is delivered by both styles in the same order
- [ ] `next_event_with` runs callbacks then returns the event
- [ ] Conformance test compares both styles on the same input

### Step 10: Broadcast events to all interested widgets

**Issue:** [#7](https://github.com/KenSuenobu/rust-pushrod/issues/7) `pushrod: [1.3] Broadcast events to all interested widgets`

**Delivers:** Window-level and application-wide events go to every widget whose mask includes them.

**Builds on:** step 7.

**Show on video:**

- [ ] Window focus and resize events reach widgets with `WINDOW` in their mask
- [ ] Order is deterministic (draw order) and tested

### Step 11: Callback registry for every widget event

**Issue:** [#22](https://github.com/KenSuenobu/rust-pushrod/issues/22) `pushrod: [1.6] Callback registry for every widget event`

**Delivers:** on_click, on_press, on_release, on_hover, on_toggle, on_value_change, on_selection_change, on_text_change, on_editing_finished, on_activate, on_key_press, on_focus_change.

**Builds on:** step 4.

**Show on video:**

- [ ] Each method fires only for its widget and event kind
- [ ] Callbacks run in registration order after the widget updated itself
- [ ] Tests for each kind

### Step 12: Custom application events with payloads

**Issue:** [#78](https://github.com/KenSuenobu/rust-pushrod/issues/78) `pushrod: [1.13] Custom application events with payloads`

**Delivers:** Applications and widgets fire their own typed events; the core ignores them but delivers them to handle_event and callbacks.

**Builds on:** step 4, step 7.

**Show on video:**

- [ ] `ui.post`, `ctx.post`, and `sender().post` deliver the payload intact
- [ ] Only targeted widgets receive it in `handle_event`
- [ ] `on_custom::<T>` fires only for payload type T
- [ ] No core side effects
- [ ] Runaway re-posting is capped
- [ ] Tests for each target and cross-thread posting

### Step 13: Text layout engine: Parley for shaping, bidi, line breaking, and font fallback (with an SDL2 compatibility gate)

**Issue:** [#111](https://github.com/KenSuenobu/rust-pushrod/issues/111) `pushrod: [1.18] Text layout engine: Parley for shaping, bidi, line breaking, and font fallback (with an SDL2 compatibility gate)`

**Delivers:** Lay out text with a real layout engine and feed its glyph runs into the GPU glyph atlas.

**Builds on:** step 5.

**Show on video:**

- [x] Spike passes the compatibility gate (or Parley is feature-gated off by default with the reason recorded in the issue)
- [x] Glyphs rasterized once into the SDL2 atlas; steady-state drawing is texture copies only
- [x] RTL, CJK, and emoji fallback render correctly; caret/selection geometry drives text fields
- [x] Layouts cached; recomputed only on text/font/width change; provide size hints
- [x] `TextLayout` trait; Parley is the default engine only once the glyph atlas (#3) lands (feature-gated until then)

### Step 14: GPU text: font cache with a glyph atlas texture

**Issue:** [#3](https://github.com/KenSuenobu/rust-pushrod/issues/3) `pushrod: [1.7] GPU text: font cache with a glyph atlas texture`

**Delivers:** Load each font once, rasterize glyphs once into a GPU atlas, and draw text by blitting glyphs.

**Builds on:** step 13.

**Show on video:**

- [x] Each font is loaded at most once per process
- [x] After warm-up, repainting text performs only texture copies
- [x] Text color changes use color modulation
- [x] `FontCache` is owned by `WidgetCache`

### Step 15: Widget gallery example

**Issue:** [#71](https://github.com/KenSuenobu/rust-pushrod/issues/71) `pushrod: [7.3] Widget gallery example`

**Delivers:** One window showing every widget and state.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [x] Shows every merged widget
- [x] README screenshot from it

### Step 16: Event propagation: capture, target, and bubble phases with handled/ignored

**Issue:** [#94](https://github.com/KenSuenobu/rust-pushrod/issues/94) `pushrod: [9.2] Event propagation: capture, target, and bubble phases with handled/ignored`

**Delivers:** Input events travel from the window down to the target widget and back up to its parents, and any widget can mark them handled.

**Builds on:** step 4.

**Show on video:**

- [x] Capture, target, bubble order tested
- [x] `Propagation::Stop` ends delivery
- [x] Return in a text field reaches the dialog's default-button handling

### Step 17: Reusable input controllers: click, drag, scroll, hover, long-press, key

**Issue:** [#96](https://github.com/KenSuenobu/rust-pushrod/issues/96) `pushrod: [9.4] Reusable input controllers: click, drag, scroll, hover, long-press, key`

**Delivers:** Composable input handlers that widgets attach instead of re-implementing press/drag/release logic.

**Builds on:** step 4, step 16.

**Show on video:**

- [x] Controllers for click, drag, scroll, hover, long-press, key
- [x] Button ported with identical behavior (PR #10 tests pass)
- [ ] Slider and scroll bar use DragController (the widgets do not exist yet; `DragController` is ready)

### Step 18: Keyboard focus, Tab order, and Return/Escape

**Issue:** [#20](https://github.com/KenSuenobu/rust-pushrod/issues/20) `pushrod: [1.4] Keyboard focus, Tab order, and Return/Escape`

**Delivers:** Focusable widgets take keyboard focus; Tab moves it; Return and Escape trigger the default and cancel buttons.

**Builds on:** step 6, step 4, step 16.

**Show on video:**

- [x] Focus moves by click, Tab, Shift-Tab, and arrows within groups
- [x] Return/Escape activate default/cancel
- [x] `FocusGained`/`FocusLost` delivered and tested
- [x] `Ui::set_focus`/`Ui::focused`
- [x] Emits the corresponding `EngineEvent` (9.10)

### Step 19: Per-state appearance: colors, border, and font for each visual state

**Issue:** [#104](https://github.com/KenSuenobu/rust-pushrod/issues/104) `pushrod: [1.17] Per-state appearance: colors, border, and font for each visual state`

**Delivers:** Every widget carries appearance attributes for normal, pressed, hovered, focused, disabled, and drag-over, so the look changes automatically as the state changes.

**Builds on:** step 6, step 4.

**Mockups:** [1.17-1.png](../mockups/1.17-1.png), [1.17-2.png](../mockups/1.17-2.png)

**Show on video:**

- [x] `StateStyle` and `Appearance` on every widget with fallback to normal and built-in defaults
- [x] Hover, press/release, focus, disabled, checked, and drag-over switch the effective style automatically (the drag-over flag is set by drag and drop, which is not built yet; `set_state` sets it by hand)
- [x] Default look unchanged (golden images identical; no golden images exist yet, so the existing paint tests guard it)
- [x] Repaint only when the effective style changes
- [ ] Stored in form files (deferred to 6.2 (#61), property introspection)

### Step 20: Text truncation: shorten text that doesn't fit with an ellipsis

**Issue:** [#289](https://github.com/KenSuenobu/rust-pushrod/issues/289) `pushrod: [1.26] Text truncation: shorten text that doesn't fit with an ellipsis`

**Delivers:** Text that is wider than its space is cut cleanly and ends in an ellipsis, at the end, start, or middle.

**Builds on:** step 14, step 13.

**Mockups:** [1.26-1.png](../mockups/1.26-1.png)

**Show on video:**

- [x] End, Start, and Middle modes cut on grapheme boundaries and never overflow the widget
- [x] Text that fits is drawn unchanged
- [x] `is_truncated()` reports shortening
- [x] Text and Button widgets default to End; paint tests cover all modes

### Step 21: Wrapped text: multi-line labels that reflow

**Issue:** [#290](https://github.com/KenSuenobu/rust-pushrod/issues/290) `pushrod: [1.27] Wrapped text: multi-line labels that reflow`

**Delivers:** Labels wrap their text onto several lines at word boundaries and reflow when their width changes.

**Builds on:** step 20, step 14, step 13.

**Mockups:** [1.27-1.png](../mockups/1.27-1.png)

**Show on video:**

- [x] Text wraps at word boundaries and reflows when the width changes
- [x] `max_lines` truncates the last line with an ellipsis
- [x] `height_for_width` is reported and used by layouts once 5.5 lands
- [x] Explicit line breaks and per-line alignment

### Step 22: Timers and idle callbacks

**Issue:** [#21](https://github.com/KenSuenobu/rust-pushrod/issues/21) `pushrod: [1.5] Timers and idle callbacks`

**Delivers:** Repeating/one-shot timers, per-widget ticks for animation, and idle callbacks.

**Builds on:** step 7.

**Show on video:**

- [x] One-shot and repeating timers fire within one frame of their interval
- [x] Timers can be cancelled from callbacks
- [x] `Tick` only flows while a widget requests animation
- [x] Idle callbacks run only when no events are pending
- [x] Emits the corresponding `EngineEvent` (9.10)

### Step 23: Backend crates: keep SDL out of the core and the widgets

**Issue:** [#134](https://github.com/KenSuenobu/rust-pushrod/issues/134) `pushrod: [10.6] Backend crates: keep SDL out of the core and the widgets`

**Delivers:** Move everything SDL-specific into a `pushrod-sdl2` backend crate, so `pushrod` and `pushrod-widgets` have no SDL dependency and a future SDL3 backend is a drop-in swap.

**Builds on:** step 5, step 4, step 1.

**Show on video:**

- [x] `cargo tree -p pushrod` and `cargo tree -p pushrod-widgets` show no `sdl2`
- [x] No SDL type in any public signature of the core or the widgets
- [x] `pushrod-sdl2` reproduces today's rendering (golden images unchanged)
- [x] Tests run on the headless backend without a display

### Step 24: Errors: return Result instead of panicking

**Issue:** [#138](https://github.com/KenSuenobu/rust-pushrod/issues/138) `pushrod: [1.25] Errors: return Result instead of panicking`

**Delivers:** Library code reports failures (fonts, images, windows, unknown widget IDs) as `Result` values instead of panicking.

**Builds on:** step 8.

**Show on video:**

- [x] No `unwrap`/`expect` in library code (lints enforced)
- [x] Missing font or image returns an error, not a panic
- [x] Error event delivered for failures during the run loop

### Step 25: Fonts: shared font handles and a built-in default font

**Issue:** [#137](https://github.com/KenSuenobu/rust-pushrod/issues/137) `pushrod: [1.24] Fonts: shared font handles and a built-in default font`

**Delivers:** Replace per-widget font file paths with shared `Font` handles, and ship a default font so applications work without any font files.

**Builds on:** step 14.

**Show on video:**

- [x] Quick start runs from any directory with no font files
- [x] Widgets accept a `Font` handle; no file paths in widget constructors
- [x] Embedded font's license included and noted in the docs

### Step 26: Public API polish before publishing: builders, prelude, and stability

**Issue:** [#136](https://github.com/KenSuenobu/rust-pushrod/issues/136) `pushrod: [1.23] Public API polish before publishing: builders, prelude, and stability`

**Delivers:** Make the public API pleasant and stable before crates.io: builders with defaults, preludes, non-exhaustive enums, and consistent names.

**Builds on:** step 4, step 1, step 24, step 25.

**Show on video:**

- [x] No public constructor needs more than three required arguments
- [x] Public enums are `#[non_exhaustive]`
- [x] Quick start uses the preludes
- [x] Naming decision recorded in the roadmap's Naming section

### Step 27: Custom paint widget (closure-drawn)

**Issue:** [#36](https://github.com/KenSuenobu/rust-pushrod/issues/36) `pushrod: [2.10] Custom paint widget (closure-drawn)`

**Delivers:** One-off custom drawing without writing a widget struct.

**Builds on:** nothing earlier; can start immediately.

**Mockups:** [2.10-1.png](../mockups/2.10-1.png)

**Show on video:**

- [x] Closure paints into the texture
- [x] Optional event closure receives events
- [x] Example draws a chart

### Step 28: Boxed label widget

**Issue:** [#27](https://github.com/KenSuenobu/rust-pushrod/issues/27) `pushrod: [2.1] Boxed label widget`

**Delivers:** Text with background and border.

**Builds on:** nothing earlier; can start immediately.

**Mockups:** [2.1-1.png](../mockups/2.1-1.png)

**Show on video:**

- [x] Renders background, centered text, border
- [x] Display only (empty event mask)

### Step 29: Separator line

**Issue:** [#83](https://github.com/KenSuenobu/rust-pushrod/issues/83) `pushrod: [2.14] Separator line`

**Delivers:** Horizontal or vertical divider line.

**Builds on:** nothing earlier; can start immediately.

**Mockups:** [2.14-1.png](../mockups/2.14-1.png)

**Show on video:**

- [x] Horizontal and vertical render correctly

### Step 30: Group box (titled frame)

**Issue:** [#41](https://github.com/KenSuenobu/rust-pushrod/issues/41) `pushrod: [3.3] Group box (titled frame)`

**Delivers:** A frame with a title in its top border, grouping related controls.

**Builds on:** nothing earlier; can start immediately.

**Mockups:** [3.3-1.png](../mockups/3.3-1.png)

**Show on video:**

- [x] Title interrupts the border
- [x] Works as the parent of a radio group

### Step 31: Image widget

**Issue:** [#33](https://github.com/KenSuenobu/rust-pushrod/issues/33) `pushrod: [2.7] Image widget`

**Delivers:** Display a PNG/BMP image centered, stretched, scaled to fit, or tiled.

**Builds on:** nothing earlier; can start immediately.

**Mockups:** [2.7-1.png](../mockups/2.7-1.png)

**Show on video:**

- [x] Loads PNG and BMP
- [x] Fit modes render as documented
- [x] Missing files don't panic

### Step 32: Glyph button (small single-character button)

**Issue:** [#28](https://github.com/KenSuenobu/rust-pushrod/issues/28) `pushrod: [2.2] Glyph button (small single-character button)`

**Delivers:** A small box with an arrow or symbol, used for scroll arrows, close boxes, and spin buttons.

**Builds on:** step 6, step 22.

**Mockups:** [2.2-1.png](../mockups/2.2-1.png)

**Show on video:**

- [x] Centered glyph
- [x] Pressed state inverts
- [x] Activate-on-press with auto-repeat

### Step 33: Button behaviors: default, cancel, toggle, activate-on-press, dismiss dialog

**Issue:** [#29](https://github.com/KenSuenobu/rust-pushrod/issues/29) `pushrod: [2.3] Button behaviors: default, cancel, toggle, activate-on-press, dismiss dialog`

**Delivers:** A push button that runs an action when clicked; it can also be a dialog's default or cancel button, a toggle, or a button that closes a dialog.

**Builds on:** step 6, step 11.

**Mockups:** [2.3-1.png](../mockups/2.3-1.png), [2.3-2.png](../mockups/2.3-2.png), [2.3-3.png](../mockups/2.3-3.png)

**Show on video:**

- [x] Each flag combination behaves as described and is tested
- [x] Default button is recognizable at a glance
- [x] No regression in PR #10 click tests

### Step 34: Check box with indeterminate state

**Issue:** [#30](https://github.com/KenSuenobu/rust-pushrod/issues/30) `pushrod: [2.4] Check box with indeterminate state`

**Delivers:** Box with a check mark and label; toggles checked; optional mixed state.

**Builds on:** step 6, step 11.

**Mockups:** [2.4-1.png](../mockups/2.4-1.png)

**Show on video:**

- [x] Click and Space toggle
- [x] Mnemonic toggle (1.12 (#76))
- [x] Indeterminate renders and resolves on click
- [x] Label click toggles

### Step 35: Radio buttons and radio groups

**Issue:** [#31](https://github.com/KenSuenobu/rust-pushrod/issues/31) `pushrod: [2.5] Radio buttons and radio groups`

**Delivers:** One-of-many selection among sibling widgets.

**Builds on:** step 6, step 11.

**Mockups:** [2.5-1.png](../mockups/2.5-1.png)

**Show on video:**

- [x] Exactly one exclusive sibling is checked after any change
- [x] Works for any widget with `EXCLUSIVE`
- [x] Arrow keys move within the group

### Step 36: Progress bar

**Issue:** [#48](https://github.com/KenSuenobu/rust-pushrod/issues/48) `pushrod: [4.1] Progress bar`

**Delivers:** A bar showing how far along a task is; with no value it shows a busy animation (like HTML's progress element).

**Builds on:** step 22.

**Mockups:** [4.1-1.png](../mockups/4.1-1.png)

**Show on video:**

- [x] Determinate fill proportional to value / max
- [x] `None` value shows the indeterminate animation without callbacks
- [x] Static in outline drawing mode

### Step 37: Slider

**Issue:** [#40](https://github.com/KenSuenobu/rust-pushrod/issues/40) `pushrod: [3.2] Slider`

**Delivers:** Horizontal/vertical slider with range, optional ticks, and value label.

**Builds on:** step 11.

**Mockups:** [3.2-1.png](../mockups/3.2-1.png)

**Show on video:**

- [x] Drag/click set value within range
- [x] Ticks and snapping
- [x] Keyboard adjusts

### Step 38: Text field: single-line entry with input masks, validation, and numeric mode

**Issue:** [#32](https://github.com/KenSuenobu/rust-pushrod/issues/32) `pushrod: [2.6] Text field: single-line entry with input masks, validation, and numeric mode`

**Delivers:** Single-line text entry with caret, selection, clipboard, input mask, validator, and numeric mode.

**Builds on:** step 18, step 22, step 11.

**Mockups:** [2.6-1.png](../mockups/2.6-1.png), [2.6-2.png](../mockups/2.6-2.png), [2.6-3.png](../mockups/2.6-3.png)

**Show on video:**

- [x] Typing, deleting, caret movement, selection, copy/paste
- [x] Masks and validators reject invalid input
- [x] Numeric mode with range
- [x] `EditingFinished` on Return and focus loss
- [x] Leading/trailing accessories inside the edit area; text never overlaps them
- [x] `Clear`, `Search`, `RevealPassword`, and custom accessories; `TextField::search()` preset

### Step 39: Clip children to their parent's bounds

**Issue:** [#56](https://github.com/KenSuenobu/rust-pushrod/issues/56) `pushrod: [5.1] Clip children to their parent's bounds`

**Delivers:** Children never draw or receive input outside their parent.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [x] Partly outside children are cropped
- [x] Hit testing ignores cropped parts
- [x] Golden test

### Step 40: Composite widgets: build widgets from parts that applications can retrieve and modify

**Issue:** [#57](https://github.com/KenSuenobu/rust-pushrod/issues/57) `pushrod: [5.2] Composite widgets: build widgets from parts that applications can retrieve and modify`

**Delivers:** A Composite trait whose parts (labels, buttons, icons, rows) are real widgets that applications can fetch, restyle, add, and remove.

**Builds on:** step 16, step 11.

**Mockups:** [5.2-1.png](../mockups/5.2-1.png)

**Show on video:**

- [x] Composites create parts as ordinary widgets with IDs
- [x] `parts()`, `part(name)`, and typed part accessors exist on the `Composite` trait, with an example composite
- [x] Changing a part's font, color, or text through `ui.get_mut` repaints it
- [x] List-structured composites support add/insert/remove of parts with automatic re-layout
- [x] Composite translates part events; callbacks on part IDs also work
- [ ] Deferred: the scroll bar is built on it in step 44 and the alert in step 52, so typed accessors on every built-in composite are completed there
- [ ] Deferred: form round trip of part overrides and added/removed parts is completed in step 65 (form writer), once forms exist

### Step 41: Layout manager foundation: Layout trait, size hints, measure and arrange

**Issue:** [#116](https://github.com/KenSuenobu/rust-pushrod/issues/116) `pushrod: [5.5] Layout manager foundation: Layout trait, size hints, measure and arrange`

**Delivers:** A small Layout trait any container can use; widgets report minimum, preferred, and maximum sizes; containers measure then arrange their children.

**Builds on:** step 8, step 13.

**Show on video:**

- [x] `SizeHint`, `LayoutParams`, and the `Layout` trait
- [x] Containers with a layout are measured and arranged once per frame when dirty
- [x] Layout invalidation stops at containers whose size doesn't change
- [x] Moving without resizing doesn't repaint
- [x] Absolute positioning unchanged when no layout is set
- [x] An application-defined layout works

### Step 42: Row and column layouts

**Issue:** [#117](https://github.com/KenSuenobu/rust-pushrod/issues/117) `pushrod: [5.6] Row and column layouts`

**Delivers:** Arrange children in a horizontal row or vertical column with spacing, margins, alignment, and stretch.

**Builds on:** step 41.

**Mockups:** [5.6-1.png](../mockups/5.6-1.png)

**Show on video:**

- [x] Row and column with spacing, margins, alignment, stretch
- [x] Spacer for right/bottom alignment
- [x] Min/max respected while resizing

### Step 43: Form layout: aligned label and field rows

**Issue:** [#118](https://github.com/KenSuenobu/rust-pushrod/issues/118) `pushrod: [5.7] Form layout: aligned label and field rows`

**Delivers:** Two-column 'Label: field' rows with labels aligned and fields stretching to the available width.

**Builds on:** step 41.

**Mockups:** [5.7-1.png](../mockups/5.7-1.png)

**Show on video:**

- [x] Labels align to the widest label
- [x] Fields stretch with the window
- [x] Narrow mode puts labels above fields
- [x] Label click focuses its field

### Step 44: Scroll bar

**Issue:** [#39](https://github.com/KenSuenobu/rust-pushrod/issues/39) `pushrod: [3.1] Scroll bar`

**Delivers:** Vertical/horizontal scroll bar with arrows and a proportional thumb.

**Builds on:** step 22, step 40, step 32, step 11.

**Mockups:** [3.1-1.png](../mockups/3.1-1.png), [3.1-2.png](../mockups/3.1-2.png)

**Show on video:**

- [x] Drag, arrows with repeat, track paging
- [x] Thumb size reflects proportion
- [x] Built on the step 40 `Composite` trait: `parts()`, `part(name)`, and typed accessors for the arrows and thumb
- [x] Parts are retrievable and modifiable as described

### Step 45: Container: a scrollable viewport onto fixed-size content

**Issue:** [#58](https://github.com/KenSuenobu/rust-pushrod/issues/58) `pushrod: [5.3] Container: a scrollable viewport onto fixed-size content`

**Delivers:** A widget whose children live on a content area with its own size and 0,0 origin; the container's frame is a viewport panned by scroll bars.

**Builds on:** step 40, step 39, step 44.

**Mockups:** [5.3-1.png](../mockups/5.3-1.png), [5.3-2.png](../mockups/5.3-2.png)

**Show on video:**

- [x] Children positioned relative to the content's 0,0; content size independent of the container size
- [x] Resizing the container changes only the visible area; moving it doesn't shift content
- [x] Scroll bars appear when needed (or always/never); wheel and keyboard pan; offset clamped to content bounds
- [x] `scroll_to`, `scroll_by`, `ensure_visible`, `viewport()`; focused widgets scrolled into view
- [x] Children clipped to and hit-tested through the viewport; panning doesn't repaint unchanged children
- [ ] Content size modes Fixed, FitChildren, FollowWidth; works with a layout manager and with a loaded form (modes and layout manager done in #58; a loaded form waits for the form loader, 6.4 (#63))
- [x] `ViewportChanged` emitted on pan and resize
- [x] Content larger than the GPU's maximum texture size works
- [x] Only visible children are drawn

### Step 46: Golden-image rendering tests

**Issue:** [#70](https://github.com/KenSuenobu/rust-pushrod/issues/70) `pushrod: [7.2] Golden-image rendering tests`

**Delivers:** Render each widget and state and compare with reference PNGs.

**Builds on:** step 2.

**Show on video:**

- [x] Goldens for every widget and state
- [x] Runs in CI
- [x] Diffs uploaded on failure

### Step 47: Performance benchmarks and budgets

**Issue:** [#115](https://github.com/KenSuenobu/rust-pushrod/issues/115) `pushrod: [7.7] Performance benchmarks and budgets`

**Delivers:** Measure frame time, input latency, memory per widget, and startup; fail CI on regressions.

**Builds on:** step 2.

**Show on video:**

- [x] Benchmarks run in CI with a stored baseline
- [x] Regressions beyond tolerance fail or warn
- [x] README shows current numbers

### Step 48: High-DPI rendering and window resize

**Issue:** [#25](https://github.com/KenSuenobu/rust-pushrod/issues/25) `pushrod: [1.10] High-DPI rendering and window resize`

**Delivers:** The root widget follows the window size; crisp rendering on high-DPI displays.

**Builds on:** step 5, step 41.

**Show on video:**

- [x] Sharp text and lines at 1x, 1.5x, and 2x
- [x] Moving a window between displays re-renders at the new scale
- [x] Resizing re-runs layouts and respects minimum size

### Step 49: Easy installation: bundled SDL, static linking, and platform guides

**Issue:** [#139](https://github.com/KenSuenobu/rust-pushrod/issues/139) `pushrod: [7.9] Easy installation: bundled SDL, static linking, and platform guides`

**Delivers:** Make installing Pushrod painless on Linux, macOS, and Windows, including building SDL from source.

**Builds on:** step 2.

**Show on video:**

- [x] CI builds with `bundled` on macOS and Windows
- [x] Installation pages for Linux, macOS, and Windows tested from a clean machine
- [ ] With 10.6 complete, `bundled` needs no system libraries

### Step 50: Edit the widget tree at run time: remove, reparent, reorder

**Issue:** [#23](https://github.com/KenSuenobu/rust-pushrod/issues/23) `pushrod: [1.8] Edit the widget tree at run time: remove, reparent, reorder`

**Delivers:** Remove widgets and change parent or stacking order without breaking IDs.

**Builds on:** step 4.

**Show on video:**

- [x] Remaining IDs never change
- [x] Removed textures are destroyed
- [x] Focus/hover/capture cleared safely
- [x] Tests for remove, reparent, reorder
- [x] Emits the corresponding `EngineEvent` (9.10)

### Step 51: Modal dialogs: run a form until a dismissing control is chosen

**Issue:** [#66](https://github.com/KenSuenobu/rust-pushrod/issues/66) `pushrod: [6.7] Modal dialogs: run a form until a dismissing control is chosen`

**Delivers:** Open a form modally; the result is the ID of the control that dismissed it.

**Builds on:** step 18, step 50, step 33.

**Show on video:**

- [x] Input blocked outside the top modal
- [x] Dismissing control reported
- [x] Nested modals
- [x] Emits the corresponding `EngineEvent` (9.10)

### Step 52: Alert dialog

**Issue:** [#35](https://github.com/KenSuenobu/rust-pushrod/issues/35) `pushrod: [2.9] Alert dialog`

**Delivers:** One-call modal alert with icon, message, and up to three buttons.

**Builds on:** step 40, step 51, step 33.

**Mockups:** [2.9-1.png](../mockups/2.9-1.png), [2.9-3.png](../mockups/2.9-3.png)

**Show on video:**

- [x] Modal: nothing else receives input while open
- [x] Return/Escape behave as described
- [x] Completion closure gets the index
- [x] Built on the step 40 `Composite` trait: `AlertParts`, plus `add_button` for extra buttons
- [x] Parts are retrievable and modifiable as described

### Step 53: Reference application: a real app built only with Pushrod

**Issue:** [#140](https://github.com/KenSuenobu/rust-pushrod/issues/140) `pushrod: [7.10] Reference application: a real app built only with Pushrod`

**Delivers:** A small but complete application, maintained in the repository, that exercises the MVP together and surfaces API problems early.

**Builds on:** step 38, step 45, step 52, step 42, step 48.

**Show on video:**

- [x] Runs on Linux, macOS, and Windows
- [x] Fully usable with the keyboard alone
- [ ] v0.2 version loads its window from a form file
- [x] Golden images in CI

### Step 54: Guide: writing widgets and composites

**Issue:** [#72](https://github.com/KenSuenobu/rust-pushrod/issues/72) `pushrod: [7.4] Guide: writing widgets and composites`

**Delivers:** Step-by-step docs for extending Pushrod.

**Builds on:** step 40, step 17.

**Show on video:**

- [x] Compiling examples for each section
- [x] Linked from README and crate docs

### Step 55: Changelog, Cargo metadata, and crates.io release

**Issue:** [#73](https://github.com/KenSuenobu/rust-pushrod/issues/73) `pushrod: [7.5] Changelog, Cargo metadata, and crates.io release`

**Delivers:** Publish the modernized library (answers #1 and #8).

**Builds on:** every earlier step in this part (it is the release).

**Show on video:**

- [x] Metadata correct
- [x] Changelog complete
- [x] docs.rs builds

### Step 56: Checkpoint: Pushrod v0.1 released

**Video:** Publish `pushrod`, `pushrod-sdl2`, and `pushrod-widgets` v0.1 to crates.io; walk through the reference application and the documentation site.

## Part 2: Pushrod v0.2, forms and resources

Forms, form files, and resources by ID (the runtime foundation for Chassis), plus input methods, mnemonics, accessibility, and specs. Completes the Pushrod MVP.

### Step 57: Module boundaries and written specs with conformance tests

**Issue:** [#114](https://github.com/KenSuenobu/rust-pushrod/issues/114) `pushrod: [7.6] Module boundaries and written specs with conformance tests`

**Delivers:** Specify each module's contract (widget, events, paint, form format) in docs/specs and test against it.

**Builds on:** step 4, step 2.

**Show on video:**

- [x] Specs for widget, events, paint, and forms
- [x] Engine and design layer separated in module layout
- [x] Conformance tests for each stated guarantee run in CI

### Step 58: Input method (IME) support: composition text and candidate window

**Issue:** [#112](https://github.com/KenSuenobu/rust-pushrod/issues/112) `pushrod: [1.19] Input method (IME) support: composition text and candidate window`

**Delivers:** Show in-progress composition text in fields and position the platform candidate window at the caret.

**Builds on:** step 18, step 13.

**Mockups:** [1.19-1.png](../mockups/1.19-1.png)

**Show on video:**

- [ ] Japanese and Chinese input methods compose and commit correctly in a text field
- [ ] Candidate window appears at the caret and follows it
- [x] Composition never fires `TextChanged` until committed

### Step 59: Keyboard mnemonics: underlined shortcut letters

**Issue:** [#76](https://github.com/KenSuenobu/rust-pushrod/issues/76) `pushrod: [1.12] Keyboard mnemonics: underlined shortcut letters`

**Delivers:** Alt+letter (or the letter alone in dialogs) activates a control, shown by an underlined character.

**Builds on:** step 18.

**Mockups:** [1.12-1.png](../mockups/1.12-1.png)

**Show on video:**

- [x] Markers are underlined, not drawn
- [x] Alt+letter activates buttons and toggles check boxes; labels focus the next field
- [x] Scoped to the topmost modal dialog
- [x] Duplicate detection in debug builds

### Step 60: Widget registry and factory

**Issue:** [#60](https://github.com/KenSuenobu/rust-pushrod/issues/60) `pushrod: [6.1] Widget registry and factory`

**Delivers:** Create widgets by type name, the prerequisite for loading forms.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [x] All built-ins registered
- [x] Custom widgets registrable
- [x] Unknown names are errors, not panics

### Step 61: Property introspection: list, get, and set widget properties by name

**Issue:** [#61](https://github.com/KenSuenobu/rust-pushrod/issues/61) `pushrod: [6.2] Property introspection: list, get, and set widget properties by name`

**Delivers:** Each widget describes its properties for saving, loading, bindings, and the Chassis inspector.

**Builds on:** step 6, step 60.

**Show on video:**

- [x] Every built-in lists and sets all properties
- [x] Round-trip test
- [x] `PropertyChanged` emitted on change

### Step 62: Accessibility: expose widgets to screen readers

**Issue:** [#80](https://github.com/KenSuenobu/rust-pushrod/issues/80) `pushrod: [1.14] Accessibility: expose widgets to screen readers`

**Delivers:** Publish roles, names, states, and actions of widgets to the platform accessibility APIs.

**Builds on:** step 6, step 61, step 18.

**Show on video:**

- [ ] A screen reader announces focused buttons, check boxes, and text fields with name and state
- [ ] Accessibility actions trigger the same callbacks as user input
- [ ] Updates are incremental (changed nodes only) and the tree is only built when assistive technology is active

### Step 63: Form file format (binary, versioned)

**Issue:** [#62](https://github.com/KenSuenobu/rust-pushrod/issues/62) `pushrod: [6.3] Form file format (binary, versioned)`

**Delivers:** The binary file that stores forms: object table by ID, string and image pools, symbols.

**Builds on:** step 61, step 60.

**Show on video:**

- [ ] Spec merged
- [ ] Reader/writer with tests on hand-built bytes
- [ ] Unknown property tags skipped
- [ ] Versioned

### Step 64: Form loader: build a form from a file with stable IDs

**Issue:** [#63](https://github.com/KenSuenobu/rust-pushrod/issues/63) `pushrod: [6.4] Form loader: build a form from a file with stable IDs`

**Delivers:** Load a form and attach callbacks by ID or name.

**Builds on:** step 63.

**Show on video:**

- [ ] Loaded form renders identically to the code-built version (golden)
- [ ] IDs resolve via generated constants
- [ ] Errors name object and property
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 65: Form writer: save a widget subtree to a file

**Issue:** [#64](https://github.com/KenSuenobu/rust-pushrod/issues/64) `pushrod: [6.5] Form writer: save a widget subtree to a file`

**Delivers:** Serialize any subtree into the form format.

**Builds on:** step 63.

**Show on video:**

- [ ] Round trip preserves properties and pixels
- [ ] Composite part overrides and added/removed parts round trip (completes step 40)

### Step 66: Generate ID constants from a form file

**Issue:** [#65](https://github.com/KenSuenobu/rust-pushrod/issues/65) `pushrod: [6.6] Generate ID constants from a form file`

**Delivers:** build.rs helper and CLI that emit Rust constants for named objects.

**Builds on:** step 64, step 63.

**Show on video:**

- [ ] build.rs integration used by the form example
- [ ] Deterministic, rustfmt-clean output

### Step 67: Resource kinds: strings, images, icons, files, menus, and alerts

**Issue:** [#154](https://github.com/KenSuenobu/rust-pushrod/issues/154) `pushrod: [6.11] Resource kinds: strings, images, icons, files, menus, and alerts`

**Delivers:** Extend the resource file format beyond forms: string tables, images, icons, arbitrary files, menus, and alerts, each with a stable ID.

**Builds on:** step 63.

**Show on video:**

- [ ] Round trip (write, read) for every kind
- [ ] IDs stay stable when resources are added, removed, or reordered
- [ ] Older readers skip unknown kinds
- [ ] `docs/FORM_FORMAT.md` documents every table

### Step 68: Typed resource IDs: generated constants per resource kind

**Issue:** [#155](https://github.com/KenSuenobu/rust-pushrod/issues/155) `pushrod: [6.12] Typed resource IDs: generated constants per resource kind`

**Delivers:** Generate typed ID constants for every resource kind and for the widgets inside each form, so code refers to resources by name.

**Builds on:** step 67, step 66.

**Show on video:**

- [ ] Constants generated for every kind and every named widget
- [ ] Passing an ID of the wrong kind is a compile error
- [ ] Generated output identical across runs

### Step 69: Resource manager: load resource sets and get resources by ID

**Issue:** [#153](https://github.com/KenSuenobu/rust-pushrod/issues/153) `pushrod: [6.10] Resource manager: load resource sets and get resources by ID`

**Delivers:** Load a resource file (from disk or embedded in the executable) and get its forms, strings, images, icons, files, menus, and alerts by ID.

**Builds on:** step 24, step 67, step 64, step 63.

**Show on video:**

- [ ] Load from a file and from embedded bytes; same results
- [ ] Every kind retrievable by typed ID; wrong-kind IDs fail to compile
- [ ] Stacked sets resolve in order; an application set overrides a library set
- [ ] Images upload once and are shared by every widget that uses them
- [ ] Malformed files return an error, never panic

### Step 70: Checkpoint: Pushrod v0.2 released: the Pushrod MVP is complete

**Video:** Publish v0.2; show a form designed in a resource file loading by ID with images, icons, and strings.

## Part 3: Pushrod Showcase, the interactive demo of every widget

One application that shows every widget and core feature live, with a property panel, an event log, and source code. From here on, every step that adds a widget also adds its Showcase page, so each video can end with the new widget running in the Showcase.

### Step 71: Showcase application shell

**Issue:** [#295](https://github.com/KenSuenobu/rust-pushrod/issues/295) `pushrod: [12.1] Showcase application shell`

**Delivers:** The showcase window: a searchable list of pages on the left and the selected page on the right.

**Builds on:** step 42, step 38, step 15, step 45.

**Mockups:** [12.1-1.png](../mockups/12.1-1.png), [12.1-2.png](../mockups/12.1-2.png)

**Show on video:**

- [ ] `cargo run -p pushrod-showcase` opens the shell with grouped pages
- [ ] Search filters the page list
- [ ] Adding a page needs only an implementation of `Page` and one registration line
- [ ] Gallery example content available as pages

### Step 72: Showcase pages for every standard widget, with a completeness test

**Issue:** [#296](https://github.com/KenSuenobu/rust-pushrod/issues/296) `pushrod: [12.2] Showcase pages for every standard widget, with a completeness test`

**Delivers:** One page per standard widget showing its states and variations, and a test that fails when a widget has no page.

**Builds on:** step 60, step 71.

**Mockups:** [12.2-1.png](../mockups/12.2-1.png)

**Show on video:**

- [ ] Every registered standard widget has a page
- [ ] The completeness test fails when a registered type has no page
- [ ] CONTRIBUTING.md states the rule

### Step 73: Showcase property panel: change any property live

**Issue:** [#297](https://github.com/KenSuenobu/rust-pushrod/issues/297) `pushrod: [12.3] Showcase property panel: change any property live`

**Delivers:** A panel listing the selected demo widget's properties; editing one updates the widget immediately.

**Builds on:** step 61, step 72.

**Mockups:** [12.3-1.png](../mockups/12.3-1.png)

**Show on video:**

- [ ] Selecting a demo widget lists its properties
- [ ] Edits apply immediately; Reset restores
- [ ] Works for any registered widget without per-widget code

### Step 74: Showcase event log

**Issue:** [#298](https://github.com/KenSuenobu/rust-pushrod/issues/298) `pushrod: [12.4] Showcase event log`

**Delivers:** A panel that lists every event the demo widgets emit, as it happens.

**Builds on:** step 8, step 71.

**Mockups:** [12.4-1.png](../mockups/12.4-1.png)

**Show on video:**

- [ ] Widget events appear as they happen with ID and payload
- [ ] Engine events can be switched on
- [ ] Filter, clear, and pause

### Step 75: Showcase source view: the code for each page

**Issue:** [#299](https://github.com/KenSuenobu/rust-pushrod/issues/299) `pushrod: [12.5] Showcase source view: the code for each page`

**Delivers:** Each page shows the code that builds it, checked by the compiler, with a Copy button.

**Builds on:** step 21, step 72.

**Mockups:** [12.5-1.png](../mockups/12.5-1.png)

**Show on video:**

- [ ] Every page shows its real source
- [ ] Copy places it on the clipboard
- [ ] Documentation pages include the same snippets

### Step 76: Showcase pages for core features: text, layouts, focus, dialogs, forms, and resources

**Issue:** [#300](https://github.com/KenSuenobu/rust-pushrod/issues/300) `pushrod: [12.6] Showcase pages for core features: text, layouts, focus, dialogs, forms, and resources`

**Delivers:** Pages that demonstrate what isn't a widget: text, layouts, focus and keys, modal dialogs, forms loaded from files, and resources by ID.

**Builds on:** step 43, step 69, step 51, step 71.

**Mockups:** [12.6-1.png](../mockups/12.6-1.png)

**Show on video:**

- [ ] A page for each MVP core feature listed
- [ ] Layout pages reflow when resized
- [ ] The forms page loads a real resource file

### Step 77: Showcase in CI: scripted tour, golden screenshots, and documentation images

**Issue:** [#301](https://github.com/KenSuenobu/rust-pushrod/issues/301) `pushrod: [12.7] Showcase in CI: scripted tour, golden screenshots, and documentation images`

**Delivers:** CI opens every page, compares it with its golden image, and publishes the screenshots the documentation uses.

**Builds on:** step 46, step 72.

**Show on video:**

- [ ] The tour visits every page and compares golden images in CI
- [ ] Documentation screenshots are generated, not hand-made
- [ ] Differences are uploaded as artifacts

### Step 78: Checkpoint: Pushrod Showcase released

**Video:** Tour the Showcase: search for a widget, change its properties, watch the event log, read the source, and run the CI tour.

## Part 4: Chassis M1: core without UI

Project model, resource access, code generation, and a command-line generator, all testable without a window. Can start as soon as Pushrod's resource manager (6.10 to 6.12) lands. Pushrod work and extra widgets this phase needs come first.

### Step 79: Event metadata in the widget registry

**Issue:** [#156](https://github.com/KenSuenobu/rust-pushrod/issues/156) `pushrod: [6.13] Event metadata in the widget registry`

**Delivers:** Each registered widget type declares the events it emits, its default event, and each event's payload, so tools like Chassis can list them.

**Builds on:** step 4, step 60.

**Show on video:**

- [ ] Every standard widget declares its events and default event
- [ ] Declared events match emitted events (test)
- [ ] Third-party widget libraries can declare theirs

### Step 80: Project model

**Issue:** [#164](https://github.com/KenSuenobu/rust-pushrod/issues/164) `chassis: [2.1] Project model`

**Delivers:** A project is a Cargo crate, its resource file, its handler files, and its generated files.

**Builds on:** step 69, step 67, step 68.

**Show on video:**

- [ ] Loads a project directory and reports its forms, resources, and handlers
- [ ] Ownership rules documented and enforced (Chassis writes only generated files and stubs)
- [ ] Unit tests cover the model without a window

### Step 81: Code generation rules

**Issue:** [#165](https://github.com/KenSuenobu/rust-pushrod/issues/165) `chassis: [6.1] Code generation rules`

**Delivers:** What Chassis writes, when, and what it never touches.

**Builds on:** step 80.

**Show on video:**

- [ ] Tests prove handler bodies survive every designer operation
- [ ] Rules documented

### Step 82: Event wiring generation

**Issue:** [#166](https://github.com/KenSuenobu/rust-pushrod/issues/166) `chassis: [5.7] Event wiring generation`

**Delivers:** Generated code registers each handler with the engine by widget ID.

**Builds on:** step 80, step 68, step 79.

**Show on video:**

- [ ] A project with handlers builds and every handler fires
- [ ] Deterministic output (same input, same output)

### Step 83: Command-line generator

**Issue:** [#167](https://github.com/KenSuenobu/rust-pushrod/issues/167) `chassis: [6.5] Command-line generator`

**Delivers:** `chassis generate` regenerates IDs and wiring without opening the UI.

**Builds on:** step 80, step 82, step 81.

**Show on video:**

- [ ] Output identical to what the UI generates
- [ ] `chassis check` exits non-zero on problems

### Step 84: Checkpoint: Chassis M1: core without UI complete

**Video:** Run `chassis generate` and `chassis check` on a project; show the generated typed IDs and wiring, and the tests that prove handler bodies are never touched.

## Part 5: Chassis M2: shell and project

The main window, three panes, Designer/Code tabs, commands, and the project drawer with resource editors. Pushrod work and extra widgets this phase needs come first.

### Step 85: Actions and keyboard shortcuts: shared commands for menus, toolbars, and buttons

**Issue:** [#97](https://github.com/KenSuenobu/rust-pushrod/issues/97) `pushrod: [9.5] Actions and keyboard shortcuts: shared commands for menus, toolbars, and buttons`

**Delivers:** One action object drives every menu item, toolbar button, and shortcut that performs the same command, keeping enabled/checked state in sync.

**Builds on:** step 4, step 11.

**Show on video:**

- [ ] A Bold action toggled from the toolbar checks the Bold menu item
- [ ] Disabling an action disables every control
- [ ] Shortcuts trigger actions; platform-standard keys map correctly
- [ ] Exclusive action groups
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 86: Drag and drop within the application

**Issue:** [#100](https://github.com/KenSuenobu/rust-pushrod/issues/100) `pushrod: [9.8] Drag and drop within the application`

**Delivers:** Drag data between widgets (list to list, file icon to drop zone) with drop targets and feedback.

**Builds on:** step 17, step 39.

**Mockups:** [9.8-1.png](../mockups/9.8-1.png)

**Show on video:**

- [ ] Drag between two lists
- [ ] Drop highlighting and cancel
- [ ] OS file drops delivered
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 87: Undo and redo command stack

**Issue:** [#81](https://github.com/KenSuenobu/rust-pushrod/issues/81) `pushrod: [1.15] Undo and redo command stack`

**Delivers:** An application-level undo stack that text fields and actions can use.

**Builds on:** step 85.

**Show on video:**

- [ ] Undo/redo of text edits and application commands
- [ ] Consecutive typing merges into one undo step
- [ ] Undo/Redo actions enable and disable automatically

### Step 88: Autoresizing (keep margins or stretch on resize)

**Issue:** [#59](https://github.com/KenSuenobu/rust-pushrod/issues/59) `pushrod: [5.4] Autoresizing (keep margins or stretch on resize)`

**Delivers:** Children keep margins or stretch when their parent is resized.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [ ] Children follow their anchors on parent resize
- [ ] Stored in form files

### Step 89: List view

**Issue:** [#52](https://github.com/KenSuenobu/rust-pushrod/issues/52) `pushrod: [4.5] List view`

**Delivers:** Scrolling list with single/multiple selection, backed by a data source.

**Builds on:** step 40, step 45, step 11.

**Mockups:** [4.5-1.png](../mockups/4.5-1.png)

**Show on video:**

- [ ] Smooth with 10,000 rows
- [ ] Selection modes and keyboard navigation
- [ ] Model change notifications update the view
- [ ] Parts are retrievable and modifiable as described
- [ ] Range-based model notifications; row widgets recycled while scrolling

### Step 90: Tab view

**Issue:** [#51](https://github.com/KenSuenobu/rust-pushrod/issues/51) `pushrod: [4.4] Tab view`

**Delivers:** Tabs that switch between pages of controls.

**Builds on:** step 6, step 40.

**Mockups:** [4.4-1.png](../mockups/4.4-1.png)

**Show on video:**

- [ ] Switching hides/shows pages
- [ ] Hidden controls keep state and IDs
- [ ] Ctrl+Tab
- [ ] Parts are retrievable and modifiable as described

### Step 91: Table view

**Issue:** [#92](https://github.com/KenSuenobu/rust-pushrod/issues/92) `pushrod: [4.9] Table view`

**Delivers:** Multi-column list with headers, column resizing, and sorting.

**Builds on:** step 40, step 89.

**Mockups:** [4.9-1.png](../mockups/4.9-1.png)

**Show on video:**

- [ ] Columns resize and sort
- [ ] Only visible cells painted
- [ ] Parts are retrievable and modifiable as described

### Step 92: Expander (collapsible section)

**Issue:** [#90](https://github.com/KenSuenobu/rust-pushrod/issues/90) `pushrod: [3.10] Expander (collapsible section)`

**Delivers:** A titled section that shows or hides its contents when clicked.

**Builds on:** step 6, step 88.

**Mockups:** [3.10-1.png](../mockups/3.10-1.png)

**Show on video:**

- [ ] Click, Space, and mnemonic toggle
- [ ] Children hidden and shown

### Step 93: Menu bar and drop-down menus (drawn by Pushrod)

**Issue:** [#37](https://github.com/KenSuenobu/rust-pushrod/issues/37) `pushrod: [2.11] Menu bar and drop-down menus (drawn by Pushrod)`

**Delivers:** Menu bar with drop-down menus, separators, check marks, submenus, shortcuts, and mnemonics.

**Builds on:** step 39, step 6, step 59, step 85, step 40.

**Mockups:** [2.11-1.png](../mockups/2.11-1.png)

**Show on video:**

- [ ] Menus open, track, select, and close correctly
- [ ] Submenus and keyboard navigation
- [ ] No OS-native menu APIs on any platform
- [ ] Parts are retrievable and modifiable as described
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 94: Image button

**Issue:** [#125](https://github.com/KenSuenobu/rust-pushrod/issues/125) `pushrod: [2.21] Image button`

**Delivers:** A button that shows an image, or an image with a text label, with the full button behavior.

**Builds on:** step 40, step 33, step 31.

**Mockups:** [2.21-1.png](../mockups/2.21-1.png)

**Show on video:**

- [ ] Image only, and image with label in each position
- [ ] Per-state images or automatic pressed/disabled looks
- [ ] All button behaviors and mnemonics work

### Step 95: Toolbar

**Issue:** [#84](https://github.com/KenSuenobu/rust-pushrod/issues/84) `pushrod: [2.15] Toolbar`

**Delivers:** A row of icon/text buttons bound to actions.

**Builds on:** step 40, step 94, step 85.

**Mockups:** [2.15-1.png](../mockups/2.15-1.png)

**Show on video:**

- [ ] Buttons reflect action state
- [ ] Overflow menu when too narrow
- [ ] Parts are retrievable and modifiable as described

### Step 96: Status bar

**Issue:** [#85](https://github.com/KenSuenobu/rust-pushrod/issues/85) `pushrod: [2.16] Status bar`

**Delivers:** A strip at the bottom of the window for messages and status fields.

**Builds on:** step 22, step 40.

**Mockups:** [2.16-1.png](../mockups/2.16-1.png)

**Show on video:**

- [ ] Timed messages clear themselves
- [ ] Hovered action status text appears
- [ ] Parts are retrievable and modifiable as described

### Step 97: Icon widget (picture with caption)

**Issue:** [#34](https://github.com/KenSuenobu/rust-pushrod/issues/34) `pushrod: [2.8] Icon widget (picture with caption)`

**Delivers:** Picture with a caption, selectable and double-clickable, like desktop icons.

**Builds on:** step 31, step 11.

**Mockups:** [2.8-1.png](../mockups/2.8-1.png)

**Show on video:**

- [ ] Selection inverts the caption
- [ ] Double-click and Return emit `Activated`

### Step 98: File chooser dialog (drawn by Pushrod)

**Issue:** [#38](https://github.com/KenSuenobu/rust-pushrod/issues/38) `pushrod: [2.12] File chooser dialog (drawn by Pushrod)`

**Delivers:** Choose a file or directory with a consistent look on every platform.

**Builds on:** step 51, step 40, step 38, step 89.

**Mockups:** [2.12-1.png](../mockups/2.12-1.png)

**Show on video:**

- [ ] Navigate, filter, choose; keyboard operable
- [ ] Returns None on cancel
- [ ] Parts are retrievable and modifiable as described

### Step 99: Split view (panes with draggable dividers)

**Issue:** [#47](https://github.com/KenSuenobu/rust-pushrod/issues/47) `pushrod: [3.9] Split view (panes with draggable dividers)`

**Delivers:** Two or more panes side by side or stacked, separated by dividers the user drags to resize them.

**Builds on:** step 40, step 17, step 41.

**Mockups:** [3.9-1.png](../mockups/3.9-1.png)

**Show on video:**

- [ ] Dragging a divider resizes neighboring panes within their min/max
- [ ] Collapsible panes; double-click to collapse/restore
- [ ] Extra space distributed by stretch factor
- [ ] Positions settable and readable; outline mode supported

### Step 100: Widget libraries: extend or replace pushrod-widgets

**Issue:** [#131](https://github.com/KenSuenobu/rust-pushrod/issues/131) `pushrod: [1.22] Widget libraries: extend or replace pushrod-widgets`

**Delivers:** Make it straightforward for a separate crate to add widgets alongside pushrod-widgets, or to replace it with a different look.

**Builds on:** step 1, step 60.

**Show on video:**

- [ ] A separate crate adds a widget that works with IDs, events, callbacks, and focus like a standard one
- [ ] A replacement library registers standard type names and a form loads with its look
- [ ] Docs page explains extending and replacing

### Step 101: Create the pushrod-extra-widgets crate

**Issue:** [#252](https://github.com/KenSuenobu/rust-pushrod/issues/252) `extra-widgets: [1.1] Create the pushrod-extra-widgets crate`

**Delivers:** The crate, its registration function, its docs section, and its place in CI.

**Builds on:** step 1, step 100, step 60, step 23.

**Mockups:** [xw-1.1-1.png](../mockups/xw-1.1-1.png)

**Show on video:**

- [ ] Workspace builds with the new crate; CI runs fmt, clippy, tests, `check_docs`, and rustdoc on it
- [ ] `register` adds widgets under `extra.` and a test loads a form using one
- [ ] Documentation site has an Extra widgets section and the crate appears in Widget libraries

### Step 102: Popover: a floating panel anchored to a control

**Issue:** [#254](https://github.com/KenSuenobu/rust-pushrod/issues/254) `extra-widgets: [1.3] Popover: a floating panel anchored to a control`

**Delivers:** A borderless panel that opens next to a control, holds any widgets, and closes on Escape or a click outside.

**Builds on:** step 101, step 39, step 40, step 16, step 18.

**Mockups:** [xw-1.3-1.png](../mockups/xw-1.3-1.png)

**Show on video:**

- [ ] Placement flips to stay inside the window
- [ ] Escape and an outside click close it; focus returns to the anchor
- [ ] Any widget tree can be the content
- [ ] Modal and modeless modes

### Step 103: Drawer (collapsible side panel)

**Issue:** [#257](https://github.com/KenSuenobu/rust-pushrod/issues/257) `extra-widgets: [5.1] Drawer (collapsible side panel)`

**Delivers:** A side panel with a title bar that collapses to a thin strip and remembers its width.

**Builds on:** step 101, step 99, step 32, step 85.

**Mockups:** [xw-5.1-1.png](../mockups/xw-5.1-1.png)

**Show on video:**

- [ ] Collapses to a strip and restores its width
- [ ] Left, right, and bottom sides
- [ ] Toggle by button and action shortcut

### Step 104: Command palette

**Issue:** [#305](https://github.com/KenSuenobu/rust-pushrod/issues/305) `extra-widgets: [6.9] Command palette`

**Delivers:** A pop-up search box listing every command; type a few letters, press Return, and it runs.

**Builds on:** step 102, step 85, step 38, step 89.

**Mockups:** [xw-6.9-1.png](../mockups/xw-6.9-1.png)

**Show on video:**

- [ ] Lists every action with its shortcut; disabled ones dimmed
- [ ] Fuzzy filtering with matched letters marked; recent first
- [ ] Keyboard only: open, move, run, close
- [ ] Extra sources with a prefix

### Step 105: Chassis crate and main window

**Issue:** [#168](https://github.com/KenSuenobu/rust-pushrod/issues/168) `chassis: [1.1] Chassis crate and main window`

**Delivers:** Binary crate and the main window with menu bar, toolbar, and status bar.

**Builds on:** step 93, step 95, step 96, step 85, step 23.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Window frame matches the mockup (menu bar, toolbar, status bar)
- [ ] Menus open and toolbar buttons trigger their commands (stubs where later issues implement them)
- [ ] Status bar updates with the selection

### Step 106: Three-pane workspace

**Issue:** [#169](https://github.com/KenSuenobu/rust-pushrod/issues/169) `chassis: [1.2] Three-pane workspace`

**Delivers:** Project drawer, editor, and inspector in resizable, collapsible split views.

**Builds on:** step 105, step 99.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Dividers drag and collapse; minimum widths respected
- [ ] F4 toggles the inspector
- [ ] Layout matches the mockup at 1200 x 760 and scales up

### Step 107: Commands, menus, and shortcuts

**Issue:** [#170](https://github.com/KenSuenobu/rust-pushrod/issues/170) `chassis: [1.4] Commands, menus, and shortcuts`

**Delivers:** Every command as an action, reachable from menus, toolbar, and familiar shortcuts.

**Builds on:** step 105, step 85, step 104.

**Show on video:**

- [ ] Every command reachable from menu and shortcut, toolbar where shown
- [ ] Commands disabled when not applicable
- [ ] Shortcut list on the docs site

### Step 108: Designer and Code tabs

**Issue:** [#171](https://github.com/KenSuenobu/rust-pushrod/issues/171) `chassis: [1.3] Designer and Code tabs`

**Delivers:** Two tabs per open form; F12 switches; each keeps its scroll, selection, and caret.

**Builds on:** step 106, step 90.

**Mockups:** [chassis-4.png](../mockups/chassis-4.png)

**Show on video:**

- [ ] F12 and clicking switch tabs
- [ ] Selection, scroll, and caret preserved across switches
- [ ] Opening another form keeps the active tab

### Step 109: New, open, and save

**Issue:** [#172](https://github.com/KenSuenobu/rust-pushrod/issues/172) `chassis: [2.2] New, open, and save`

**Delivers:** Create a project from scratch, open an existing one, save the resource file and generated code.

**Builds on:** step 80, step 107, step 81, step 98, step 65.

**Show on video:**

- [ ] A new project builds and runs (an empty form) without edits
- [ ] Save then reopen restores every resource exactly
- [ ] Closing with unsaved changes asks to save

### Step 110: Project drawer

**Issue:** [#173](https://github.com/KenSuenobu/rust-pushrod/issues/173) `chassis: [2.3] Project drawer`

**Delivers:** Resource sections (Forms, Strings, Images, Icons, Files, Menus) as expanders with lists.

**Builds on:** step 106, step 80, step 92, step 89, step 103.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Sections expand and collapse; counts correct
- [ ] Double-click opens the right editor
- [ ] Selection follows the open form

### Step 111: Manage resources and IDs

**Issue:** [#174](https://github.com/KenSuenobu/rust-pushrod/issues/174) `chassis: [2.4] Manage resources and IDs`

**Delivers:** Add, rename, duplicate, and delete resources; see and copy each resource's ID.

**Builds on:** step 110, step 67, step 68, step 87.

**Show on video:**

- [ ] Renaming a form renames its handler file and generated code; the project still builds
- [ ] Copy ID Constant copies the exact generated name
- [ ] IDs never change when other resources are added or removed

### Step 112: String table editor

**Issue:** [#175](https://github.com/KenSuenobu/rust-pushrod/issues/175) `chassis: [2.5] String table editor`

**Delivers:** Edit the resource file's strings: name, text, and ID.

**Builds on:** step 110, step 69, step 67, step 91.

**Show on video:**

- [ ] Add, edit, delete, search strings
- [ ] Widgets can use a string resource for their text
- [ ] Deleting a used string warns

### Step 113: Images and icons

**Issue:** [#176](https://github.com/KenSuenobu/rust-pushrod/issues/176) `chassis: [2.6] Images and icons`

**Delivers:** Import images and icons into the resource file and use them in widgets by ID.

**Builds on:** step 110, step 69, step 67, step 31, step 97, step 86.

**Show on video:**

- [ ] Import, preview, rename, replace, delete images and icons
- [ ] Widgets reference them by ID
- [ ] Replacing keeps the ID and updates every widget

### Step 114: Files

**Issue:** [#177](https://github.com/KenSuenobu/rust-pushrod/issues/177) `chassis: [2.8] Files`

**Delivers:** Add arbitrary files (license text, templates, data) to the resource file and load them by ID.

**Builds on:** step 110, step 69, step 67.

**Show on video:**

- [ ] Add, rename, replace, export, delete files
- [ ] Text files preview
- [ ] Files load at run time by ID

### Step 115: Checkpoint: Chassis M2: shell and project complete

**Video:** Open Chassis: create a project, browse the project drawer, add strings, images, icons, and files.

## Part 6: Chassis M3: form designer

Design surface, palette, drag and drop, selection, snapping, containers, layouts, and undo. Pushrod work and extra widgets this phase needs come first.

### Step 116: Event filters: observe or intercept another widget's events

**Issue:** [#95](https://github.com/KenSuenobu/rust-pushrod/issues/95) `pushrod: [9.3] Event filters: observe or intercept another widget's events`

**Delivers:** Install a filter that sees a widget's (or the whole application's) events first and can stop them.

**Builds on:** step 16.

**Show on video:**

- [ ] Widget, subtree, and application filters
- [ ] Stop prevents delivery
- [ ] Removal works

### Step 117: Drawing modes: live or outline feedback while moving and resizing

**Issue:** [#105](https://github.com/KenSuenobu/rust-pushrod/issues/105) `pushrod: [10.1] Drawing modes: live or outline feedback while moving and resizing`

**Delivers:** Live mode redraws content as things move; outline mode shows only a dotted rectangle until the move ends. Switchable at run time, with a system-wide event.

**Builds on:** step 8, step 17.

**Mockups:** [10.1-1.png](../mockups/10.1-1.png), [10.1-2.png](../mockups/10.1-2.png)

**Show on video:**

- [ ] Live, Outline, and Auto modes for drag, resize, divider, and column-width interactions
- [ ] Outline mode performs no content repaint until release
- [ ] Value controls defer change events and dependent repaints in outline mode
- [ ] Auto switches based on measured frame time
- [ ] `ui.set_drawing_mode` switches at run time; in-progress interactions finish in their starting mode
- [ ] Each change broadcasts `DrawingModeChanged { old, new, reason }`

### Step 118: Flow layout (wrapping rows)

**Issue:** [#121](https://github.com/KenSuenobu/rust-pushrod/issues/121) `pushrod: [5.10] Flow layout (wrapping rows)`

**Delivers:** Children placed left to right, wrapping to the next row when the width runs out.

**Builds on:** step 41.

**Mockups:** [5.10-1.png](../mockups/5.10-1.png)

**Show on video:**

- [ ] Reflows on resize
- [ ] Height-for-width measurement

### Step 119: Tooltips

**Issue:** [#77](https://github.com/KenSuenobu/rust-pushrod/issues/77) `pushrod: [2.13] Tooltips`

**Delivers:** Short help text shown after the pointer rests on a widget.

**Builds on:** step 22, step 39.

**Mockups:** [2.13-1.png](../mockups/2.13-1.png)

**Show on video:**

- [ ] Appears after the delay and hides correctly
- [ ] Stays inside the window; never under the pointer
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 120: Design-mode hooks for Chassis

**Issue:** [#68](https://github.com/KenSuenobu/rust-pushrod/issues/68) `pushrod: [6.9] Design-mode hooks for Chassis`

**Delivers:** Select, move, resize, and edit widgets without triggering their behavior.

**Builds on:** step 61, step 116, step 87, step 50, step 40.

**Show on video:**

- [ ] No widget reacts in design mode
- [ ] Select/move/resize/reparent, undoable
- [ ] Back to run mode restores behavior

### Step 121: Tool palette (grid of tool buttons)

**Issue:** [#42](https://github.com/KenSuenobu/rust-pushrod/issues/42) `extra-widgets: [2.6] Tool palette (grid of tool buttons)`

**Delivers:** A grid of icon buttons with one-of-many selection and optional drag out, for tools, swatches, or components.

**Builds on:** step 101, step 94, step 119, step 118, step 86.

**Mockups:** [xw-2.6-1.png](../mockups/xw-2.6-1.png)

**Show on video:**

- [ ] Select, Click, and Drag modes
- [ ] Grid and List layouts, group header rows, and tooltips
- [ ] Grid reflows with width; keyboard navigation

### Step 122: Breadcrumb bar (path control)

**Issue:** [#256](https://github.com/KenSuenobu/rust-pushrod/issues/256) `extra-widgets: [4.9] Breadcrumb bar (path control)`

**Delivers:** A row of clickable path segments, with a menu of siblings on each separator.

**Builds on:** step 101, step 33, step 93, step 13.

**Mockups:** [xw-4.9-1.png](../mockups/xw-4.9-1.png)

**Show on video:**

- [ ] Clickable segments and sibling menus
- [ ] Overflow collapses into a menu
- [ ] Keyboard navigation

### Step 123: Canvas view (zoomable surface of items)

**Issue:** [#304](https://github.com/KenSuenobu/rust-pushrod/issues/304) `extra-widgets: [4.14] Canvas view (zoomable surface of items)`

**Delivers:** A scrollable, zoomable surface holding positioned items, with selection, rubber band, and moving.

**Builds on:** step 101, step 45, step 17, step 50, step 27.

**Mockups:** [xw-4.14-1.png](../mockups/xw-4.14-1.png)

**Show on video:**

- [ ] Pan and zoom around the pointer
- [ ] Click, extend, and rubber-band selection; move with snapping
- [ ] Widgets and custom-painted items; connectors between items
- [ ] Only visible items are painted

### Step 124: Design surface

**Issue:** [#178](https://github.com/KenSuenobu/rust-pushrod/issues/178) `chassis: [3.1] Design surface`

**Delivers:** The open form on a dotted grid, in Pushrod design mode.

**Builds on:** step 108, step 80, step 120, step 64, step 123.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Forms look exactly as at run time
- [ ] Widgets don't react in the designer
- [ ] Grid size and visibility adjustable

### Step 125: Widget palette

**Issue:** [#179](https://github.com/KenSuenobu/rust-pushrod/issues/179) `chassis: [3.2] Widget palette`

**Delivers:** Every registered widget type, searchable and grouped, including layouts.

**Builds on:** step 106, step 60, step 100, step 121.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] All registered types listed and searchable
- [ ] A widget library added to the project appears in the palette

### Step 126: Selection, move, and resize

**Issue:** [#180](https://github.com/KenSuenobu/rust-pushrod/issues/180) `chassis: [3.4] Selection, move, and resize`

**Delivers:** Click, Shift+click, marquee; eight handles; keyboard nudge.

**Builds on:** step 124, step 117.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] All gestures work and update the inspector live
- [ ] Multi-selection moves together

### Step 127: Drag and drop placement

**Issue:** [#181](https://github.com/KenSuenobu/rust-pushrod/issues/181) `chassis: [3.3] Drag and drop placement`

**Delivers:** Drag from the palette; outline preview; drop creates and selects.

**Builds on:** step 124, step 125, step 86.

**Mockups:** [chassis-2.png](../mockups/chassis-2.png)

**Show on video:**

- [ ] Outline follows the pointer and snaps; drop creates and selects
- [ ] Escape cancels; one undo removes the widget

### Step 128: Snapping and alignment

**Issue:** [#182](https://github.com/KenSuenobu/rust-pushrod/issues/182) `chassis: [3.5] Snapping and alignment`

**Delivers:** Grid snap, guides to neighbors, align and distribute commands.

**Builds on:** step 126.

**Mockups:** [chassis-2.png](../mockups/chassis-2.png)

**Show on video:**

- [ ] Guides appear when edges or centers line up
- [ ] Align and distribute work on multi-selections

### Step 129: Clipboard and undo

**Issue:** [#183](https://github.com/KenSuenobu/rust-pushrod/issues/183) `chassis: [3.7] Clipboard and undo`

**Delivers:** Cut, copy, paste, duplicate; every designer change undoable.

**Builds on:** step 126, step 87.

**Show on video:**

- [ ] Every change undoes and redoes exactly
- [ ] Pasted widgets get unique names

### Step 130: Containers, z-order, and tab order

**Issue:** [#184](https://github.com/KenSuenobu/rust-pushrod/issues/184) `chassis: [3.6] Containers, z-order, and tab order`

**Delivers:** Drop into group boxes and containers; bring forward and back; set the Tab order.

**Builds on:** step 126, step 18, step 30, step 45.

**Show on video:**

- [ ] Moving a group box moves its children
- [ ] Tab order edits persist to the resource file

### Step 131: Layout managers in the designer

**Issue:** [#185](https://github.com/KenSuenobu/rust-pushrod/issues/185) `chassis: [3.8] Layout managers in the designer`

**Delivers:** Insert widgets into Column, Row, Form, and Grid layouts; spacers; breadcrumb; live resize.

**Builds on:** step 127, step 126, step 41, step 42, step 43, step 122.

**Mockups:** [chassis-layout-1.png](../mockups/chassis-layout-1.png), [chassis-layout-2.png](../mockups/chassis-layout-2.png), [chassis-layout-3.png](../mockups/chassis-layout-3.png), [chassis-layout-4.png](../mockups/chassis-layout-4.png), [chassis-layout-5.png](../mockups/chassis-layout-5.png), [chassis-layout-6.png](../mockups/chassis-layout-6.png)

**Show on video:**

- [ ] The contact dialog can be built entirely with layouts, as in the layout mockups
- [ ] Resizing in the designer matches the running application
- [ ] Reordering by drag, and undo, work inside layouts
- [ ] Breadcrumb and Escape select parents

### Step 132: Checkpoint: Chassis M3: form designer complete

**Video:** Build the contact dialog by dragging widgets, then again with layouts; resize it live.

## Part 7: Chassis M4: inspector, events, and code

Property inspector, events menu and indicators, handler creation, and the code view. Pushrod work and extra widgets this phase needs come first.

### Step 133: Pointer cursor per widget

**Issue:** [#101](https://github.com/KenSuenobu/rust-pushrod/issues/101) `pushrod: [9.9] Pointer cursor per widget`

**Delivers:** Widgets choose the pointer shape (I-beam over text, resize arrows, hand over links).

**Builds on:** step 4.

**Show on video:**

- [ ] Cursor changes on hover and restores on exit
- [ ] Busy cursor while `ui.set_busy(true)`

### Step 134: Choice button: pop-up list and click-to-cycle modes

**Issue:** [#44](https://github.com/KenSuenobu/rust-pushrod/issues/44) `pushrod: [3.6] Choice button: pop-up list and click-to-cycle modes`

**Delivers:** Pick one of several options compactly, as a pop-up list or a click-to-cycle button.

**Builds on:** step 40, step 39, step 11.

**Mockups:** [3.6-1.png](../mockups/3.6-1.png), [3.6-2.png](../mockups/3.6-2.png)

**Show on video:**

- [ ] Popup opens over the current item; selects by mouse/keyboard
- [ ] Cycle mode advances and reverses
- [ ] Parts are retrievable and modifiable as described

### Step 135: Context menus

**Issue:** [#88](https://github.com/KenSuenobu/rust-pushrod/issues/88) `pushrod: [2.19] Context menus`

**Delivers:** Right-click (or long-press) pop-up menus for a widget.

**Builds on:** step 17, step 93.

**Mockups:** [2.19-1.png](../mockups/2.19-1.png)

**Show on video:**

- [ ] Opens on secondary click, long-press, and Menu key
- [ ] Dynamic menus via callback

### Step 136: Layers and invalidation: cache hints, texture memory budget, incremental pipeline

**Issue:** [#113](https://github.com/KenSuenobu/rust-pushrod/issues/113) `pushrod: [1.20] Layers and invalidation: cache hints, texture memory budget, incremental pipeline`

**Delivers:** Decide which widgets get their own GPU texture, cap texture memory, and invalidate layout, paint, and compositing separately.

**Builds on:** step 5, step 47.

**Show on video:**

- [ ] Cache hints honored; Auto measurably reduces texture count on the gallery
- [ ] Memory budget enforced with eviction
- [ ] Moving a widget re-composites without repainting
- [ ] Golden images unchanged

### Step 137: Chassis: Code editor crate and editing core

**Issue:** [#142](https://github.com/KenSuenobu/rust-pushrod/issues/142) `pushrod: [11.1] Chassis: Code editor crate and editing core`

**Delivers:** Create `chassis-code-editor` with its text buffer, cursor and selection model, and edit commands, independent of drawing.

**Builds on:** step 1, step 100.

**Show on video:**

- [ ] Edits, moves, and selections covered by unit tests, including multi-byte characters
- [ ] A 100,000-line file opens and edits without noticeable delay
- [ ] Core has no dependency on drawing or the backend

### Step 138: Chassis: Code editor view: gutter, visible-line rendering, caret

**Issue:** [#143](https://github.com/KenSuenobu/rust-pushrod/issues/143) `pushrod: [11.2] Chassis: Code editor view: gutter, visible-line rendering, caret`

**Delivers:** The `CodeEditor` widget: draws line numbers, the visible lines, the current line, and the caret, and scrolls through files of any length.

**Builds on:** step 137, step 45, step 25, step 48, step 136, step 14, step 13.

**Mockups:** [11.2-1.png](../mockups/11.2-1.png)

**Show on video:**

- [ ] Smooth scrolling through a 5,000-line file
- [ ] Typing repaints only the edited line
- [ ] Sharp at 1x and 2x (1.10)
- [ ] Line numbers and current-line band as in the mockup

### Step 139: Chassis: Code editor input: keyboard, mouse selection, clipboard, input methods

**Issue:** [#144](https://github.com/KenSuenobu/rust-pushrod/issues/144) `pushrod: [11.3] Chassis: Code editor input: keyboard, mouse selection, clipboard, input methods`

**Delivers:** Editing with the keyboard and mouse the way users expect from a code editor.

**Builds on:** step 138, step 17, step 58, step 85, step 18.

**Mockups:** [11.3-1.png](../mockups/11.3-1.png)

**Show on video:**

- [ ] Every listed key and gesture works
- [ ] Composition text from an input method appears inline
- [ ] `TextChanged` and `SelectionChanged` carry the changed range

### Step 140: Chassis: Code editor undo and redo

**Issue:** [#145](https://github.com/KenSuenobu/rust-pushrod/issues/145) `pushrod: [11.4] Chassis: Code editor undo and redo`

**Delivers:** Undo and redo every edit, with consecutive typing grouped into one step.

**Builds on:** step 87, step 137.

**Show on video:**

- [ ] Typing a word and undoing removes the whole word
- [ ] Undo restores the previous selection
- [ ] Chassis edits undo as one step

### Step 141: Chassis: Code editor syntax highlighting with tree-sitter

**Issue:** [#146](https://github.com/KenSuenobu/rust-pushrod/issues/146) `pushrod: [11.5] Chassis: Code editor syntax highlighting with tree-sitter`

**Delivers:** Highlight code incrementally as it is edited, with a small, flat color scheme.

**Builds on:** step 138.

**Mockups:** [11.5-1.png](../mockups/11.5-1.png)

**Show on video:**

- [ ] Highlighting stays correct while typing inside strings and comments
- [ ] Editing re-parses incrementally, not the whole file
- [ ] Rust highlighted out of the box; other languages registrable

### Step 142: Chassis: Code editor API for the Chassis designer

**Issue:** [#149](https://github.com/KenSuenobu/rust-pushrod/issues/149) `pushrod: [11.8] Chassis: Code editor API for the Chassis designer`

**Delivers:** Let Chassis insert, find, and protect generated code: double-click a control and land in its event handler.

**Builds on:** step 141, step 140.

**Mockups:** [11.8-1.png](../mockups/11.8-1.png)

**Show on video:**

- [ ] Double-click flow from the mockup works end to end in an example
- [ ] Guarded ranges refuse edits and track surrounding edits
- [ ] Generated insertions undo as one step

### Step 143: Drag value (numeric scrubber)

**Issue:** [#302](https://github.com/KenSuenobu/rust-pushrod/issues/302) `extra-widgets: [2.9] Drag value (numeric scrubber)`

**Delivers:** A compact number that changes as you drag it sideways, and becomes a text field when clicked.

**Builds on:** step 101, step 38, step 17, step 133.

**Mockups:** [xw-2.9-1.png](../mockups/xw-2.9-1.png)

**Show on video:**

- [ ] Drag changes the value with fine and coarse modifiers
- [ ] Click to type; Escape cancels
- [ ] Range, decimals, prefix, and suffix

### Step 144: Property grid (name and value inspector)

**Issue:** [#255](https://github.com/KenSuenobu/rust-pushrod/issues/255) `extra-widgets: [4.5] Property grid (name and value inspector)`

**Delivers:** Categorized name/value rows with an editor per value type.

**Builds on:** step 101, step 61, step 43, step 38, step 34, step 134, step 92.

**Mockups:** [xw-4.5-1.png](../mockups/xw-4.5-1.png)

**Show on video:**

- [ ] Categories, divider, and changed-from-default marking
- [ ] Typed editors and custom editors by kind
- [ ] Works from Pushrod property introspection and from any source

### Step 145: Inspector drawer

**Issue:** [#186](https://github.com/KenSuenobu/rust-pushrod/issues/186) `chassis: [4.1] Inspector drawer`

**Delivers:** Identity header and Properties/Events tabs for the selection.

**Builds on:** step 106, step 103.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Follows the selection in the designer and the caret's handler in the code view
- [ ] Collapses and restores

### Step 146: Property grid

**Issue:** [#187](https://github.com/KenSuenobu/rust-pushrod/issues/187) `chassis: [4.2] Property grid`

**Delivers:** Every property from introspection, grouped and labeled.

**Builds on:** step 145, step 61, step 43, step 144.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png), [chassis-layout-4.png](../mockups/chassis-layout-4.png)

**Show on video:**

- [ ] Any registered widget's properties appear, including third-party widgets
- [ ] Changes show in the designer immediately and undo
- [ ] Layout settings for the selection's parent layout appear in an "In <layout>" section

### Step 147: Typed property editors

**Issue:** [#188](https://github.com/KenSuenobu/rust-pushrod/issues/188) `chassis: [4.3] Typed property editors`

**Delivers:** Text, number, check, choice, color, font, string-resource, and image editors.

**Builds on:** step 146, step 112, step 113, step 38, step 34, step 134, step 25, step 143.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Each kind has its editor; invalid values can't be applied
- [ ] Behavior flags appear as grouped check boxes
- [ ] Text and images can reference resources by ID

### Step 148: Code view

**Issue:** [#189](https://github.com/KenSuenobu/rust-pushrod/issues/189) `chassis: [5.4] Code view`

**Delivers:** The Code tab: the form's handler file in the Chassis code editor.

**Builds on:** step 108, step 138, step 139, step 142.

**Mockups:** [chassis-4.png](../mockups/chassis-4.png)

**Show on video:**

- [ ] Guarded lines can't be edited; bodies can
- [ ] Caret in a handler selects its widget

### Step 149: Create and open handlers

**Issue:** [#190](https://github.com/KenSuenobu/rust-pushrod/issues/190) `chassis: [5.3] Create and open handlers`

**Delivers:** Choosing an event (or double-clicking a widget) creates or opens its handler.

**Builds on:** step 148, step 80, step 79, step 142.

**Mockups:** [chassis-3.png](../mockups/chassis-3.png), [chassis-4.png](../mockups/chassis-4.png)

**Show on video:**

- [ ] Existing handlers open; missing ones are created once, never duplicated
- [ ] New stubs compile immediately
- [ ] Undo removes the stub and its wiring

### Step 150: Names and IDs

**Issue:** [#191](https://github.com/KenSuenobu/rust-pushrod/issues/191) `chassis: [4.4] Names and IDs`

**Delivers:** Validated, unique Rust identifiers; renames carried into constants and handlers.

**Builds on:** step 146, step 149, step 68, step 142.

**Show on video:**

- [ ] Invalid or duplicate names refused with a message
- [ ] Renaming renames handlers and the project still builds

### Step 151: Events context menu

**Issue:** [#192](https://github.com/KenSuenobu/rust-pushrod/issues/192) `chassis: [5.1] Events context menu`

**Delivers:** Right-click lists the widget's events, with a dot where code exists.

**Builds on:** step 149, step 79, step 135.

**Mockups:** [chassis-3.png](../mockups/chassis-3.png)

**Show on video:**

- [ ] Lists exactly the widget's events, default first
- [ ] Dots and handler names reflect the code file
- [ ] Choosing an event opens or creates its handler

### Step 152: Inspector Events tab

**Issue:** [#193](https://github.com/KenSuenobu/rust-pushrod/issues/193) `chassis: [5.2] Inspector Events tab`

**Delivers:** Every event with its handler name or "add..."; count on the tab.

**Builds on:** step 145, step 149, step 79.

**Mockups:** [chassis-4.png](../mockups/chassis-4.png)

**Show on video:**

- [ ] Matches the context menu's information
- [ ] Count updates as handlers are added and removed

### Step 153: Code indicator on the design surface

**Issue:** [#194](https://github.com/KenSuenobu/rust-pushrod/issues/194) `chassis: [5.5] Code indicator on the design surface`

**Delivers:** Corner mark on widgets that have handlers.

**Builds on:** step 124, step 80.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Appears and disappears as handlers are added and removed

### Step 154: Keep handlers in sync

**Issue:** [#195](https://github.com/KenSuenobu/rust-pushrod/issues/195) `chassis: [5.6] Keep handlers in sync`

**Delivers:** Renames, deletions, and orphaned handlers.

**Builds on:** step 149, step 82.

**Show on video:**

- [ ] Renames update handler names and wiring
- [ ] Deleting a widget never silently deletes code

### Step 155: Checkpoint: Chassis M4: inspector, events, and code complete

**Video:** Select widgets, edit properties, right-click for events, create handlers, and write code in the Code tab.

## Part 8: Chassis M5: build, run, and release (MVP)

Build and run from Chassis, CI, documentation, and the sample project. Completes the Chassis MVP. Pushrod work and extra widgets this phase needs come first.

### Step 156: Log view (append-only output)

**Issue:** [#303](https://github.com/KenSuenobu/rust-pushrod/issues/303) `extra-widgets: [4.13] Log view (append-only output)`

**Delivers:** A read-only, scrolling view of lines that follows the newest line, for build output and logs.

**Builds on:** step 101, step 45, step 13, step 44.

**Mockups:** [xw-4.13-1.png](../mockups/xw-4.13-1.png)

**Show on video:**

- [ ] Appending 100,000 lines stays responsive; old lines trimmed
- [ ] Follows the tail unless the user scrolled up
- [ ] Select and copy; levels marked without relying on color

### Step 157: Build and run

**Issue:** [#196](https://github.com/KenSuenobu/rust-pushrod/issues/196) `chassis: [6.2] Build and run`

**Delivers:** Run (F5) builds with Cargo and launches the application; output panel.

**Builds on:** step 109, step 81, step 156.

**Show on video:**

- [ ] F5 builds and launches the app
- [ ] Build output visible; failures reported

### Step 158: CI and golden screenshots

**Issue:** [#197](https://github.com/KenSuenobu/rust-pushrod/issues/197) `chassis: [7.1] CI and golden screenshots`

**Delivers:** Build, test, and compare Chassis's own screens with goldens.

**Builds on:** step 105, step 46, step 23.

**Show on video:**

- [ ] Core tests from M1 onward
- [ ] Golden images for designer, events menu, and code view

### Step 159: Chassis documentation

**Issue:** [#198](https://github.com/KenSuenobu/rust-pushrod/issues/198) `chassis: [7.2] Chassis documentation`

**Delivers:** A Chassis section on the docs site with a start-to-finish tutorial.

**Builds on:** step 157, step 3.

**Show on video:**

- [ ] All pages published with screenshots from Chassis itself
- [ ] Tutorial reproduces the sample project

### Step 160: Sample project

**Issue:** [#199](https://github.com/KenSuenobu/rust-pushrod/issues/199) `chassis: [7.3] Sample project`

**Delivers:** The contacts sample, built with Chassis, fictional data only.

**Builds on:** step 157, step 83.

**Show on video:**

- [ ] Builds and runs in CI
- [ ] Matches the tutorial

### Step 161: Checkpoint: Chassis MVP released

**Video:** Press F5: the contact dialog builds and runs. Publish the Chassis MVP.

## Part 9: Chassis example projects

Example projects built with Chassis once it reaches the MVP: small, complete applications that exercise its features and that anyone can build and test against.

### Step 162: Hello, Chassis

**Issue:** [#208](https://github.com/KenSuenobu/rust-pushrod/issues/208) `chassis: [8.1] Hello, Chassis`

**Delivers:** The smallest Chassis application: one form, a label, and a button that counts clicks.

**Builds on:** step 157, step 83, step 158, step 33, step 51.

**Mockups:** [chassis-example-hello-1.png](../mockups/chassis-example-hello-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 163: Temperature converter

**Issue:** [#209](https://github.com/KenSuenobu/rust-pushrod/issues/209) `chassis: [8.2] Temperature converter`

**Delivers:** Two linked fields that convert between Celsius and Fahrenheit as you type.

**Builds on:** step 157, step 83, step 158, step 38, step 37, step 43.

**Mockups:** [chassis-example-temperature-1.png](../mockups/chassis-example-temperature-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 164: Tip calculator

**Issue:** [#210](https://github.com/KenSuenobu/rust-pushrod/issues/210) `chassis: [8.3] Tip calculator`

**Delivers:** Enter a bill, pick a tip percentage, and split it between people.

**Builds on:** step 157, step 83, step 158, step 35, step 37, step 30, step 42.

**Mockups:** [chassis-example-tip-calculator-1.png](../mockups/chassis-example-tip-calculator-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 165: Settings dialog

**Issue:** [#211](https://github.com/KenSuenobu/rust-pushrod/issues/211) `chassis: [8.4] Settings dialog`

**Delivers:** A classic preferences dialog with OK, Cancel, and Apply.

**Builds on:** step 157, step 83, step 158, step 51, step 33, step 34, step 35, step 30.

**Mockups:** [chassis-example-settings-dialog-1.png](../mockups/chassis-example-settings-dialog-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 166: Image viewer

**Issue:** [#212](https://github.com/KenSuenobu/rust-pushrod/issues/212) `chassis: [8.5] Image viewer`

**Delivers:** Browse a set of pictures stored in the resource file.

**Builds on:** step 157, step 83, step 158, step 31, step 45, step 69.

**Mockups:** [chassis-example-image-viewer-1.png](../mockups/chassis-example-image-viewer-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 167: Stopwatch

**Issue:** [#213](https://github.com/KenSuenobu/rust-pushrod/issues/213) `chassis: [8.6] Stopwatch`

**Delivers:** Start, stop, lap, and reset, with a progress bar for the current minute.

**Builds on:** step 157, step 83, step 158, step 22, step 36.

**Mockups:** [chassis-example-stopwatch-1.png](../mockups/chassis-example-stopwatch-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 168: Quiz

**Issue:** [#214](https://github.com/KenSuenobu/rust-pushrod/issues/214) `chassis: [8.7] Quiz`

**Delivers:** A short multiple-choice quiz whose questions live in the resource file.

**Builds on:** step 157, step 83, step 158, step 35, step 36, step 52, step 69.

**Mockups:** [chassis-example-quiz-1.png](../mockups/chassis-example-quiz-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 169: Dice roller

**Issue:** [#215](https://github.com/KenSuenobu/rust-pushrod/issues/215) `chassis: [8.8] Dice roller`

**Delivers:** Roll one to four dice and keep a running total.

**Builds on:** step 157, step 83, step 158, step 97, step 37, step 42, step 69.

**Mockups:** [chassis-example-dice-1.png](../mockups/chassis-example-dice-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 170: Checkpoint: All example projects built

**Video:** Tour the eight examples; show each one's CI test and golden screenshots.

## Part 10: Chassis v2

The second release: settings, menu editor, form preview, multi-selection, error list, templates, packaging, code intelligence, localization, form inheritance, bindings, project search, multiple windows, accessibility checks, source control, and more build targets, and an AI assistant through MCP. Pushrod work and extra widgets it needs come first.

### Step 171: Property bindings: keep two properties in sync

**Issue:** [#98](https://github.com/KenSuenobu/rust-pushrod/issues/98) `pushrod: [9.6] Property bindings: keep two properties in sync`

**Delivers:** Bind a source widget property to a target property, one-way or two-way, with optional conversion.

**Builds on:** step 61.

**Show on video:**

- [ ] One-way and two-way bindings with transforms
- [ ] No infinite loops
- [ ] Unbind

### Step 172: Multi-line text area

**Issue:** [#87](https://github.com/KenSuenobu/rust-pushrod/issues/87) `pushrod: [2.18] Multi-line text area`

**Delivers:** Multi-line plain-text editing with scrolling, selection, clipboard, and undo.

**Builds on:** step 38, step 14, step 45.

**Mockups:** [2.18-1.png](../mockups/2.18-1.png)

**Show on video:**

- [ ] Editing, selection, clipboard, undo
- [ ] Smooth with 10,000 lines

### Step 173: Link label

**Issue:** [#126](https://github.com/KenSuenobu/rust-pushrod/issues/126) `pushrod: [2.22] Link label`

**Delivers:** Clickable text that looks like a link and opens a URL or runs a callback.

**Builds on:** step 18, step 11.

**Mockups:** [2.22-1.png](../mockups/2.22-1.png)

**Show on video:**

- [ ] Click, Return, and Space activate
- [ ] URL targets open the system browser unless the callback cancels
- [ ] Hover cursor and focus outline

### Step 174: Multiple windows: separate OS windows, each with its own engine

**Issue:** [#82](https://github.com/KenSuenobu/rust-pushrod/issues/82) `pushrod: [1.16] Multiple windows: separate OS windows, each with its own engine`

**Delivers:** Open additional operating-system windows (dialogs, palettes, inspectors), each driven by its own engine instance and widget tree.

**Builds on:** step 4.

**Show on video:**

- [ ] Two OS windows, each with its own engine and widget tree, render and receive input independently
- [ ] Shared application state; callbacks can open, find, and close other windows
- [ ] Closing one window doesn't end the application unless it's the last
- [ ] Works in both callback and event-loop styles

### Step 175: Bind actions by name from form files

**Issue:** [#67](https://github.com/KenSuenobu/rust-pushrod/issues/67) `pushrod: [6.8] Bind actions by name from form files`

**Delivers:** Form files name the action a control triggers; applications register handlers by name.

**Builds on:** step 64, step 85.

**Show on video:**

- [ ] Actions bind at load time
- [ ] Missing actions reported

### Step 176: no_std + alloc core crate

**Issue:** [#107](https://github.com/KenSuenobu/rust-pushrod/issues/107) `pushrod: [10.3] no_std + alloc core crate`

**Delivers:** Split the widget tree, events, and widgets into a core crate that builds without the standard library.

**Builds on:** step 57, step 23.

**Show on video:**

- [ ] `pushrod-core` builds for a `no_std` target with `alloc`
- [ ] `pushrod` API unchanged

### Step 177: Experimental: Atari ST / TOS backend

**Issue:** [#108](https://github.com/KenSuenobu/rust-pushrod/issues/108) `pushrod: [10.4] Experimental: Atari ST / TOS backend`

**Delivers:** Run Pushrod applications on an Atari ST: 68000 target, TOS input, monochrome or low-color drawing, outline mode, direct drawing.

**Builds on:** step 5, step 117, step 176.

**Show on video:**

- [ ] A Pushrod form with a button and callback runs in the Hatari emulator
- [ ] Size and speed measured and documented
- [ ] Runs on real ST hardware

### Step 178: SDL3 backend (pushrod-sdl3)

**Issue:** [#135](https://github.com/KenSuenobu/rust-pushrod/issues/135) `pushrod: [10.5] SDL3 backend (pushrod-sdl3)`

**Delivers:** A second backend crate on SDL3, a drop-in replacement for pushrod-sdl2.

**Builds on:** step 46, step 23.

**Show on video:**

- [ ] The gallery and reference application run on SDL3 unchanged
- [ ] Golden images match the SDL2 backend
- [ ] pushrod-sdl2 keeps working; applications choose either

### Step 179: Rich text view (formatted, read-only)

**Issue:** [#308](https://github.com/KenSuenobu/rust-pushrod/issues/308) `extra-widgets: [4.15] Rich text view (formatted, read-only)`

**Delivers:** Read-only formatted text: headings, bold and italic, lists, code, and links, from Markdown or styled spans.

**Builds on:** step 101, step 13, step 21, step 45, step 173.

**Mockups:** [xw-4.15-1.png](../mockups/xw-4.15-1.png)

**Show on video:**

- [ ] Headings, emphasis, lists, code, rules, and links render and wrap
- [ ] Select and copy
- [ ] Links emit an event or open in the browser

### Step 180: Workspace settings

**Issue:** [#200](https://github.com/KenSuenobu/rust-pushrod/issues/200) `chassis: [1.5] Workspace settings`

**Delivers:** Remember window size, pane widths, open forms, and recent projects.

**Builds on:** step 106, step 109.

**Show on video:**

- [ ] Restarting restores the layout and reopens the last project
- [ ] Recent projects in the File menu

### Step 181: Menu editor

**Issue:** [#201](https://github.com/KenSuenobu/rust-pushrod/issues/201) `chassis: [2.7] Menu editor`

**Delivers:** Design menu bars and context menus stored in the resource file.

**Builds on:** step 110, step 67, step 175, step 93.

**Show on video:**

- [ ] Create, nest, reorder menu items
- [ ] Items bind to actions by name
- [ ] Menus load at run time by ID

### Step 182: Form preview

**Issue:** [#202](https://github.com/KenSuenobu/rust-pushrod/issues/202) `chassis: [3.9] Form preview`

**Delivers:** Run the form live inside Chassis without building.

**Builds on:** step 124, step 174.

**Show on video:**

- [ ] Preview behaves like the running form except for handlers
- [ ] Tab order and resizing can be tested

### Step 183: Multiple selection editing

**Issue:** [#203](https://github.com/KenSuenobu/rust-pushrod/issues/203) `chassis: [4.5] Multiple selection editing`

**Delivers:** Edit the properties several selected widgets share.

**Builds on:** step 146.

**Show on video:**

- [ ] Shared properties shown; differing values blank
- [ ] One edit applies to all, undoable in one step

### Step 184: Error list

**Issue:** [#204](https://github.com/KenSuenobu/rust-pushrod/issues/204) `chassis: [6.3] Error list`

**Delivers:** Compiler errors listed and linked to the code view.

**Builds on:** step 157, step 148.

**Show on video:**

- [ ] Errors listed and navigable
- [ ] Handler errors marked in the events list

### Step 185: Project templates

**Issue:** [#205](https://github.com/KenSuenobu/rust-pushrod/issues/205) `chassis: [6.4] Project templates`

**Delivers:** Start from an empty app, a dialog app, or the contacts sample.

**Builds on:** step 109.

**Show on video:**

- [ ] Each template builds and runs unchanged

### Step 186: Packaging and releases

**Issue:** [#206](https://github.com/KenSuenobu/rust-pushrod/issues/206) `chassis: [7.4] Packaging and releases`

**Delivers:** `cargo install chassis` and downloadable builds.

**Builds on:** step 158, step 159, step 55, step 49.

**Show on video:**

- [ ] `cargo install chassis` works
- [ ] Release builds start on a clean machine

### Step 187: Find in project

**Issue:** [#216](https://github.com/KenSuenobu/rust-pushrod/issues/216) `chassis: [1.6] Find in project`

**Delivers:** Search forms, widget names and properties, resources, and code from one place.

**Builds on:** step 80, step 148, step 89.

**Show on video:**

- [ ] One search covers forms, resources, and code
- [ ] Every result navigates to its source

### Step 188: Multiple windows per project

**Issue:** [#217](https://github.com/KenSuenobu/rust-pushrod/issues/217) `chassis: [1.7] Multiple windows per project`

**Delivers:** Design applications with several windows, each its own form, and choose which opens first.

**Builds on:** step 109, step 82, step 174.

**Show on video:**

- [ ] A project with two windows builds and runs both
- [ ] Startup window configurable

### Step 189: Localization

**Issue:** [#218](https://github.com/KenSuenobu/rust-pushrod/issues/218) `chassis: [2.9] Localization`

**Delivers:** String tables per language, a language switch in the designer, and missing-translation reports.

**Builds on:** step 112, step 67.

**Show on video:**

- [ ] Strings editable per language
- [ ] Designer previews any language
- [ ] Missing translations listed

### Step 190: Source control status

**Issue:** [#219](https://github.com/KenSuenobu/rust-pushrod/issues/219) `chassis: [2.10] Source control status`

**Delivers:** Show which forms, resources, and code files have changed since the last commit.

**Builds on:** step 110.

**Show on video:**

- [ ] Changed and new items marked
- [ ] Branch shown in the status bar
- [ ] No effect outside Git repositories

### Step 191: Visual form inheritance

**Issue:** [#220](https://github.com/KenSuenobu/rust-pushrod/issues/220) `chassis: [3.10] Visual form inheritance`

**Delivers:** Base a form on another form and override only what differs.

**Builds on:** step 131, step 80, step 67.

**Show on video:**

- [ ] Derived forms update when the base changes
- [ ] Overrides survive base changes
- [ ] Inherited widgets visibly marked

### Step 192: Accessibility and usability checks

**Issue:** [#221](https://github.com/KenSuenobu/rust-pushrod/issues/221) `chassis: [3.11] Accessibility and usability checks`

**Delivers:** Warn about missing labels, Tab order gaps, mnemonic conflicts, and tiny click targets.

**Builds on:** step 130, step 83, step 62, step 59.

**Show on video:**

- [ ] Each check detects its problem on a test form
- [ ] Problems navigate to the widget
- [ ] `chassis check` reports them

### Step 193: Data bindings in the inspector

**Issue:** [#222](https://github.com/KenSuenobu/rust-pushrod/issues/222) `chassis: [4.6] Data bindings in the inspector`

**Delivers:** Bind a widget property to another property or to application state without writing code.

**Builds on:** step 146, step 171.

**Show on video:**

- [ ] A slider-to-label binding works without code
- [ ] Bindings round-trip through the resource file
- [ ] Bound properties marked in the inspector

### Step 194: Code intelligence with rust-analyzer

**Issue:** [#223](https://github.com/KenSuenobu/rust-pushrod/issues/223) `chassis: [5.8] Code intelligence with rust-analyzer`

**Delivers:** Completion, hover, signature help, and inline errors in the Code tab, through the Language Server Protocol.

**Builds on:** step 148, step 142.

**Show on video:**

- [ ] Completion and hover work in handler files
- [ ] Errors appear inline while typing
- [ ] Chassis works when rust-analyzer is absent

### Step 195: Build targets, including the Atari ST

**Issue:** [#224](https://github.com/KenSuenobu/rust-pushrod/issues/224) `chassis: [6.6] Build targets, including the Atari ST`

**Delivers:** Choose a build target and backend: desktop SDL2, SDL3 when available, and the experimental Atari ST backend.

**Builds on:** step 157, step 178, step 177.

**Show on video:**

- [ ] Switching targets builds the right backend
- [ ] Atari ST build produces a runnable program in an emulator

### Step 196: Command catalog: every designer command by name, with typed arguments

**Issue:** [#312](https://github.com/KenSuenobu/rust-pushrod/issues/312) `chassis: [9.1] Command catalog: every designer command by name, with typed arguments`

**Delivers:** Each designer command gets a stable name, typed arguments, a description, and a JSON schema, and can be run without the mouse or menus.

**Builds on:** step 107, step 127, step 146, step 60, step 61.

**Show on video:**

- [ ] Every designer command is in the catalog with a name, argument schema, and description
- [ ] `run` validates, executes through undo, and reports what changed
- [ ] Readable errors with suggestions for near misses
- [ ] A group of commands undoes as one step

### Step 197: MCP server inside Chassis

**Issue:** [#313](https://github.com/KenSuenobu/rust-pushrod/issues/313) `chassis: [9.2] MCP server inside Chassis`

**Delivers:** Chassis listens for MCP clients on this computer only, behind a token, when the user turns it on.

**Builds on:** step 196, step 12.

**Mockups:** [chassis-assistant-2.png](../mockups/chassis-assistant-2.png)

**Show on video:**

- [ ] Off by default; with default settings, listens on localhost only and rejects requests without the token
- [ ] Tool calls run on the UI thread and never block painting
- [ ] Works with Streamable HTTP and stdio
- [ ] Turning it off disconnects clients at once

### Step 198: Assistant tools: read the project

**Issue:** [#314](https://github.com/KenSuenobu/rust-pushrod/issues/314) `chassis: [9.3] Assistant tools: read the project`

**Delivers:** Tools that let an assistant look: the project, forms, widget types and their properties, the selection, resources, and build errors.

**Builds on:** step 197, step 60, step 61.

**Show on video:**

- [ ] Every tool returns valid, compact JSON for the sample project
- [ ] `describe_widget_type` covers every registered type, including extra widgets
- [ ] Large forms can be read by subtree
- [ ] No read tool changes anything

### Step 199: Assistant tools: edit forms and resources

**Issue:** [#315](https://github.com/KenSuenobu/rust-pushrod/issues/315) `chassis: [9.4] Assistant tools: edit forms and resources`

**Delivers:** Tools that create forms, add, move, and delete widgets, set properties and layout, and edit resources, each validated and undoable.

**Builds on:** step 198.

**Show on video:**

- [ ] Each write tool validates, applies through undo, and reports the result
- [ ] `apply_form` builds a complete form in one call, or changes nothing if invalid
- [ ] One assistant turn is one undo step
- [ ] Transcript tests run in CI without a model

### Step 200: Assistant tools: events, handler code, and builds

**Issue:** [#316](https://github.com/KenSuenobu/rust-pushrod/issues/316) `chassis: [9.5] Assistant tools: events, handler code, and builds`

**Delivers:** Tools to list a widget's events, read and write a handler's code, build the project, and read the errors.

**Builds on:** step 198, step 148, step 157.

**Show on video:**

- [ ] A handler can be created, read, and replaced through tools
- [ ] `build` returns structured errors tied to handlers
- [ ] Outside edits to handler files are picked up
- [ ] The API summary is generated from the code, not hand-written

### Step 201: Assistant tool: a picture of the form

**Issue:** [#317](https://github.com/KenSuenobu/rust-pushrod/issues/317) `chassis: [9.6] Assistant tool: a picture of the form`

**Delivers:** A tool that returns an image of a form, so the assistant can see what it built.

**Builds on:** step 197, step 182, step 46.

**Show on video:**

- [ ] Returns a PNG that matches the designer
- [ ] Optional name labels
- [ ] Size limited and configurable

### Step 202: Assistant activity, review mode, and safety

**Issue:** [#318](https://github.com/KenSuenobu/rust-pushrod/issues/318) `chassis: [9.7] Assistant activity, review mode, and safety`

**Delivers:** Chassis shows when an assistant is connected and what it changed, and can hold its changes as a preview to apply or discard.

**Builds on:** step 199, step 200, step 103, step 156.

**Mockups:** [chassis-assistant-1.png](../mockups/chassis-assistant-1.png), [chassis-assistant-3.png](../mockups/chassis-assistant-3.png)

**Show on video:**

- [ ] Indicator visible whenever a client is connected
- [ ] Every assistant change is listed and individually undoable
- [ ] Review mode previews, then applies or discards as one step
- [ ] Stop works mid-turn; with default settings destructive actions confirm and nothing outside the project is reachable

### Step 203: Connect dialog and assistant guide

**Issue:** [#319](https://github.com/KenSuenobu/rust-pushrod/issues/319) `chassis: [9.8] Connect dialog and assistant guide`

**Delivers:** A dialog with the address, token, and ready-to-paste setup for common clients, and a guide page with a worked example.

**Builds on:** step 197.

**Mockups:** [chassis-assistant-2.png](../mockups/chassis-assistant-2.png)

**Show on video:**

- [ ] Copyable setup for Claude Code, the Claude desktop app, and generic clients
- [ ] Test confirms a connection
- [ ] Guide reproduces on the sample project
- [ ] Privacy statement in the dialog and the guide

### Step 204: Assistant settings: every check is configurable

**Issue:** [#322](https://github.com/KenSuenobu/rust-pushrod/issues/322) `chassis: [9.10] Assistant settings: every check is configurable`

**Delivers:** One settings page where each assistant check, confirmation, and limit can be turned on, off, or adjusted, with safe defaults.

**Builds on:** step 202, step 43.

**Mockups:** [chassis-assistant-5.png](../mockups/chassis-assistant-5.png)

**Show on video:**

- [ ] Every check, confirmation, and limit in 9.2 to 9.7 is read from the policy; none is hard-coded
- [ ] Defaults are the safe values in the table; Restore defaults works
- [ ] Lowering a protection is marked and asks, stating the consequence
- [ ] Project settings can tighten but never loosen the user's settings
- [ ] Each setting has a test showing the tools obey it

### Step 205: Assistant drawer: chat inside Chassis

**Issue:** [#320](https://github.com/KenSuenobu/rust-pushrod/issues/320) `chassis: [9.9] Assistant drawer: chat inside Chassis`

**Delivers:** A chat panel in Chassis that talks to a model directly and uses the same tools, for people who want one window.

**Builds on:** step 202, step 204, step 12, step 172, step 179, step 103.

**Mockups:** [chassis-assistant-4.png](../mockups/chassis-assistant-4.png)

**Show on video:**

- [ ] Chat requests build and edit forms through the same tools as MCP clients
- [ ] Streaming replies; the interface never blocks
- [ ] Key stored in the system keychain; nothing sent without one
- [ ] Provider trait with the Claude API implemented

### Step 206: Checkpoint: Chassis v2 released

**Video:** Show the v2 features, including an Atari ST build running in an emulator.

## Part 11: Extra widgets 1 and 2 (the pushrod-extra-widgets MVP)

The rest of the `pushrod-extra-widgets` crate's first two milestones: the popover and gallery, then everyday controls, pickers, lists, panels, and dialogs. Pushrod work these need comes first.

### Step 207: Stack layout (one child visible at a time)

**Issue:** [#120](https://github.com/KenSuenobu/rust-pushrod/issues/120) `pushrod: [5.9] Stack layout (one child visible at a time)`

**Delivers:** Children occupy the same area; one is shown at a time (pages, wizards, tab contents).

**Builds on:** step 41.

**Show on video:**

- [ ] Switching pages hides others and keeps their state
- [ ] Size policy: largest or current

### Step 208: Spin box

**Issue:** [#49](https://github.com/KenSuenobu/rust-pushrod/issues/49) `pushrod: [4.2] Spin box`

**Delivers:** Numeric field with up/down arrows, range, and step.

**Builds on:** step 40, step 38, step 32.

**Mockups:** [4.2-1.png](../mockups/4.2-1.png)

**Show on video:**

- [ ] Arrows, keys, wheel adjust within range
- [ ] Typed values clamp
- [ ] Parts are retrievable and modifiable as described

### Step 209: Combo box

**Issue:** [#53](https://github.com/KenSuenobu/rust-pushrod/issues/53) `pushrod: [4.6] Combo box`

**Delivers:** Text field with a drop-down list of suggestions.

**Builds on:** step 40, step 38, step 39, step 89.

**Mockups:** [4.6-1.png](../mockups/4.6-1.png)

**Show on video:**

- [ ] Typing filters; arrow opens; choosing fills
- [ ] Parts are retrievable and modifiable as described

### Step 210: Extra widgets gallery example

**Issue:** [#253](https://github.com/KenSuenobu/rust-pushrod/issues/253) `extra-widgets: [1.2] Extra widgets gallery example`

**Delivers:** A runnable gallery showing every extra widget, used for golden screenshots.

**Builds on:** step 101, step 46, step 15, step 90.

**Mockups:** [xw-1.2-1.png](../mockups/xw-1.2-1.png)

**Show on video:**

- [ ] Gallery runs and shows every widget built so far
- [ ] Golden images captured in CI
- [ ] Contributing guide says new widgets must be added to the gallery

### Step 211: Menu button and split button

**Issue:** [#258](https://github.com/KenSuenobu/rust-pushrod/issues/258) `extra-widgets: [2.1] Menu button and split button`

**Delivers:** A button that opens a menu, and a button with a separate arrow part that opens one.

**Builds on:** step 101, step 33, step 93, step 85.

**Mockups:** [xw-2.1-1.png](../mockups/xw-2.1-1.png)

**Show on video:**

- [ ] Menu button opens its menu by click, Space, and Alt+Down
- [ ] Split button: main part runs the default action, arrow part opens the menu
- [ ] Menu items are actions with shortcuts and enabled state

### Step 212: Segmented control

**Issue:** [#50](https://github.com/KenSuenobu/rust-pushrod/issues/50) `extra-widgets: [2.2] Segmented control`

**Delivers:** A row of joined buttons: pick one, or toggle several.

**Builds on:** step 101, step 33, step 35, step 19.

**Mockups:** [xw-2.2-1.png](../mockups/xw-2.2-1.png)

**Show on video:**

- [ ] One and Many modes
- [ ] Per-segment disabled state, text, and icons
- [ ] Keyboard navigation and selection

### Step 213: Stepper (up and down arrows)

**Issue:** [#259](https://github.com/KenSuenobu/rust-pushrod/issues/259) `extra-widgets: [2.3] Stepper (up and down arrows)`

**Delivers:** A pair of small up/down arrows that step a value, attachable to any field.

**Builds on:** step 101, step 32, step 22, step 17, step 171.

**Mockups:** [xw-2.3-1.png](../mockups/xw-2.3-1.png)

**Show on video:**

- [ ] Click steps; holding repeats with a delay
- [ ] Limits disable the matching arrow unless wrapping
- [ ] Buddy binding keeps another widget in step

### Step 214: Dial (rotary knob)

**Issue:** [#127](https://github.com/KenSuenobu/rust-pushrod/issues/127) `extra-widgets: [2.4] Dial (rotary knob)`

**Delivers:** A round control turned by dragging, the wheel, or the keyboard, for values like volume, pan, or angle.

**Builds on:** step 101, step 11, step 17.

**Mockups:** [xw-2.4-1.png](../mockups/xw-2.4-1.png)

**Show on video:**

- [ ] Drag, wheel, and keys change the value within range
- [ ] Wrapping mode for angles
- [ ] Notches and optional value text

### Step 215: Level indicator (meter)

**Issue:** [#54](https://github.com/KenSuenobu/rust-pushrod/issues/54) `extra-widgets: [2.5] Level indicator (meter)`

**Delivers:** A bar showing a level against warning and critical thresholds, continuous or in discrete segments.

**Builds on:** step 101, step 6, step 19, step 17.

**Mockups:** [xw-2.5-1.png](../mockups/xw-2.5-1.png)

**Show on video:**

- [ ] Continuous and discrete styles
- [ ] Warning and critical thresholds visible without color
- [ ] Optional text and editable mode

### Step 216: Range slider (two thumbs)

**Issue:** [#306](https://github.com/KenSuenobu/rust-pushrod/issues/306) `extra-widgets: [2.10] Range slider (two thumbs)`

**Delivers:** A slider with two thumbs selecting a lower and an upper value.

**Builds on:** step 101, step 37, step 17.

**Mockups:** [xw-2.10-1.png](../mockups/xw-2.10-1.png)

**Show on video:**

- [ ] Two thumbs that cannot cross
- [ ] Dragging the span moves both
- [ ] Keyboard control of each thumb

### Step 217: Calendar (month view)

**Issue:** [#260](https://github.com/KenSuenobu/rust-pushrod/issues/260) `extra-widgets: [3.1] Calendar (month view)`

**Delivers:** A month grid for choosing a day or a range of days, usable on its own or in the date picker.

**Builds on:** step 101, step 32, step 17, step 18.

**Mockups:** [xw-3.1-1.png](../mockups/xw-3.1-1.png)

**Show on video:**

- [ ] Single and range selection
- [ ] Min/max limits and marked days
- [ ] Keyboard navigation by day and month

### Step 218: Date picker

**Issue:** [#91](https://github.com/KenSuenobu/rust-pushrod/issues/91) `extra-widgets: [3.2] Date picker`

**Delivers:** A date field with a drop-down month calendar.

**Builds on:** step 102, step 217, step 38, step 32.

**Mockups:** [xw-3.2-1.png](../mockups/xw-3.2-1.png)

**Show on video:**

- [ ] Typed and chosen dates stay in step
- [ ] Min/max enforced
- [ ] Up/Down steps the date part under the caret

### Step 219: Time field and date-time field

**Issue:** [#261](https://github.com/KenSuenobu/rust-pushrod/issues/261) `extra-widgets: [3.3] Time field and date-time field`

**Delivers:** A time entry with steppable parts, and a combined date-and-time field.

**Builds on:** step 213, step 218, step 38.

**Mockups:** [xw-3.3-1.png](../mockups/xw-3.3-1.png)

**Show on video:**

- [ ] Part-by-part editing with keys and stepper
- [ ] 12- and 24-hour formats
- [ ] Date-time field keeps one combined value

### Step 220: Color well and color picker

**Issue:** [#46](https://github.com/KenSuenobu/rust-pushrod/issues/46) `extra-widgets: [3.4] Color well and color picker`

**Delivers:** A swatch button that opens a color picker with swatches, sliders, and a hex field.

**Builds on:** step 102, step 121, step 37, step 38, step 86.

**Mockups:** [xw-3.4-1.png](../mockups/xw-3.4-1.png)

**Show on video:**

- [ ] Swatches, recent colors, RGB and HSV sliders, hex field
- [ ] Popover and inline use
- [ ] Drag a color between wells

### Step 221: Font chooser

**Issue:** [#129](https://github.com/KenSuenobu/rust-pushrod/issues/129) `extra-widgets: [3.5] Font chooser`

**Delivers:** A font button that opens a chooser with family, style, size, and a preview.

**Builds on:** step 102, step 89, step 25, step 38, step 51.

**Mockups:** [xw-3.5-1.png](../mockups/xw-3.5-1.png)

**Show on video:**

- [ ] Family, style, and size lists with filtering
- [ ] Preview text editable
- [ ] Popover and dialog use

### Step 222: Path field (file or folder with Browse)

**Issue:** [#262](https://github.com/KenSuenobu/rust-pushrod/issues/262) `extra-widgets: [3.6] Path field (file or folder with Browse)`

**Delivers:** A text field for a file or folder path, with a Browse button and drop support.

**Builds on:** step 101, step 38, step 98, step 86.

**Mockups:** [xw-3.6-1.png](../mockups/xw-3.6-1.png)

**Show on video:**

- [ ] Browse opens the chooser at the current path
- [ ] Typed, browsed, and dropped paths
- [ ] Missing-path mark when required

### Step 223: Token field (tags in a field)

**Issue:** [#307](https://github.com/KenSuenobu/rust-pushrod/issues/307) `extra-widgets: [3.10] Token field (tags in a field)`

**Delivers:** A text field whose entries become removable tokens, with optional suggestions.

**Builds on:** step 102, step 38, step 118, step 89.

**Mockups:** [xw-3.10-1.png](../mockups/xw-3.10-1.png)

**Show on video:**

- [ ] Tokens created by Return, Tab, and separators; removed by x and Backspace
- [ ] Wraps to more lines as tokens are added
- [ ] Suggestions, validation, and uniqueness

### Step 224: Tree view (outline)

**Issue:** [#55](https://github.com/KenSuenobu/rust-pushrod/issues/55) `extra-widgets: [4.1] Tree view (outline)`

**Delivers:** A hierarchical list with expand/collapse triangles, backed by a tree data source.

**Builds on:** step 101, step 89, step 91, step 45, step 86.

**Mockups:** [xw-4.1-1.png](../mockups/xw-4.1-1.png)

**Show on video:**

- [ ] Expand and collapse by mouse and keyboard
- [ ] Lazy children from a data source
- [ ] Optional columns and drag to reparent

### Step 225: Check list

**Issue:** [#263](https://github.com/KenSuenobu/rust-pushrod/issues/263) `extra-widgets: [4.6] Check list`

**Delivers:** A list with a check box on every row, separate from the selection.

**Builds on:** step 101, step 89, step 34.

**Mockups:** [xw-4.6-1.png](../mockups/xw-4.6-1.png)

**Show on video:**

- [ ] Check state independent of selection
- [ ] Space toggles the selected rows
- [ ] Check-all header with mixed state

### Step 226: Editable list (add, remove, reorder)

**Issue:** [#264](https://github.com/KenSuenobu/rust-pushrod/issues/264) `extra-widgets: [4.7] Editable list (add, remove, reorder)`

**Delivers:** A list with + and - buttons underneath, in-place editing, and reordering.

**Builds on:** step 101, step 89, step 32, step 38, step 86.

**Mockups:** [xw-4.7-1.png](../mockups/xw-4.7-1.png)

**Show on video:**

- [ ] Add, remove, edit, and reorder
- [ ] Validation and uniqueness
- [ ] Buttons disabled when not applicable

### Step 227: Accordion (one section open at a time)

**Issue:** [#265](https://github.com/KenSuenobu/rust-pushrod/issues/265) `extra-widgets: [5.2] Accordion (one section open at a time)`

**Delivers:** A stack of titled sections where opening one closes the others.

**Builds on:** step 101, step 92, step 42.

**Mockups:** [xw-5.2-1.png](../mockups/xw-5.2-1.png)

**Show on video:**

- [ ] Opening one closes the others (or multiple mode)
- [ ] Open section fills the height
- [ ] Keyboard navigation between headers

### Step 228: Sidebar (navigation list)

**Issue:** [#266](https://github.com/KenSuenobu/rust-pushrod/issues/266) `extra-widgets: [5.3] Sidebar (navigation list)`

**Delivers:** A list of sections with icons and headings that switches the page beside it.

**Builds on:** step 101, step 89, step 207.

**Mockups:** [xw-5.3-1.png](../mockups/xw-5.3-1.png)

**Show on video:**

- [ ] Headings, icons, and counts
- [ ] Bound to a stack layout
- [ ] Keyboard navigation

### Step 229: Input dialog (ask for one value)

**Issue:** [#267](https://github.com/KenSuenobu/rust-pushrod/issues/267) `extra-widgets: [6.1] Input dialog (ask for one value)`

**Delivers:** A one-call modal dialog asking for a single text, number, or choice.

**Builds on:** step 101, step 51, step 38, step 208, step 209.

**Mockups:** [xw-6.1-1.png](../mockups/xw-6.1-1.png)

**Show on video:**

- [ ] Text, number, and choice forms
- [ ] Validation disables OK
- [ ] Completion receives None on Cancel

### Step 230: Progress dialog

**Issue:** [#268](https://github.com/KenSuenobu/rust-pushrod/issues/268) `extra-widgets: [6.2] Progress dialog`

**Delivers:** A modal dialog with a message, a progress bar, and Cancel, for long tasks.

**Builds on:** step 101, step 51, step 36, step 22.

**Mockups:** [xw-6.2-1.png](../mockups/xw-6.2-1.png)

**Show on video:**

- [ ] Delayed appearance for quick tasks
- [ ] Determinate and busy progress
- [ ] Cancel reported to the task

### Step 231: Task dialog (alert with details and options)

**Issue:** [#269](https://github.com/KenSuenobu/rust-pushrod/issues/269) `extra-widgets: [6.3] Task dialog (alert with details and options)`

**Delivers:** An alert with a main instruction, expandable details, a "don't ask again" box, and command choices.

**Builds on:** step 101, step 52, step 92, step 34, step 172, step 51.

**Mockups:** [xw-6.3-1.png](../mockups/xw-6.3-1.png)

**Show on video:**

- [ ] Instruction, message, and copyable details
- [ ] Check box state returned
- [ ] Command choices with descriptions

### Step 232: Recent files menu

**Issue:** [#309](https://github.com/KenSuenobu/rust-pushrod/issues/309) `extra-widgets: [6.10] Recent files menu`

**Delivers:** An Open Recent submenu that remembers the last files, with Clear and missing-file handling.

**Builds on:** step 101, step 93, step 85, step 20.

**Mockups:** [xw-6.10-1.png](../mockups/xw-6.10-1.png)

**Show on video:**

- [ ] Add moves to top and caps the list
- [ ] Numbered items with mnemonics; Clear Menu
- [ ] Persists across runs; missing files handled

### Step 233: Checkpoint: pushrod-extra-widgets MVP released

**Video:** Publish `pushrod-extra-widgets`; tour the gallery: pickers in popovers, tree and check lists, accordion and sidebar, and the one-call dialogs.

## Part 12: Extra widgets 3, specialized views and assistants

Specialized views and assistants in `pushrod-extra-widgets`. Pushrod work these need comes first.

### Step 234: Ruler

**Issue:** [#270](https://github.com/KenSuenobu/rust-pushrod/issues/270) `extra-widgets: [2.7] Ruler`

**Delivers:** A horizontal or vertical scale in pixels, points, or units, with markers that can be dragged.

**Builds on:** step 101, step 27, step 45, step 17.

**Mockups:** [xw-2.7-1.png](../mockups/xw-2.7-1.png)

**Show on video:**

- [ ] Units, origin, and zoom
- [ ] Follows an attached container's scroll
- [ ] Pointer tracking line and draggable markers

### Step 235: Shortcut field (key combination recorder)

**Issue:** [#271](https://github.com/KenSuenobu/rust-pushrod/issues/271) `extra-widgets: [2.8] Shortcut field (key combination recorder)`

**Delivers:** A field that records the key combination pressed while it has focus.

**Builds on:** step 101, step 38, step 85, step 18.

**Mockups:** [xw-2.8-1.png](../mockups/xw-2.8-1.png)

**Show on video:**

- [ ] Records combinations with modifiers
- [ ] Escape cancels, Backspace clears
- [ ] Conflicts reported inline

### Step 236: Image well

**Issue:** [#272](https://github.com/KenSuenobu/rust-pushrod/issues/272) `extra-widgets: [3.7] Image well`

**Delivers:** A framed area that shows an image and accepts a new one by drop, paste, or Choose.

**Builds on:** step 101, step 31, step 86, step 98.

**Mockups:** [xw-3.7-1.png](../mockups/xw-3.7-1.png)

**Show on video:**

- [ ] Drop, paste, choose, and clear
- [ ] Drop feedback outline
- [ ] Optional square crop

### Step 237: New password (enter, confirm, strength)

**Issue:** [#273](https://github.com/KenSuenobu/rust-pushrod/issues/273) `extra-widgets: [3.8] New password (enter, confirm, strength)`

**Delivers:** Two password fields with a strength meter and a match check.

**Builds on:** step 215, step 38, step 171.

**Mockups:** [xw-3.8-1.png](../mockups/xw-3.8-1.png)

**Show on video:**

- [ ] Strength meter and rule messages
- [ ] Match check and show/hide
- [ ] Acceptable state bindable to OK

### Step 238: Character picker

**Issue:** [#274](https://github.com/KenSuenobu/rust-pushrod/issues/274) `extra-widgets: [3.9] Character picker`

**Delivers:** A searchable grid of characters by block, with a preview and code point.

**Builds on:** step 101, step 13, step 38, step 45, step 134.

**Mockups:** [xw-3.9-1.png](../mockups/xw-3.9-1.png)

**Show on video:**

- [ ] Blocks, name search, and recent characters
- [ ] Preview with name and code point
- [ ] Grid virtualized for large blocks

### Step 239: Column browser

**Issue:** [#45](https://github.com/KenSuenobu/rust-pushrod/issues/45) `extra-widgets: [4.2] Column browser`

**Delivers:** Columns of lists where selecting an item in one column fills the next, as in NeXTSTEP and the Mac Finder.

**Builds on:** step 101, step 89, step 45, step 44.

**Mockups:** [xw-4.2-1.png](../mockups/xw-4.2-1.png)

**Show on video:**

- [ ] Selecting fills the next column
- [ ] Keyboard navigation across columns
- [ ] Horizontal scrolling keeps the selection visible

### Step 240: Icon grid view

**Issue:** [#275](https://github.com/KenSuenobu/rust-pushrod/issues/275) `extra-widgets: [4.3] Icon grid view`

**Delivers:** A scrolling grid of icons with captions, rubber-band selection, and drag and drop.

**Builds on:** step 101, step 97, step 45, step 17, step 86.

**Mockups:** [xw-4.3-1.png](../mockups/xw-4.3-1.png)

**Show on video:**

- [ ] Reflowing, virtualized grid
- [ ] Rubber-band and keyboard selection, type-to-find
- [ ] Drag and drop and in-place rename

### Step 241: Data grid (editable table)

**Issue:** [#276](https://github.com/KenSuenobu/rust-pushrod/issues/276) `extra-widgets: [4.4] Data grid (editable table)`

**Delivers:** A table whose cells edit in place with typed editors, with row add and delete.

**Builds on:** step 218, step 91, step 38, step 34, step 134.

**Mockups:** [xw-4.4-1.png](../mockups/xw-4.4-1.png)

**Show on video:**

- [ ] Typed cell editors with validation
- [ ] Spreadsheet-style keyboard editing
- [ ] Add and delete rows through the data source

### Step 242: Filter field (live filter for lists, tables, and trees)

**Issue:** [#277](https://github.com/KenSuenobu/rust-pushrod/issues/277) `extra-widgets: [4.10] Filter field (live filter for lists, tables, and trees)`

**Delivers:** A search field bound to a list, table, or tree that hides non-matching rows as you type.

**Builds on:** step 224, step 38, step 89, step 91.

**Mockups:** [xw-4.10-1.png](../mockups/xw-4.10-1.png)

**Show on video:**

- [ ] Works with list, table, and tree views
- [ ] Tree filtering keeps ancestors
- [ ] Custom matching and no-match message

### Step 243: Dual list (list builder)

**Issue:** [#278](https://github.com/KenSuenobu/rust-pushrod/issues/278) `extra-widgets: [4.8] Dual list (list builder)`

**Delivers:** Available and chosen lists side by side, with buttons to move items between them and order the chosen list.

**Builds on:** step 242, step 89, step 33, step 86.

**Mockups:** [xw-4.8-1.png](../mockups/xw-4.8-1.png)

**Show on video:**

- [ ] Move items by buttons, double-click, and drag
- [ ] Order the chosen list
- [ ] Buttons disabled when not applicable

### Step 244: Rule editor (query builder)

**Issue:** [#279](https://github.com/KenSuenobu/rust-pushrod/issues/279) `extra-widgets: [4.11] Rule editor (query builder)`

**Delivers:** Rows of "field, operator, value" with add and remove buttons, joined by All or Any.

**Builds on:** step 218, step 134, step 38, step 32, step 42.

**Mockups:** [xw-4.11-1.png](../mockups/xw-4.11-1.png)

**Show on video:**

- [ ] Operators follow the field's type
- [ ] Add, remove, and optional nested groups
- [ ] Rules returned as plain, serializable data

### Step 245: Image viewer (zoom and pan)

**Issue:** [#280](https://github.com/KenSuenobu/rust-pushrod/issues/280) `extra-widgets: [4.12] Image viewer (zoom and pan)`

**Delivers:** An image in a scrollable view with zoom to fit, actual size, zoom by wheel, and drag to pan.

**Builds on:** step 101, step 31, step 45, step 17, step 133.

**Mockups:** [xw-4.12-1.png](../mockups/xw-4.12-1.png)

**Show on video:**

- [ ] Fit, fill, 100%, and zoom around the pointer
- [ ] Drag to pan
- [ ] Tiled drawing for large images

### Step 246: Document tabs (closable and reorderable)

**Issue:** [#281](https://github.com/KenSuenobu/rust-pushrod/issues/281) `extra-widgets: [5.4] Document tabs (closable and reorderable)`

**Delivers:** A tab bar for open documents: close buttons, a modified mark, drag to reorder, and overflow.

**Builds on:** step 101, step 90, step 86, step 135, step 93.

**Mockups:** [xw-5.4-1.png](../mockups/xw-5.4-1.png)

**Show on video:**

- [ ] Close buttons, modified marks, middle-click close
- [ ] Drag to reorder and context menu
- [ ] Overflow arrows and tab list menu

### Step 247: Wizard (step-by-step assistant)

**Issue:** [#282](https://github.com/KenSuenobu/rust-pushrod/issues/282) `extra-widgets: [6.4] Wizard (step-by-step assistant)`

**Delivers:** Pages shown one at a time with Back, Next, and Finish, and a list of steps.

**Builds on:** step 101, step 207, step 51, step 33.

**Mockups:** [xw-6.4-1.png](../mockups/xw-6.4-1.png)

**Show on video:**

- [ ] Back, Next, Finish, and Cancel with page validation
- [ ] Step list and conditional pages
- [ ] Summary page optional

### Step 248: Settings dialog

**Issue:** [#283](https://github.com/KenSuenobu/rust-pushrod/issues/283) `extra-widgets: [6.5] Settings dialog`

**Delivers:** A sidebar of categories, a page per category, and OK, Cancel, and Apply.

**Builds on:** step 228, step 207, step 43, step 51, step 90.

**Mockups:** [xw-6.5-1.png](../mockups/xw-6.5-1.png)

**Show on video:**

- [ ] Sidebar or tabs layout
- [ ] Apply enabled only on changes; OK applies and closes
- [ ] Restore Defaults per page and optional search

### Step 249: About dialog

**Issue:** [#284](https://github.com/KenSuenobu/rust-pushrod/issues/284) `extra-widgets: [6.6] About dialog`

**Delivers:** The application's icon, name, version, copyright, website, and credits and license pages.

**Builds on:** step 101, step 51, step 31, step 173, step 172.

**Mockups:** [xw-6.6-1.png](../mockups/xw-6.6-1.png)

**Show on video:**

- [ ] Icon, name, version, copyright, and website
- [ ] Credits and license pages
- [ ] Values from Cargo metadata

### Step 250: Find and replace bar

**Issue:** [#285](https://github.com/KenSuenobu/rust-pushrod/issues/285) `extra-widgets: [6.7] Find and replace bar`

**Delivers:** A bar with find and replace fields, match options, a match count, and next and previous.

**Builds on:** step 101, step 38, step 34, step 32, step 172, step 85.

**Mockups:** [xw-6.7-1.png](../mockups/xw-6.7-1.png)

**Show on video:**

- [ ] Find, next, previous, and match count
- [ ] Case, whole word, and regular expression options
- [ ] Replace and Replace All through the Searchable trait

### Step 251: Keyboard shortcuts editor

**Issue:** [#286](https://github.com/KenSuenobu/rust-pushrod/issues/286) `extra-widgets: [6.8] Keyboard shortcuts editor`

**Delivers:** A filterable table of actions and their shortcuts, edited with the shortcut field.

**Builds on:** step 235, step 242, step 91, step 85.

**Mockups:** [xw-6.8-1.png](../mockups/xw-6.8-1.png)

**Show on video:**

- [ ] Filter by name or key combination
- [ ] Conflict detection with reassign
- [ ] Reset one or all

### Step 252: Checkpoint: Extra widgets 3 released

**Video:** Tour the specialized widgets: column browser, icon grid, data grid, rule editor, wizard, settings dialog, find bar, and shortcuts editor.

## Part 13: Remaining Pushrod backlog

Pushrod issues not needed by Chassis, in roadmap order. Theming (Epic 8) stays last, as the distant future it was planned to be.

### Step 253: Callback handles: disconnect, block, and unblock

**Issue:** [#99](https://github.com/KenSuenobu/rust-pushrod/issues/99) `pushrod: [9.7] Callback handles: disconnect, block, and unblock`

**Delivers:** Every registration returns a handle to remove or temporarily silence the callback.

**Builds on:** step 11.

**Show on video:**

- [ ] Disconnect removes; block suppresses until unblock
- [ ] Scoped blocking restores state even on early return

### Step 254: Application lifecycle: startup and close-request callbacks

**Issue:** [#26](https://github.com/KenSuenobu/rust-pushrod/issues/26) `pushrod: [1.11] Application lifecycle: startup and close-request callbacks`

**Delivers:** Run code before the first frame; confirm or veto closing the window.

**Builds on:** step 4.

**Show on video:**

- [ ] `on_start` runs once before the first draw
- [ ] `on_close_request` can veto
- [ ] Tests via `Engine::dispatch`

### Step 255: Grid layout

**Issue:** [#119](https://github.com/KenSuenobu/rust-pushrod/issues/119) `pushrod: [5.8] Grid layout`

**Delivers:** Rows and columns of cells with spans, per-row/column stretch, and alignment.

**Builds on:** step 41.

**Mockups:** [5.8-1.png](../mockups/5.8-1.png)

**Show on video:**

- [ ] Spans, stretch, alignment, spacing
- [ ] Correct min/preferred sizes from cell hints

### Step 256: Switch (on/off slide toggle)

**Issue:** [#86](https://github.com/KenSuenobu/rust-pushrod/issues/86) `pushrod: [2.17] Switch (on/off slide toggle)`

**Delivers:** A sliding on/off switch for settings that take effect immediately.

**Builds on:** step 6, step 11.

**Mockups:** [2.17-1.png](../mockups/2.17-1.png)

**Show on video:**

- [ ] Click, drag, Space toggle

### Step 257: Message bar (non-modal inline notice)

**Issue:** [#89](https://github.com/KenSuenobu/rust-pushrod/issues/89) `pushrod: [2.20] Message bar (non-modal inline notice)`

**Delivers:** A dismissible banner for warnings and information that doesn't block the window.

**Builds on:** step 22, step 40, step 33.

**Mockups:** [2.20-1.png](../mockups/2.20-1.png)

**Show on video:**

- [ ] Shows, hides, and dismisses
- [ ] Action buttons fire callbacks
- [ ] Parts are retrievable and modifiable as described

### Step 258: Busy spinner

**Issue:** [#128](https://github.com/KenSuenobu/rust-pushrod/issues/128) `pushrod: [4.10] Busy spinner`

**Delivers:** A small animated indicator for work of unknown length, for use inline or on buttons.

**Builds on:** step 22.

**Mockups:** [4.10-1.png](../mockups/4.10-1.png)

**Show on video:**

- [ ] Animates only while running and visible
- [ ] Static fallback in outline mode
- [ ] Fits inline with text and inside buttons

### Step 259: Selectable label text

**Issue:** [#291](https://github.com/KenSuenobu/rust-pushrod/issues/291) `pushrod: [2.23] Selectable label text`

**Delivers:** Text in labels can be selected with the mouse or keyboard and copied.

**Builds on:** step 38, step 133, step 21.

**Mockups:** [2.23-1.png](../mockups/2.23-1.png)

**Show on video:**

- [ ] Mouse and keyboard selection, copy to the clipboard
- [ ] Works across wrapped lines
- [ ] No caret or editing; off by default

### Step 260: Count badge on buttons, tabs, and rows

**Issue:** [#292](https://github.com/KenSuenobu/rust-pushrod/issues/292) `pushrod: [2.24] Count badge on buttons, tabs, and rows`

**Delivers:** A small count or dot drawn on a corner of a button, tab, or list row.

**Builds on:** step 89, step 94, step 90.

**Mockups:** [2.24-1.png](../mockups/2.24-1.png)

**Show on video:**

- [ ] Count, dot, and text badges on image buttons, tabs, and list rows
- [ ] Zero hides it; large counts are capped
- [ ] Same look everywhere through one paint helper

### Step 261: Empty-state text for list, table, and other views

**Issue:** [#293](https://github.com/KenSuenobu/rust-pushrod/issues/293) `pushrod: [4.11] Empty-state text for list, table, and other views`

**Delivers:** A message shown in the middle of a view when it has no rows.

**Builds on:** step 89, step 91, step 21.

**Mockups:** [4.11-1.png](../mockups/4.11-1.png)

**Show on video:**

- [ ] Placeholder shown only when there are no rows
- [ ] Table headers stay visible
- [ ] Custom placeholder widget supported

### Step 262: Arithmetic in number fields

**Issue:** [#294](https://github.com/KenSuenobu/rust-pushrod/issues/294) `pushrod: [4.12] Arithmetic in number fields`

**Delivers:** Type a simple calculation into a number field and it is replaced by the result.

**Builds on:** step 38, step 208.

**Mockups:** [4.12-1.png](../mockups/4.12-1.png)

**Show on video:**

- [ ] +, -, *, / and parentheses evaluated on commit
- [ ] Result clamped and validated like typed numbers
- [ ] Off by default; invalid expressions marked

### Step 263: Dirty-rectangle redraw

**Issue:** [#24](https://github.com/KenSuenobu/rust-pushrod/issues/24) `pushrod: [1.9] Dirty-rectangle redraw`

**Delivers:** Only re-blit the screen regions that changed.

**Builds on:** step 46.

**Show on video:**

- [ ] One changed widget copies only intersecting textures
- [ ] Golden tests (7.2) pass
- [ ] Benchmark in the PR

### Step 264: Chassis: Code editor find

**Issue:** [#147](https://github.com/KenSuenobu/rust-pushrod/issues/147) `pushrod: [11.6] Chassis: Code editor find`

**Delivers:** A find bar inside the editor: type to search, step through matches.

**Builds on:** step 38, step 32, step 139.

**Mockups:** [11.6-1.png](../mockups/11.6-1.png)

**Show on video:**

- [ ] Matches update as the search text changes
- [ ] Stepping wraps around the file
- [ ] Escape returns focus to the editor at the current match

### Step 265: Chassis: Code editor auto-indent and bracket matching

**Issue:** [#148](https://github.com/KenSuenobu/rust-pushrod/issues/148) `pushrod: [11.7] Chassis: Code editor auto-indent and bracket matching`

**Delivers:** New lines keep and adjust indentation, and the bracket matching the one at the caret is boxed.

**Builds on:** step 141, step 139.

**Mockups:** [11.7-1.png](../mockups/11.7-1.png)

**Show on video:**

- [ ] Return after '{' indents one level
- [ ] Typing '}' on a blank line outdents it
- [ ] Matching brackets boxed; brackets in strings ignored

### Step 266: Chassis: Code editor documentation, example, and tests

**Issue:** [#150](https://github.com/KenSuenobu/rust-pushrod/issues/150) `pushrod: [11.9] Chassis: Code editor documentation, example, and tests`

**Delivers:** Document the code editor on the site, ship an example editor, and cover it with golden-image tests.

**Builds on:** step 46, step 142, step 264, step 265.

**Show on video:**

- [ ] Docs pages published with screenshots from the example
- [ ] Golden images for each mockup state
- [ ] Benchmarks in CI

### Step 267: Theme for colors and metrics (future)

**Issue:** [#74](https://github.com/KenSuenobu/rust-pushrod/issues/74) `pushrod: [8.1] Theme for colors and metrics (future)`

**Delivers:** Move hard-coded colors and metrics into a theme; the default reproduces today's look.

**Builds on:** step 46.

**Show on video:**

- [ ] (Future) classic theme matches goldens

### Step 268: Skins: image-based widget rendering (future)

**Issue:** [#75](https://github.com/KenSuenobu/rust-pushrod/issues/75) `pushrod: [8.2] Skins: image-based widget rendering (future)`

**Delivers:** Let a skin replace paint with nine-slice images.

**Builds on:** step 267.

**Show on video:**

- [ ] (Future) sample skin renders the gallery
