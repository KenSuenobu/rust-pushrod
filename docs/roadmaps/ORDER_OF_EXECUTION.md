# Order of Execution

The complete plan of action for Pushrod and Chassis: **every open issue in the exact order to do it**, one step at a time, from the first Pushrod commit to Chassis v2. Each step is one piece of work, one pull request, and one video.

**193 steps:** 184 issues and 9 checkpoints (releases and phase demos). The order is generated from the roadmaps and checked: every step comes after everything it depends on.

## How to use this document

- **Work top to bottom.** Don't skip ahead: later steps assume earlier ones are merged.
- **One step, one pull request, one video.** Each step lists what it delivers, which earlier steps it builds on, and an on-camera checklist taken from the issue's acceptance criteria. Show each item working in the video, and the step is done.
- **Done steps** keep their number (marked *done*), so the series numbering never shifts; their videos can be recaps.
- **Checkpoints** are episodes of their own: a release or the end of a phase, demonstrating everything built so far.
- **Every step ships its documentation** (Docusaurus pages, full doc headers, changelog entry), as CONTRIBUTING.md requires.
- **Suggested video title:** `Pushrod #<step>: <title>` (or `Chassis #<step>: ...`), so the series sorts in order on YouTube.

Sources: [Pushrod roadmap](../ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md), [Chassis roadmap](../ROADMAP_CHASSIS.md) (Action Plan), [GitHub projects](../PROJECTS.md). When a roadmap changes, regenerate this file.

## Overview

| Part | Steps | Result |
|---|---|---|
| [Part 1: Pushrod v0.1, dialog applications](#part-1-pushrod-v01-dialog-applications) | 1 to 54 (54) | Everything needed to build and ship real dialog-style applications with Pushrod, ending with the first crates.io release. |
| [Part 2: Pushrod v0.2, forms and resources](#part-2-pushrod-v02-forms-and-resources) | 55 to 68 (14) | Forms, form files, and resources by ID (the runtime foundation for Chassis), plus input methods, mnemonics, accessibility, and specs. |
| [Part 3: Chassis M1: core without UI](#part-3-chassis-m1-core-without-ui) | 69 to 74 (6) | Project model, resource access, code generation, and a command-line generator, all testable without a window. |
| [Part 4: Chassis M2: shell and project](#part-4-chassis-m2-shell-and-project) | 75 to 100 (26) | The main window, three panes, Designer/Code tabs, commands, and the project drawer with resource editors. |
| [Part 5: Chassis M3: form designer](#part-5-chassis-m3-form-designer) | 101 to 113 (13) | Design surface, palette, drag and drop, selection, snapping, containers, layouts, and undo. |
| [Part 6: Chassis M4: inspector, events, and code](#part-6-chassis-m4-inspector-events-and-code) | 114 to 133 (20) | Property inspector, events menu and indicators, handler creation, and the code view. |
| [Part 7: Chassis M5: build, run, and release (MVP)](#part-7-chassis-m5-build-run-and-release-mvp) | 134 to 138 (5) | Build and run from Chassis, CI, documentation, and the sample project. |
| [Part 8: Chassis example projects](#part-8-chassis-example-projects) | 139 to 147 (9) | Example projects built with Chassis once it reaches the MVP: small, complete applications that exercise its features and that anyone can build and test against. |
| [Part 9: Chassis v2](#part-9-chassis-v2) | 148 to 170 (23) | The second release: settings, menu editor, form preview, multi-selection, error list, templates, packaging, code intelligence, localization, form inheritance, bindings, project search, multiple windows, accessibility checks, source control, and more build targets. |
| [Part 10: Remaining Pushrod backlog](#part-10-remaining-pushrod-backlog) | 171 to 193 (23) | Pushrod issues not needed by Chassis, in roadmap order. |

## Step index

| Step | Issue | Title |
|---|---|---|
| ✅ 1 | [#130](https://github.com/KenSuenobu/rust-pushrod/issues/130) | pushrod: [1.21] Split the standard widgets into the pushrod-widgets crate (done) |
| ✅ 2 | [#69](https://github.com/KenSuenobu/rust-pushrod/issues/69) | pushrod: [7.1] CI: build, test, clippy, fmt, doc with SDL2 (done) |
| ✅ 3 | [#122](https://github.com/KenSuenobu/rust-pushrod/issues/122) | pushrod: [7.8] Documentation site: build pushrod-docs in CI and publish it (done) |
| 4 | [#93](https://github.com/KenSuenobu/rust-pushrod/issues/93) | pushrod: [9.1] Event vocabulary: modern event names and structure |
| 5 | [#106](https://github.com/KenSuenobu/rust-pushrod/issues/106) | pushrod: [10.2] Backend-neutral paint API (SDL2 backend by default) |
| 6 | [#19](https://github.com/KenSuenobu/rust-pushrod/issues/19) | pushrod: [1.1] Widget behavior flags and visual states |
| 7 | [#6](https://github.com/KenSuenobu/rust-pushrod/issues/6) | pushrod: [1.2] Event masks for the run loop and for widgets |
| 8 | [#109](https://github.com/KenSuenobu/rust-pushrod/issues/109) | pushrod: [9.10] Engine events: every internal action Pushrod performs emits an event |
| 9 | [#124](https://github.com/KenSuenobu/rust-pushrod/issues/124) | pushrod: [9.11] Event-loop style: keep parity with callbacks |
| 10 | [#7](https://github.com/KenSuenobu/rust-pushrod/issues/7) | pushrod: [1.3] Broadcast events to all interested widgets |
| 11 | [#22](https://github.com/KenSuenobu/rust-pushrod/issues/22) | pushrod: [1.6] Callback registry for every widget event |
| 12 | [#78](https://github.com/KenSuenobu/rust-pushrod/issues/78) | pushrod: [1.13] Custom application events with payloads |
| 13 | [#111](https://github.com/KenSuenobu/rust-pushrod/issues/111) | pushrod: [1.18] Text layout engine: Parley for shaping, bidi, line breaking, and font fallback (with an SDL2 compatibility gate) |
| 14 | [#3](https://github.com/KenSuenobu/rust-pushrod/issues/3) | pushrod: [1.7] GPU text: font cache with a glyph atlas texture |
| 15 | [#71](https://github.com/KenSuenobu/rust-pushrod/issues/71) | pushrod: [7.3] Widget gallery example |
| 16 | [#94](https://github.com/KenSuenobu/rust-pushrod/issues/94) | pushrod: [9.2] Event propagation: capture, target, and bubble phases with handled/ignored |
| 17 | [#96](https://github.com/KenSuenobu/rust-pushrod/issues/96) | pushrod: [9.4] Reusable input controllers: click, drag, scroll, hover, long-press, key |
| 18 | [#20](https://github.com/KenSuenobu/rust-pushrod/issues/20) | pushrod: [1.4] Keyboard focus, Tab order, and Return/Escape |
| 19 | [#104](https://github.com/KenSuenobu/rust-pushrod/issues/104) | pushrod: [1.17] Per-state appearance: colors, border, and font for each visual state |
| 20 | [#21](https://github.com/KenSuenobu/rust-pushrod/issues/21) | pushrod: [1.5] Timers and idle callbacks |
| 21 | [#134](https://github.com/KenSuenobu/rust-pushrod/issues/134) | pushrod: [10.6] Backend crates: keep SDL out of the core and the widgets |
| 22 | [#138](https://github.com/KenSuenobu/rust-pushrod/issues/138) | pushrod: [1.25] Errors: return Result instead of panicking |
| 23 | [#137](https://github.com/KenSuenobu/rust-pushrod/issues/137) | pushrod: [1.24] Fonts: shared font handles and a built-in default font |
| 24 | [#136](https://github.com/KenSuenobu/rust-pushrod/issues/136) | pushrod: [1.23] Public API polish before publishing: builders, prelude, and stability |
| 25 | [#36](https://github.com/KenSuenobu/rust-pushrod/issues/36) | pushrod: [2.10] Custom paint widget (closure-drawn) |
| 26 | [#27](https://github.com/KenSuenobu/rust-pushrod/issues/27) | pushrod: [2.1] Boxed label widget |
| 27 | [#83](https://github.com/KenSuenobu/rust-pushrod/issues/83) | pushrod: [2.14] Separator line |
| 28 | [#41](https://github.com/KenSuenobu/rust-pushrod/issues/41) | pushrod: [3.3] Group box (titled frame) |
| 29 | [#33](https://github.com/KenSuenobu/rust-pushrod/issues/33) | pushrod: [2.7] Image widget |
| 30 | [#28](https://github.com/KenSuenobu/rust-pushrod/issues/28) | pushrod: [2.2] Glyph button (small single-character button) |
| 31 | [#29](https://github.com/KenSuenobu/rust-pushrod/issues/29) | pushrod: [2.3] Button behaviors: default, cancel, toggle, activate-on-press, dismiss dialog |
| 32 | [#30](https://github.com/KenSuenobu/rust-pushrod/issues/30) | pushrod: [2.4] Check box with indeterminate state |
| 33 | [#31](https://github.com/KenSuenobu/rust-pushrod/issues/31) | pushrod: [2.5] Radio buttons and radio groups |
| 34 | [#48](https://github.com/KenSuenobu/rust-pushrod/issues/48) | pushrod: [4.1] Progress bar |
| 35 | [#40](https://github.com/KenSuenobu/rust-pushrod/issues/40) | pushrod: [3.2] Slider |
| 36 | [#32](https://github.com/KenSuenobu/rust-pushrod/issues/32) | pushrod: [2.6] Text field: single-line entry with input masks, validation, and numeric mode |
| 37 | [#56](https://github.com/KenSuenobu/rust-pushrod/issues/56) | pushrod: [5.1] Clip children to their parent's bounds |
| 38 | [#57](https://github.com/KenSuenobu/rust-pushrod/issues/57) | pushrod: [5.2] Composite widgets: build widgets from parts that applications can retrieve and modify |
| 39 | [#116](https://github.com/KenSuenobu/rust-pushrod/issues/116) | pushrod: [5.5] Layout manager foundation: Layout trait, size hints, measure and arrange |
| 40 | [#117](https://github.com/KenSuenobu/rust-pushrod/issues/117) | pushrod: [5.6] Row and column layouts |
| 41 | [#118](https://github.com/KenSuenobu/rust-pushrod/issues/118) | pushrod: [5.7] Form layout: aligned label and field rows |
| 42 | [#39](https://github.com/KenSuenobu/rust-pushrod/issues/39) | pushrod: [3.1] Scroll bar |
| 43 | [#58](https://github.com/KenSuenobu/rust-pushrod/issues/58) | pushrod: [5.3] Container: a scrollable viewport onto fixed-size content |
| 44 | [#70](https://github.com/KenSuenobu/rust-pushrod/issues/70) | pushrod: [7.2] Golden-image rendering tests |
| 45 | [#115](https://github.com/KenSuenobu/rust-pushrod/issues/115) | pushrod: [7.7] Performance benchmarks and budgets |
| 46 | [#25](https://github.com/KenSuenobu/rust-pushrod/issues/25) | pushrod: [1.10] High-DPI rendering and window resize |
| 47 | [#139](https://github.com/KenSuenobu/rust-pushrod/issues/139) | pushrod: [7.9] Easy installation: bundled SDL, static linking, and platform guides |
| 48 | [#23](https://github.com/KenSuenobu/rust-pushrod/issues/23) | pushrod: [1.8] Edit the widget tree at run time: remove, reparent, reorder |
| 49 | [#66](https://github.com/KenSuenobu/rust-pushrod/issues/66) | pushrod: [6.7] Modal dialogs: run a form until a dismissing control is chosen |
| 50 | [#35](https://github.com/KenSuenobu/rust-pushrod/issues/35) | pushrod: [2.9] Alert dialog |
| 51 | [#140](https://github.com/KenSuenobu/rust-pushrod/issues/140) | pushrod: [7.10] Reference application: a real app built only with Pushrod |
| 52 | [#72](https://github.com/KenSuenobu/rust-pushrod/issues/72) | pushrod: [7.4] Guide: writing widgets and composites |
| 53 | [#73](https://github.com/KenSuenobu/rust-pushrod/issues/73) | pushrod: [7.5] Changelog, Cargo metadata, and crates.io release |
| **54** | checkpoint | **Pushrod v0.1 released** |
| 55 | [#114](https://github.com/KenSuenobu/rust-pushrod/issues/114) | pushrod: [7.6] Module boundaries and written specs with conformance tests |
| 56 | [#112](https://github.com/KenSuenobu/rust-pushrod/issues/112) | pushrod: [1.19] Input method (IME) support: composition text and candidate window |
| 57 | [#76](https://github.com/KenSuenobu/rust-pushrod/issues/76) | pushrod: [1.12] Keyboard mnemonics: underlined shortcut letters |
| 58 | [#60](https://github.com/KenSuenobu/rust-pushrod/issues/60) | pushrod: [6.1] Widget registry and factory |
| 59 | [#61](https://github.com/KenSuenobu/rust-pushrod/issues/61) | pushrod: [6.2] Property introspection: list, get, and set widget properties by name |
| 60 | [#80](https://github.com/KenSuenobu/rust-pushrod/issues/80) | pushrod: [1.14] Accessibility: expose widgets to screen readers |
| 61 | [#62](https://github.com/KenSuenobu/rust-pushrod/issues/62) | pushrod: [6.3] Form file format (binary, versioned) |
| 62 | [#63](https://github.com/KenSuenobu/rust-pushrod/issues/63) | pushrod: [6.4] Form loader: build a form from a file with stable IDs |
| 63 | [#64](https://github.com/KenSuenobu/rust-pushrod/issues/64) | pushrod: [6.5] Form writer: save a widget subtree to a file |
| 64 | [#65](https://github.com/KenSuenobu/rust-pushrod/issues/65) | pushrod: [6.6] Generate ID constants from a form file |
| 65 | [#154](https://github.com/KenSuenobu/rust-pushrod/issues/154) | pushrod: [6.11] Resource kinds: strings, images, icons, files, menus, and alerts |
| 66 | [#155](https://github.com/KenSuenobu/rust-pushrod/issues/155) | pushrod: [6.12] Typed resource IDs: generated constants per resource kind |
| 67 | [#153](https://github.com/KenSuenobu/rust-pushrod/issues/153) | pushrod: [6.10] Resource manager: load resource sets and get resources by ID |
| **68** | checkpoint | **Pushrod v0.2 released: the Pushrod MVP is complete** |
| 69 | [#156](https://github.com/KenSuenobu/rust-pushrod/issues/156) | pushrod: [6.13] Event metadata in the widget registry |
| 70 | [#164](https://github.com/KenSuenobu/rust-pushrod/issues/164) | chassis: [2.1] Project model |
| 71 | [#165](https://github.com/KenSuenobu/rust-pushrod/issues/165) | chassis: [6.1] Code generation rules |
| 72 | [#166](https://github.com/KenSuenobu/rust-pushrod/issues/166) | chassis: [5.7] Event wiring generation |
| 73 | [#167](https://github.com/KenSuenobu/rust-pushrod/issues/167) | chassis: [6.5] Command-line generator |
| **74** | checkpoint | **Chassis M1: core without UI complete** |
| 75 | [#97](https://github.com/KenSuenobu/rust-pushrod/issues/97) | pushrod: [9.5] Actions and keyboard shortcuts: shared commands for menus, toolbars, and buttons |
| 76 | [#100](https://github.com/KenSuenobu/rust-pushrod/issues/100) | pushrod: [9.8] Drag and drop within the application |
| 77 | [#81](https://github.com/KenSuenobu/rust-pushrod/issues/81) | pushrod: [1.15] Undo and redo command stack |
| 78 | [#59](https://github.com/KenSuenobu/rust-pushrod/issues/59) | pushrod: [5.4] Autoresizing (keep margins or stretch on resize) |
| 79 | [#52](https://github.com/KenSuenobu/rust-pushrod/issues/52) | pushrod: [4.5] List view |
| 80 | [#51](https://github.com/KenSuenobu/rust-pushrod/issues/51) | pushrod: [4.4] Tab view |
| 81 | [#92](https://github.com/KenSuenobu/rust-pushrod/issues/92) | pushrod: [4.9] Table view |
| 82 | [#90](https://github.com/KenSuenobu/rust-pushrod/issues/90) | pushrod: [3.10] Expander (collapsible section) |
| 83 | [#37](https://github.com/KenSuenobu/rust-pushrod/issues/37) | pushrod: [2.11] Menu bar and drop-down menus (drawn by Pushrod) |
| 84 | [#125](https://github.com/KenSuenobu/rust-pushrod/issues/125) | pushrod: [2.21] Image button |
| 85 | [#84](https://github.com/KenSuenobu/rust-pushrod/issues/84) | pushrod: [2.15] Toolbar |
| 86 | [#85](https://github.com/KenSuenobu/rust-pushrod/issues/85) | pushrod: [2.16] Status bar |
| 87 | [#34](https://github.com/KenSuenobu/rust-pushrod/issues/34) | pushrod: [2.8] Icon widget (picture with caption) |
| 88 | [#38](https://github.com/KenSuenobu/rust-pushrod/issues/38) | pushrod: [2.12] File chooser dialog (drawn by Pushrod) |
| 89 | [#47](https://github.com/KenSuenobu/rust-pushrod/issues/47) | pushrod: [3.9] Split view (panes with draggable dividers) |
| 90 | [#168](https://github.com/KenSuenobu/rust-pushrod/issues/168) | chassis: [1.1] Chassis crate and main window |
| 91 | [#169](https://github.com/KenSuenobu/rust-pushrod/issues/169) | chassis: [1.2] Three-pane workspace |
| 92 | [#170](https://github.com/KenSuenobu/rust-pushrod/issues/170) | chassis: [1.4] Commands, menus, and shortcuts |
| 93 | [#171](https://github.com/KenSuenobu/rust-pushrod/issues/171) | chassis: [1.3] Designer and Code tabs |
| 94 | [#172](https://github.com/KenSuenobu/rust-pushrod/issues/172) | chassis: [2.2] New, open, and save |
| 95 | [#173](https://github.com/KenSuenobu/rust-pushrod/issues/173) | chassis: [2.3] Project drawer |
| 96 | [#174](https://github.com/KenSuenobu/rust-pushrod/issues/174) | chassis: [2.4] Manage resources and IDs |
| 97 | [#175](https://github.com/KenSuenobu/rust-pushrod/issues/175) | chassis: [2.5] String table editor |
| 98 | [#176](https://github.com/KenSuenobu/rust-pushrod/issues/176) | chassis: [2.6] Images and icons |
| 99 | [#177](https://github.com/KenSuenobu/rust-pushrod/issues/177) | chassis: [2.8] Files |
| **100** | checkpoint | **Chassis M2: shell and project complete** |
| 101 | [#95](https://github.com/KenSuenobu/rust-pushrod/issues/95) | pushrod: [9.3] Event filters: observe or intercept another widget's events |
| 102 | [#105](https://github.com/KenSuenobu/rust-pushrod/issues/105) | pushrod: [10.1] Drawing modes: live or outline feedback while moving and resizing |
| 103 | [#68](https://github.com/KenSuenobu/rust-pushrod/issues/68) | pushrod: [6.9] Design-mode hooks for Chassis |
| 104 | [#131](https://github.com/KenSuenobu/rust-pushrod/issues/131) | pushrod: [1.22] Widget libraries: extend or replace pushrod-widgets |
| 105 | [#178](https://github.com/KenSuenobu/rust-pushrod/issues/178) | chassis: [3.1] Design surface |
| 106 | [#179](https://github.com/KenSuenobu/rust-pushrod/issues/179) | chassis: [3.2] Widget palette |
| 107 | [#180](https://github.com/KenSuenobu/rust-pushrod/issues/180) | chassis: [3.4] Selection, move, and resize |
| 108 | [#181](https://github.com/KenSuenobu/rust-pushrod/issues/181) | chassis: [3.3] Drag and drop placement |
| 109 | [#182](https://github.com/KenSuenobu/rust-pushrod/issues/182) | chassis: [3.5] Snapping and alignment |
| 110 | [#183](https://github.com/KenSuenobu/rust-pushrod/issues/183) | chassis: [3.7] Clipboard and undo |
| 111 | [#184](https://github.com/KenSuenobu/rust-pushrod/issues/184) | chassis: [3.6] Containers, z-order, and tab order |
| 112 | [#185](https://github.com/KenSuenobu/rust-pushrod/issues/185) | chassis: [3.8] Layout managers in the designer |
| **113** | checkpoint | **Chassis M3: form designer complete** |
| 114 | [#44](https://github.com/KenSuenobu/rust-pushrod/issues/44) | pushrod: [3.6] Choice button: pop-up list and click-to-cycle modes |
| 115 | [#88](https://github.com/KenSuenobu/rust-pushrod/issues/88) | pushrod: [2.19] Context menus |
| 116 | [#113](https://github.com/KenSuenobu/rust-pushrod/issues/113) | pushrod: [1.20] Layers and invalidation: cache hints, texture memory budget, incremental pipeline |
| 117 | [#142](https://github.com/KenSuenobu/rust-pushrod/issues/142) | pushrod: [11.1] Chassis: Code editor crate and editing core |
| 118 | [#143](https://github.com/KenSuenobu/rust-pushrod/issues/143) | pushrod: [11.2] Chassis: Code editor view: gutter, visible-line rendering, caret |
| 119 | [#144](https://github.com/KenSuenobu/rust-pushrod/issues/144) | pushrod: [11.3] Chassis: Code editor input: keyboard, mouse selection, clipboard, input methods |
| 120 | [#145](https://github.com/KenSuenobu/rust-pushrod/issues/145) | pushrod: [11.4] Chassis: Code editor undo and redo |
| 121 | [#146](https://github.com/KenSuenobu/rust-pushrod/issues/146) | pushrod: [11.5] Chassis: Code editor syntax highlighting with tree-sitter |
| 122 | [#149](https://github.com/KenSuenobu/rust-pushrod/issues/149) | pushrod: [11.8] Chassis: Code editor API for the Chassis designer |
| 123 | [#186](https://github.com/KenSuenobu/rust-pushrod/issues/186) | chassis: [4.1] Inspector drawer |
| 124 | [#187](https://github.com/KenSuenobu/rust-pushrod/issues/187) | chassis: [4.2] Property grid |
| 125 | [#188](https://github.com/KenSuenobu/rust-pushrod/issues/188) | chassis: [4.3] Typed property editors |
| 126 | [#189](https://github.com/KenSuenobu/rust-pushrod/issues/189) | chassis: [5.4] Code view |
| 127 | [#190](https://github.com/KenSuenobu/rust-pushrod/issues/190) | chassis: [5.3] Create and open handlers |
| 128 | [#191](https://github.com/KenSuenobu/rust-pushrod/issues/191) | chassis: [4.4] Names and IDs |
| 129 | [#192](https://github.com/KenSuenobu/rust-pushrod/issues/192) | chassis: [5.1] Events context menu |
| 130 | [#193](https://github.com/KenSuenobu/rust-pushrod/issues/193) | chassis: [5.2] Inspector Events tab |
| 131 | [#194](https://github.com/KenSuenobu/rust-pushrod/issues/194) | chassis: [5.5] Code indicator on the design surface |
| 132 | [#195](https://github.com/KenSuenobu/rust-pushrod/issues/195) | chassis: [5.6] Keep handlers in sync |
| **133** | checkpoint | **Chassis M4: inspector, events, and code complete** |
| 134 | [#196](https://github.com/KenSuenobu/rust-pushrod/issues/196) | chassis: [6.2] Build and run |
| 135 | [#197](https://github.com/KenSuenobu/rust-pushrod/issues/197) | chassis: [7.1] CI and golden screenshots |
| 136 | [#198](https://github.com/KenSuenobu/rust-pushrod/issues/198) | chassis: [7.2] Chassis documentation |
| 137 | [#199](https://github.com/KenSuenobu/rust-pushrod/issues/199) | chassis: [7.3] Sample project |
| **138** | checkpoint | **Chassis MVP released** |
| 139 | [#208](https://github.com/KenSuenobu/rust-pushrod/issues/208) | chassis: [8.1] Hello, Chassis |
| 140 | [#209](https://github.com/KenSuenobu/rust-pushrod/issues/209) | chassis: [8.2] Temperature converter |
| 141 | [#210](https://github.com/KenSuenobu/rust-pushrod/issues/210) | chassis: [8.3] Tip calculator |
| 142 | [#211](https://github.com/KenSuenobu/rust-pushrod/issues/211) | chassis: [8.4] Settings dialog |
| 143 | [#212](https://github.com/KenSuenobu/rust-pushrod/issues/212) | chassis: [8.5] Image viewer |
| 144 | [#213](https://github.com/KenSuenobu/rust-pushrod/issues/213) | chassis: [8.6] Stopwatch |
| 145 | [#214](https://github.com/KenSuenobu/rust-pushrod/issues/214) | chassis: [8.7] Quiz |
| 146 | [#215](https://github.com/KenSuenobu/rust-pushrod/issues/215) | chassis: [8.8] Dice roller |
| **147** | checkpoint | **All example projects built** |
| 148 | [#98](https://github.com/KenSuenobu/rust-pushrod/issues/98) | pushrod: [9.6] Property bindings: keep two properties in sync |
| 149 | [#82](https://github.com/KenSuenobu/rust-pushrod/issues/82) | pushrod: [1.16] Multiple windows: separate OS windows, each with its own engine |
| 150 | [#67](https://github.com/KenSuenobu/rust-pushrod/issues/67) | pushrod: [6.8] Bind actions by name from form files |
| 151 | [#107](https://github.com/KenSuenobu/rust-pushrod/issues/107) | pushrod: [10.3] no_std + alloc core crate |
| 152 | [#108](https://github.com/KenSuenobu/rust-pushrod/issues/108) | pushrod: [10.4] Experimental: Atari ST / TOS backend |
| 153 | [#135](https://github.com/KenSuenobu/rust-pushrod/issues/135) | pushrod: [10.5] SDL3 backend (pushrod-sdl3) |
| 154 | [#200](https://github.com/KenSuenobu/rust-pushrod/issues/200) | chassis: [1.5] Workspace settings |
| 155 | [#201](https://github.com/KenSuenobu/rust-pushrod/issues/201) | chassis: [2.7] Menu editor |
| 156 | [#202](https://github.com/KenSuenobu/rust-pushrod/issues/202) | chassis: [3.9] Form preview |
| 157 | [#203](https://github.com/KenSuenobu/rust-pushrod/issues/203) | chassis: [4.5] Multiple selection editing |
| 158 | [#204](https://github.com/KenSuenobu/rust-pushrod/issues/204) | chassis: [6.3] Error list |
| 159 | [#205](https://github.com/KenSuenobu/rust-pushrod/issues/205) | chassis: [6.4] Project templates |
| 160 | [#206](https://github.com/KenSuenobu/rust-pushrod/issues/206) | chassis: [7.4] Packaging and releases |
| 161 | [#216](https://github.com/KenSuenobu/rust-pushrod/issues/216) | chassis: [1.6] Find in project |
| 162 | [#217](https://github.com/KenSuenobu/rust-pushrod/issues/217) | chassis: [1.7] Multiple windows per project |
| 163 | [#218](https://github.com/KenSuenobu/rust-pushrod/issues/218) | chassis: [2.9] Localization |
| 164 | [#219](https://github.com/KenSuenobu/rust-pushrod/issues/219) | chassis: [2.10] Source control status |
| 165 | [#220](https://github.com/KenSuenobu/rust-pushrod/issues/220) | chassis: [3.10] Visual form inheritance |
| 166 | [#221](https://github.com/KenSuenobu/rust-pushrod/issues/221) | chassis: [3.11] Accessibility and usability checks |
| 167 | [#222](https://github.com/KenSuenobu/rust-pushrod/issues/222) | chassis: [4.6] Data bindings in the inspector |
| 168 | [#223](https://github.com/KenSuenobu/rust-pushrod/issues/223) | chassis: [5.8] Code intelligence with rust-analyzer |
| 169 | [#224](https://github.com/KenSuenobu/rust-pushrod/issues/224) | chassis: [6.6] Build targets, including the Atari ST |
| **170** | checkpoint | **Chassis v2 released** |
| 171 | [#99](https://github.com/KenSuenobu/rust-pushrod/issues/99) | pushrod: [9.7] Callback handles: disconnect, block, and unblock |
| 172 | [#101](https://github.com/KenSuenobu/rust-pushrod/issues/101) | pushrod: [9.9] Pointer cursor per widget |
| 173 | [#26](https://github.com/KenSuenobu/rust-pushrod/issues/26) | pushrod: [1.11] Application lifecycle: startup and close-request callbacks |
| 174 | [#119](https://github.com/KenSuenobu/rust-pushrod/issues/119) | pushrod: [5.8] Grid layout |
| 175 | [#120](https://github.com/KenSuenobu/rust-pushrod/issues/120) | pushrod: [5.9] Stack layout (one child visible at a time) |
| 176 | [#121](https://github.com/KenSuenobu/rust-pushrod/issues/121) | pushrod: [5.10] Flow layout (wrapping rows) |
| 177 | [#45](https://github.com/KenSuenobu/rust-pushrod/issues/45) | pushrod: [3.7] Column browser (multi-column hierarchical browser) |
| 178 | [#49](https://github.com/KenSuenobu/rust-pushrod/issues/49) | pushrod: [4.2] Spin box |
| 179 | [#53](https://github.com/KenSuenobu/rust-pushrod/issues/53) | pushrod: [4.6] Combo box |
| 180 | [#91](https://github.com/KenSuenobu/rust-pushrod/issues/91) | pushrod: [3.11] Date picker |
| 181 | [#77](https://github.com/KenSuenobu/rust-pushrod/issues/77) | pushrod: [2.13] Tooltips |
| 182 | [#86](https://github.com/KenSuenobu/rust-pushrod/issues/86) | pushrod: [2.17] Switch (on/off slide toggle) |
| 183 | [#87](https://github.com/KenSuenobu/rust-pushrod/issues/87) | pushrod: [2.18] Multi-line text area |
| 184 | [#89](https://github.com/KenSuenobu/rust-pushrod/issues/89) | pushrod: [2.20] Message bar (non-modal inline notice) |
| 185 | [#126](https://github.com/KenSuenobu/rust-pushrod/issues/126) | pushrod: [2.22] Link label |
| 186 | [#127](https://github.com/KenSuenobu/rust-pushrod/issues/127) | pushrod: [3.12] Dial (rotary knob) |
| 187 | [#128](https://github.com/KenSuenobu/rust-pushrod/issues/128) | pushrod: [4.10] Busy spinner |
| 188 | [#24](https://github.com/KenSuenobu/rust-pushrod/issues/24) | pushrod: [1.9] Dirty-rectangle redraw |
| 189 | [#147](https://github.com/KenSuenobu/rust-pushrod/issues/147) | pushrod: [11.6] Chassis: Code editor find |
| 190 | [#148](https://github.com/KenSuenobu/rust-pushrod/issues/148) | pushrod: [11.7] Chassis: Code editor auto-indent and bracket matching |
| 191 | [#150](https://github.com/KenSuenobu/rust-pushrod/issues/150) | pushrod: [11.9] Chassis: Code editor documentation, example, and tests |
| 192 | [#74](https://github.com/KenSuenobu/rust-pushrod/issues/74) | pushrod: [8.1] Theme for colors and metrics (future) |
| 193 | [#75](https://github.com/KenSuenobu/rust-pushrod/issues/75) | pushrod: [8.2] Skins: image-based widget rendering (future) |

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

- [ ] `pushrod::Event` with the variants above; `EventKind`; `widget()`
- [ ] PR #10 behavior and tests ported to the new names
- [ ] `ButtonWidget` uses `PointerPressed/Released/Entered/Exited`
- [ ] Rustdoc lists every event with when it is emitted

### Step 5: Backend-neutral paint API (SDL2 backend by default)

**Issue:** [#106](https://github.com/KenSuenobu/rust-pushrod/issues/106) `pushrod: [10.2] Backend-neutral paint API (SDL2 backend by default)`

**Delivers:** Widgets paint through a small Painter trait instead of an SDL2 canvas, so other backends become possible without touching widgets.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [ ] All widgets paint through `Painter`; no widget imports `sdl2::render`
- [ ] SDL2 backend renders pixel-identical goldens
- [ ] A headless test backend records paint calls for unit tests
- [ ] Cache policy is backend-selected

### Step 6: Widget behavior flags and visual states

**Issue:** [#19](https://github.com/KenSuenobu/rust-pushrod/issues/19) `pushrod: [1.1] Widget behavior flags and visual states`

**Delivers:** Behavior flags (toggle, default, cancel, exclusive, focusable, ...) and visual states (pressed, checked, disabled, ...) on every widget.

**Builds on:** step 4.

**Mockups:** [1.1-1.png](../mockups/1.1-1.png)

**Show on video:**

- [ ] `Behavior` and `VisualState` exist on `WidgetCore` with default trait accessors
- [ ] `set_hidden`/`is_hidden` are implemented on `HIDDEN` (no API break)
- [ ] Disabled widgets are skipped by input routing and focus
- [ ] `paint::state_overlay` draws DISABLED, FOCUSED, and INDETERMINATE after `paint()`, using a cached GPU dither texture
- [ ] Tests for flags, states, and disabled routing

### Step 7: Event masks for the run loop and for widgets

**Issue:** [#6](https://github.com/KenSuenobu/rust-pushrod/issues/6) `pushrod: [1.2] Event masks for the run loop and for widgets`

**Delivers:** Filter events for the whole run loop and per widget, so only events in the mask are handled.

**Builds on:** step 4.

**Show on video:**

- [ ] Run-loop mask filters before routing and callbacks; changes apply on the next event
- [ ] Masked SDL event types are disabled in SDL and re-enabled when the mask changes
- [ ] Close requests are always delivered
- [ ] Idle loop blocks on `wait_event_timeout` (near-zero idle CPU)
- [ ] Widgets never receive events outside their mask; a `BoxWidget` receives nothing while a button still clicks

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

- [ ] Spike passes the compatibility gate (or Parley is feature-gated off by default with the reason recorded in the issue)
- [ ] Glyphs rasterized once into the SDL2 atlas; steady-state drawing is texture copies only
- [ ] RTL, CJK, and emoji fallback render correctly; caret/selection geometry drives text fields
- [ ] Layouts cached; recomputed only on text/font/width change; provide size hints
- [ ] `TextLayout` trait with Parley as default once the gate passes

### Step 14: GPU text: font cache with a glyph atlas texture

**Issue:** [#3](https://github.com/KenSuenobu/rust-pushrod/issues/3) `pushrod: [1.7] GPU text: font cache with a glyph atlas texture`

**Delivers:** Load each font once, rasterize glyphs once into a GPU atlas, and draw text by blitting glyphs.

**Builds on:** step 13.

**Show on video:**

- [ ] Each font is loaded at most once per process
- [ ] After warm-up, repainting text performs only texture copies
- [ ] Text color changes use color modulation
- [ ] `FontCache` is owned by `WidgetCache`

### Step 15: Widget gallery example

**Issue:** [#71](https://github.com/KenSuenobu/rust-pushrod/issues/71) `pushrod: [7.3] Widget gallery example`

**Delivers:** One window showing every widget and state.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [ ] Shows every merged widget
- [ ] README screenshot from it

### Step 16: Event propagation: capture, target, and bubble phases with handled/ignored

**Issue:** [#94](https://github.com/KenSuenobu/rust-pushrod/issues/94) `pushrod: [9.2] Event propagation: capture, target, and bubble phases with handled/ignored`

**Delivers:** Input events travel from the window down to the target widget and back up to its parents, and any widget can mark them handled.

**Builds on:** step 4.

**Show on video:**

- [ ] Capture, target, bubble order tested
- [ ] `Propagation::Stop` ends delivery
- [ ] Return in a text field reaches the dialog's default-button handling

### Step 17: Reusable input controllers: click, drag, scroll, hover, long-press, key

**Issue:** [#96](https://github.com/KenSuenobu/rust-pushrod/issues/96) `pushrod: [9.4] Reusable input controllers: click, drag, scroll, hover, long-press, key`

**Delivers:** Composable input handlers that widgets attach instead of re-implementing press/drag/release logic.

**Builds on:** step 4, step 16.

**Show on video:**

- [ ] Controllers for click, drag, scroll, hover, long-press, key
- [ ] Button ported with identical behavior (PR #10 tests pass)
- [ ] Slider and scroll bar use DragController

### Step 18: Keyboard focus, Tab order, and Return/Escape

**Issue:** [#20](https://github.com/KenSuenobu/rust-pushrod/issues/20) `pushrod: [1.4] Keyboard focus, Tab order, and Return/Escape`

**Delivers:** Focusable widgets take keyboard focus; Tab moves it; Return and Escape trigger the default and cancel buttons.

**Builds on:** step 6, step 4, step 16.

**Show on video:**

- [ ] Focus moves by click, Tab, Shift-Tab, and arrows within groups
- [ ] Return/Escape activate default/cancel
- [ ] `FocusGained`/`FocusLost` delivered and tested
- [ ] `Ui::set_focus`/`Ui::focused`
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 19: Per-state appearance: colors, border, and font for each visual state

**Issue:** [#104](https://github.com/KenSuenobu/rust-pushrod/issues/104) `pushrod: [1.17] Per-state appearance: colors, border, and font for each visual state`

**Delivers:** Every widget carries appearance attributes for normal, pressed, hovered, focused, disabled, and drag-over, so the look changes automatically as the state changes.

**Builds on:** step 6, step 4.

**Mockups:** [1.17-1.png](../mockups/1.17-1.png), [1.17-2.png](../mockups/1.17-2.png)

**Show on video:**

- [ ] `StateStyle` and `Appearance` on every widget with fallback to normal and built-in defaults
- [ ] Hover, press/release, focus, disabled, checked, and drag-over switch the effective style automatically
- [ ] Default look unchanged (golden images identical)
- [ ] Repaint only when the effective style changes
- [ ] Stored in form files

### Step 20: Timers and idle callbacks

**Issue:** [#21](https://github.com/KenSuenobu/rust-pushrod/issues/21) `pushrod: [1.5] Timers and idle callbacks`

**Delivers:** Repeating/one-shot timers, per-widget ticks for animation, and idle callbacks.

**Builds on:** step 7.

**Show on video:**

- [ ] One-shot and repeating timers fire within one frame of their interval
- [ ] Timers can be cancelled from callbacks
- [ ] `Tick` only flows while a widget requests animation
- [ ] Idle callbacks run only when no events are pending
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 21: Backend crates: keep SDL out of the core and the widgets

**Issue:** [#134](https://github.com/KenSuenobu/rust-pushrod/issues/134) `pushrod: [10.6] Backend crates: keep SDL out of the core and the widgets`

**Delivers:** Move everything SDL-specific into a `pushrod-sdl2` backend crate, so `pushrod` and `pushrod-widgets` have no SDL dependency and a future SDL3 backend is a drop-in swap.

**Builds on:** step 5, step 4, step 1.

**Show on video:**

- [ ] `cargo tree -p pushrod` and `cargo tree -p pushrod-widgets` show no `sdl2`
- [ ] No SDL type in any public signature of the core or the widgets
- [ ] `pushrod-sdl2` reproduces today's rendering (golden images unchanged)
- [ ] Tests run on the headless backend without a display

### Step 22: Errors: return Result instead of panicking

**Issue:** [#138](https://github.com/KenSuenobu/rust-pushrod/issues/138) `pushrod: [1.25] Errors: return Result instead of panicking`

**Delivers:** Library code reports failures (fonts, images, windows, unknown widget IDs) as `Result` values instead of panicking.

**Builds on:** step 8.

**Show on video:**

- [ ] No `unwrap`/`expect` in library code (lints enforced)
- [ ] Missing font or image returns an error, not a panic
- [ ] Error event delivered for failures during the run loop

### Step 23: Fonts: shared font handles and a built-in default font

**Issue:** [#137](https://github.com/KenSuenobu/rust-pushrod/issues/137) `pushrod: [1.24] Fonts: shared font handles and a built-in default font`

**Delivers:** Replace per-widget font file paths with shared `Font` handles, and ship a default font so applications work without any font files.

**Builds on:** step 14.

**Show on video:**

- [ ] Quick start runs from any directory with no font files
- [ ] Widgets accept a `Font` handle; no file paths in widget constructors
- [ ] Embedded font's license included and noted in the docs

### Step 24: Public API polish before publishing: builders, prelude, and stability

**Issue:** [#136](https://github.com/KenSuenobu/rust-pushrod/issues/136) `pushrod: [1.23] Public API polish before publishing: builders, prelude, and stability`

**Delivers:** Make the public API pleasant and stable before crates.io: builders with defaults, preludes, non-exhaustive enums, and consistent names.

**Builds on:** step 4, step 1, step 22, step 23.

**Show on video:**

- [ ] No public constructor needs more than three required arguments
- [ ] Public enums are `#[non_exhaustive]`
- [ ] Quick start uses the preludes
- [ ] Naming decision recorded in the roadmap's Naming section

### Step 25: Custom paint widget (closure-drawn)

**Issue:** [#36](https://github.com/KenSuenobu/rust-pushrod/issues/36) `pushrod: [2.10] Custom paint widget (closure-drawn)`

**Delivers:** One-off custom drawing without writing a widget struct.

**Builds on:** nothing earlier; can start immediately.

**Mockups:** [2.10-1.png](../mockups/2.10-1.png)

**Show on video:**

- [ ] Closure paints into the texture
- [ ] Optional event closure receives events
- [ ] Example draws a chart

### Step 26: Boxed label widget

**Issue:** [#27](https://github.com/KenSuenobu/rust-pushrod/issues/27) `pushrod: [2.1] Boxed label widget`

**Delivers:** Text with background and border.

**Builds on:** nothing earlier; can start immediately.

**Mockups:** [2.1-1.png](../mockups/2.1-1.png)

**Show on video:**

- [ ] Renders background, centered text, border
- [ ] Display only (empty event mask)

### Step 27: Separator line

**Issue:** [#83](https://github.com/KenSuenobu/rust-pushrod/issues/83) `pushrod: [2.14] Separator line`

**Delivers:** Horizontal or vertical divider line.

**Builds on:** nothing earlier; can start immediately.

**Mockups:** [2.14-1.png](../mockups/2.14-1.png)

**Show on video:**

- [ ] Horizontal and vertical render correctly

### Step 28: Group box (titled frame)

**Issue:** [#41](https://github.com/KenSuenobu/rust-pushrod/issues/41) `pushrod: [3.3] Group box (titled frame)`

**Delivers:** A frame with a title in its top border, grouping related controls.

**Builds on:** nothing earlier; can start immediately.

**Mockups:** [3.3-1.png](../mockups/3.3-1.png)

**Show on video:**

- [ ] Title interrupts the border
- [ ] Works as the parent of a radio group

### Step 29: Image widget

**Issue:** [#33](https://github.com/KenSuenobu/rust-pushrod/issues/33) `pushrod: [2.7] Image widget`

**Delivers:** Display a PNG/BMP image centered, stretched, scaled to fit, or tiled.

**Builds on:** nothing earlier; can start immediately.

**Mockups:** [2.7-1.png](../mockups/2.7-1.png)

**Show on video:**

- [ ] Loads PNG and BMP
- [ ] Fit modes render as documented
- [ ] Missing files don't panic

### Step 30: Glyph button (small single-character button)

**Issue:** [#28](https://github.com/KenSuenobu/rust-pushrod/issues/28) `pushrod: [2.2] Glyph button (small single-character button)`

**Delivers:** A small box with an arrow or symbol, used for scroll arrows, close boxes, and spin buttons.

**Builds on:** step 6, step 20.

**Mockups:** [2.2-1.png](../mockups/2.2-1.png)

**Show on video:**

- [ ] Centered glyph
- [ ] Pressed state inverts
- [ ] Activate-on-press with auto-repeat

### Step 31: Button behaviors: default, cancel, toggle, activate-on-press, dismiss dialog

**Issue:** [#29](https://github.com/KenSuenobu/rust-pushrod/issues/29) `pushrod: [2.3] Button behaviors: default, cancel, toggle, activate-on-press, dismiss dialog`

**Delivers:** A push button that runs an action when clicked; it can also be a dialog's default or cancel button, a toggle, or a button that closes a dialog.

**Builds on:** step 6, step 11.

**Mockups:** [2.3-1.png](../mockups/2.3-1.png), [2.3-2.png](../mockups/2.3-2.png), [2.3-3.png](../mockups/2.3-3.png)

**Show on video:**

- [ ] Each flag combination behaves as described and is tested
- [ ] Default button is recognizable at a glance
- [ ] No regression in PR #10 click tests

### Step 32: Check box with indeterminate state

**Issue:** [#30](https://github.com/KenSuenobu/rust-pushrod/issues/30) `pushrod: [2.4] Check box with indeterminate state`

**Delivers:** Box with a check mark and label; toggles checked; optional mixed state.

**Builds on:** step 6, step 11.

**Mockups:** [2.4-1.png](../mockups/2.4-1.png)

**Show on video:**

- [ ] Click, Space, and mnemonic toggle
- [ ] Indeterminate renders and resolves on click
- [ ] Label click toggles

### Step 33: Radio buttons and radio groups

**Issue:** [#31](https://github.com/KenSuenobu/rust-pushrod/issues/31) `pushrod: [2.5] Radio buttons and radio groups`

**Delivers:** One-of-many selection among sibling widgets.

**Builds on:** step 6, step 11.

**Mockups:** [2.5-1.png](../mockups/2.5-1.png)

**Show on video:**

- [ ] Exactly one exclusive sibling is checked after any change
- [ ] Works for any widget with `EXCLUSIVE`
- [ ] Arrow keys move within the group

### Step 34: Progress bar

**Issue:** [#48](https://github.com/KenSuenobu/rust-pushrod/issues/48) `pushrod: [4.1] Progress bar`

**Delivers:** A bar showing how far along a task is; with no value it shows a busy animation (like HTML's progress element).

**Builds on:** step 20.

**Mockups:** [4.1-1.png](../mockups/4.1-1.png)

**Show on video:**

- [ ] Determinate fill proportional to value / max
- [ ] `None` value shows the indeterminate animation without callbacks
- [ ] Static in outline drawing mode

### Step 35: Slider

**Issue:** [#40](https://github.com/KenSuenobu/rust-pushrod/issues/40) `pushrod: [3.2] Slider`

**Delivers:** Horizontal/vertical slider with range, optional ticks, and value label.

**Builds on:** step 11.

**Mockups:** [3.2-1.png](../mockups/3.2-1.png)

**Show on video:**

- [ ] Drag/click set value within range
- [ ] Ticks and snapping
- [ ] Keyboard adjusts

### Step 36: Text field: single-line entry with input masks, validation, and numeric mode

**Issue:** [#32](https://github.com/KenSuenobu/rust-pushrod/issues/32) `pushrod: [2.6] Text field: single-line entry with input masks, validation, and numeric mode`

**Delivers:** Single-line text entry with caret, selection, clipboard, input mask, validator, and numeric mode.

**Builds on:** step 18, step 20, step 11.

**Mockups:** [2.6-1.png](../mockups/2.6-1.png), [2.6-2.png](../mockups/2.6-2.png), [2.6-3.png](../mockups/2.6-3.png)

**Show on video:**

- [ ] Typing, deleting, caret movement, selection, copy/paste
- [ ] Masks and validators reject invalid input
- [ ] Numeric mode with range
- [ ] `EditingFinished` on Return and focus loss
- [ ] Leading/trailing accessories inside the edit area; text never overlaps them
- [ ] `Clear`, `Search`, `RevealPassword`, and custom accessories; `TextField::search()` preset

### Step 37: Clip children to their parent's bounds

**Issue:** [#56](https://github.com/KenSuenobu/rust-pushrod/issues/56) `pushrod: [5.1] Clip children to their parent's bounds`

**Delivers:** Children never draw or receive input outside their parent.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [ ] Partly outside children are cropped
- [ ] Hit testing ignores cropped parts
- [ ] Golden test

### Step 38: Composite widgets: build widgets from parts that applications can retrieve and modify

**Issue:** [#57](https://github.com/KenSuenobu/rust-pushrod/issues/57) `pushrod: [5.2] Composite widgets: build widgets from parts that applications can retrieve and modify`

**Delivers:** A Composite trait whose parts (labels, buttons, icons, rows) are real widgets that applications can fetch, restyle, add, and remove.

**Builds on:** step 16, step 11.

**Mockups:** [5.2-1.png](../mockups/5.2-1.png)

**Show on video:**

- [ ] Composites create parts as ordinary widgets with IDs
- [ ] `parts()`, `part(name)`, and typed part accessors exist on every built-in composite
- [ ] Changing a part's font, color, or text through `ui.get_mut` repaints it
- [ ] List-structured composites support add/insert/remove of parts with automatic re-layout
- [ ] Composite translates part events; callbacks on part IDs also work
- [ ] Form round trip preserves part overrides and added/removed parts
- [ ] Scroll bar and alert built on it

### Step 39: Layout manager foundation: Layout trait, size hints, measure and arrange

**Issue:** [#116](https://github.com/KenSuenobu/rust-pushrod/issues/116) `pushrod: [5.5] Layout manager foundation: Layout trait, size hints, measure and arrange`

**Delivers:** A small Layout trait any container can use; widgets report minimum, preferred, and maximum sizes; containers measure then arrange their children.

**Builds on:** step 8, step 13.

**Show on video:**

- [ ] `SizeHint`, `LayoutParams`, and the `Layout` trait
- [ ] Containers with a layout are measured and arranged once per frame when dirty
- [ ] Layout invalidation stops at containers whose size doesn't change
- [ ] Moving without resizing doesn't repaint
- [ ] Absolute positioning unchanged when no layout is set
- [ ] An application-defined layout works

### Step 40: Row and column layouts

**Issue:** [#117](https://github.com/KenSuenobu/rust-pushrod/issues/117) `pushrod: [5.6] Row and column layouts`

**Delivers:** Arrange children in a horizontal row or vertical column with spacing, margins, alignment, and stretch.

**Builds on:** step 39.

**Mockups:** [5.6-1.png](../mockups/5.6-1.png)

**Show on video:**

- [ ] Row and column with spacing, margins, alignment, stretch
- [ ] Spacer for right/bottom alignment
- [ ] Min/max respected while resizing

### Step 41: Form layout: aligned label and field rows

**Issue:** [#118](https://github.com/KenSuenobu/rust-pushrod/issues/118) `pushrod: [5.7] Form layout: aligned label and field rows`

**Delivers:** Two-column 'Label: field' rows with labels aligned and fields stretching to the available width.

**Builds on:** step 39.

**Mockups:** [5.7-1.png](../mockups/5.7-1.png)

**Show on video:**

- [ ] Labels align to the widest label
- [ ] Fields stretch with the window
- [ ] Narrow mode puts labels above fields
- [ ] Label click focuses its field

### Step 42: Scroll bar

**Issue:** [#39](https://github.com/KenSuenobu/rust-pushrod/issues/39) `pushrod: [3.1] Scroll bar`

**Delivers:** Vertical/horizontal scroll bar with arrows and a proportional thumb.

**Builds on:** step 20, step 38, step 30, step 11.

**Mockups:** [3.1-1.png](../mockups/3.1-1.png), [3.1-2.png](../mockups/3.1-2.png)

**Show on video:**

- [ ] Drag, arrows with repeat, track paging
- [ ] Thumb size reflects proportion
- [ ] Parts are retrievable and modifiable as described

### Step 43: Container: a scrollable viewport onto fixed-size content

**Issue:** [#58](https://github.com/KenSuenobu/rust-pushrod/issues/58) `pushrod: [5.3] Container: a scrollable viewport onto fixed-size content`

**Delivers:** A widget whose children live on a content area with its own size and 0,0 origin; the container's frame is a viewport panned by scroll bars.

**Builds on:** step 38, step 37, step 42.

**Mockups:** [5.3-1.png](../mockups/5.3-1.png), [5.3-2.png](../mockups/5.3-2.png)

**Show on video:**

- [ ] Children positioned relative to the content's 0,0; content size independent of the container size
- [ ] Resizing the container changes only the visible area; moving it doesn't shift content
- [ ] Scroll bars appear when needed (or always/never); wheel and keyboard pan; offset clamped to content bounds
- [ ] `scroll_to`, `scroll_by`, `ensure_visible`, `viewport()`; focused widgets scrolled into view
- [ ] Children clipped to and hit-tested through the viewport; panning doesn't repaint unchanged children
- [ ] Content size modes Fixed, FitChildren, FollowWidth; works with a layout manager and with a loaded form
- [ ] `ViewportChanged` emitted on pan and resize
- [ ] Content larger than the GPU's maximum texture size works
- [ ] Only visible children are drawn

### Step 44: Golden-image rendering tests

**Issue:** [#70](https://github.com/KenSuenobu/rust-pushrod/issues/70) `pushrod: [7.2] Golden-image rendering tests`

**Delivers:** Render each widget and state and compare with reference PNGs.

**Builds on:** step 2.

**Show on video:**

- [ ] Goldens for every widget and state
- [ ] Runs in CI
- [ ] Diffs uploaded on failure

### Step 45: Performance benchmarks and budgets

**Issue:** [#115](https://github.com/KenSuenobu/rust-pushrod/issues/115) `pushrod: [7.7] Performance benchmarks and budgets`

**Delivers:** Measure frame time, input latency, memory per widget, and startup; fail CI on regressions.

**Builds on:** step 2.

**Show on video:**

- [ ] Benchmarks run in CI with a stored baseline
- [ ] Regressions beyond tolerance fail or warn
- [ ] README shows current numbers

### Step 46: High-DPI rendering and window resize

**Issue:** [#25](https://github.com/KenSuenobu/rust-pushrod/issues/25) `pushrod: [1.10] High-DPI rendering and window resize`

**Delivers:** The root widget follows the window size; crisp rendering on high-DPI displays.

**Builds on:** step 5, step 39.

**Show on video:**

- [ ] Sharp text and lines at 1x, 1.5x, and 2x
- [ ] Moving a window between displays re-renders at the new scale
- [ ] Resizing re-runs layouts and respects minimum size

### Step 47: Easy installation: bundled SDL, static linking, and platform guides

**Issue:** [#139](https://github.com/KenSuenobu/rust-pushrod/issues/139) `pushrod: [7.9] Easy installation: bundled SDL, static linking, and platform guides`

**Delivers:** Make installing Pushrod painless on Linux, macOS, and Windows, including building SDL from source.

**Builds on:** step 2.

**Show on video:**

- [ ] CI builds with `bundled` on macOS and Windows
- [ ] Installation pages for Linux, macOS, and Windows tested from a clean machine
- [ ] With 10.6 complete, `bundled` needs no system libraries

### Step 48: Edit the widget tree at run time: remove, reparent, reorder

**Issue:** [#23](https://github.com/KenSuenobu/rust-pushrod/issues/23) `pushrod: [1.8] Edit the widget tree at run time: remove, reparent, reorder`

**Delivers:** Remove widgets and change parent or stacking order without breaking IDs.

**Builds on:** step 4.

**Show on video:**

- [ ] Remaining IDs never change
- [ ] Removed textures are destroyed
- [ ] Focus/hover/capture cleared safely
- [ ] Tests for remove, reparent, reorder
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 49: Modal dialogs: run a form until a dismissing control is chosen

**Issue:** [#66](https://github.com/KenSuenobu/rust-pushrod/issues/66) `pushrod: [6.7] Modal dialogs: run a form until a dismissing control is chosen`

**Delivers:** Open a form modally; the result is the ID of the control that dismissed it.

**Builds on:** step 18, step 48, step 31.

**Show on video:**

- [ ] Input blocked outside the top modal
- [ ] Dismissing control reported
- [ ] Nested modals
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 50: Alert dialog

**Issue:** [#35](https://github.com/KenSuenobu/rust-pushrod/issues/35) `pushrod: [2.9] Alert dialog`

**Delivers:** One-call modal alert with icon, message, and up to three buttons.

**Builds on:** step 38, step 49, step 31.

**Mockups:** [2.9-1.png](../mockups/2.9-1.png), [2.9-3.png](../mockups/2.9-3.png)

**Show on video:**

- [ ] Modal: nothing else receives input while open
- [ ] Return/Escape behave as described
- [ ] Completion closure gets the index
- [ ] Parts are retrievable and modifiable as described

### Step 51: Reference application: a real app built only with Pushrod

**Issue:** [#140](https://github.com/KenSuenobu/rust-pushrod/issues/140) `pushrod: [7.10] Reference application: a real app built only with Pushrod`

**Delivers:** A small but complete application, maintained in the repository, that exercises the MVP together and surfaces API problems early.

**Builds on:** step 36, step 43, step 50, step 40, step 46.

**Show on video:**

- [ ] Runs on Linux, macOS, and Windows
- [ ] Fully usable with the keyboard alone
- [ ] v0.2 version loads its window from a form file
- [ ] Golden images in CI

### Step 52: Guide: writing widgets and composites

**Issue:** [#72](https://github.com/KenSuenobu/rust-pushrod/issues/72) `pushrod: [7.4] Guide: writing widgets and composites`

**Delivers:** Step-by-step docs for extending Pushrod.

**Builds on:** step 38, step 17.

**Show on video:**

- [ ] Compiling examples for each section
- [ ] Linked from README and crate docs

### Step 53: Changelog, Cargo metadata, and crates.io release

**Issue:** [#73](https://github.com/KenSuenobu/rust-pushrod/issues/73) `pushrod: [7.5] Changelog, Cargo metadata, and crates.io release`

**Delivers:** Publish the modernized library (answers #1 and #8).

**Builds on:** every earlier step in this part (it is the release).

**Show on video:**

- [ ] Metadata correct
- [ ] Changelog complete
- [ ] docs.rs builds

### Step 54: Checkpoint: Pushrod v0.1 released

**Video:** Publish `pushrod`, `pushrod-sdl2`, and `pushrod-widgets` v0.1 to crates.io; walk through the reference application and the documentation site.

## Part 2: Pushrod v0.2, forms and resources

Forms, form files, and resources by ID (the runtime foundation for Chassis), plus input methods, mnemonics, accessibility, and specs. Completes the Pushrod MVP.

### Step 55: Module boundaries and written specs with conformance tests

**Issue:** [#114](https://github.com/KenSuenobu/rust-pushrod/issues/114) `pushrod: [7.6] Module boundaries and written specs with conformance tests`

**Delivers:** Specify each module's contract (widget, events, paint, form format) in docs/specs and test against it.

**Builds on:** step 4, step 2.

**Show on video:**

- [ ] Specs for widget, events, paint, and forms
- [ ] Engine and design layer separated in module layout
- [ ] Conformance tests for each stated guarantee run in CI

### Step 56: Input method (IME) support: composition text and candidate window

**Issue:** [#112](https://github.com/KenSuenobu/rust-pushrod/issues/112) `pushrod: [1.19] Input method (IME) support: composition text and candidate window`

**Delivers:** Show in-progress composition text in fields and position the platform candidate window at the caret.

**Builds on:** step 18, step 13.

**Mockups:** [1.19-1.png](../mockups/1.19-1.png)

**Show on video:**

- [ ] Japanese and Chinese input methods compose and commit correctly in a text field
- [ ] Candidate window appears at the caret and follows it
- [ ] Composition never fires `TextChanged` until committed

### Step 57: Keyboard mnemonics: underlined shortcut letters

**Issue:** [#76](https://github.com/KenSuenobu/rust-pushrod/issues/76) `pushrod: [1.12] Keyboard mnemonics: underlined shortcut letters`

**Delivers:** Alt+letter (or the letter alone in dialogs) activates a control, shown by an underlined character.

**Builds on:** step 18.

**Mockups:** [1.12-1.png](../mockups/1.12-1.png)

**Show on video:**

- [ ] Markers are underlined, not drawn
- [ ] Alt+letter activates buttons and toggles check boxes; labels focus the next field
- [ ] Scoped to the topmost modal dialog
- [ ] Duplicate detection in debug builds

### Step 58: Widget registry and factory

**Issue:** [#60](https://github.com/KenSuenobu/rust-pushrod/issues/60) `pushrod: [6.1] Widget registry and factory`

**Delivers:** Create widgets by type name, the prerequisite for loading forms.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [ ] All built-ins registered
- [ ] Custom widgets registrable
- [ ] Unknown names are errors, not panics

### Step 59: Property introspection: list, get, and set widget properties by name

**Issue:** [#61](https://github.com/KenSuenobu/rust-pushrod/issues/61) `pushrod: [6.2] Property introspection: list, get, and set widget properties by name`

**Delivers:** Each widget describes its properties for saving, loading, bindings, and the Chassis inspector.

**Builds on:** step 6, step 58.

**Show on video:**

- [ ] Every built-in lists and sets all properties
- [ ] Round-trip test
- [ ] `PropertyChanged` emitted on change

### Step 60: Accessibility: expose widgets to screen readers

**Issue:** [#80](https://github.com/KenSuenobu/rust-pushrod/issues/80) `pushrod: [1.14] Accessibility: expose widgets to screen readers`

**Delivers:** Publish roles, names, states, and actions of widgets to the platform accessibility APIs.

**Builds on:** step 6, step 59, step 18.

**Show on video:**

- [ ] A screen reader announces focused buttons, check boxes, and text fields with name and state
- [ ] Accessibility actions trigger the same callbacks as user input
- [ ] Updates are incremental (changed nodes only) and the tree is only built when assistive technology is active

### Step 61: Form file format (binary, versioned)

**Issue:** [#62](https://github.com/KenSuenobu/rust-pushrod/issues/62) `pushrod: [6.3] Form file format (binary, versioned)`

**Delivers:** The binary file that stores forms: object table by ID, string and image pools, symbols.

**Builds on:** step 59, step 58.

**Show on video:**

- [ ] Spec merged
- [ ] Reader/writer with tests on hand-built bytes
- [ ] Unknown property tags skipped
- [ ] Versioned

### Step 62: Form loader: build a form from a file with stable IDs

**Issue:** [#63](https://github.com/KenSuenobu/rust-pushrod/issues/63) `pushrod: [6.4] Form loader: build a form from a file with stable IDs`

**Delivers:** Load a form and attach callbacks by ID or name.

**Builds on:** step 61.

**Show on video:**

- [ ] Loaded form renders identically to the code-built version (golden)
- [ ] IDs resolve via generated constants
- [ ] Errors name object and property
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 63: Form writer: save a widget subtree to a file

**Issue:** [#64](https://github.com/KenSuenobu/rust-pushrod/issues/64) `pushrod: [6.5] Form writer: save a widget subtree to a file`

**Delivers:** Serialize any subtree into the form format.

**Builds on:** step 61.

**Show on video:**

- [ ] Round trip preserves properties and pixels
- [ ] Composite parts not written

### Step 64: Generate ID constants from a form file

**Issue:** [#65](https://github.com/KenSuenobu/rust-pushrod/issues/65) `pushrod: [6.6] Generate ID constants from a form file`

**Delivers:** build.rs helper and CLI that emit Rust constants for named objects.

**Builds on:** step 62, step 61.

**Show on video:**

- [ ] build.rs integration used by the form example
- [ ] Deterministic, rustfmt-clean output

### Step 65: Resource kinds: strings, images, icons, files, menus, and alerts

**Issue:** [#154](https://github.com/KenSuenobu/rust-pushrod/issues/154) `pushrod: [6.11] Resource kinds: strings, images, icons, files, menus, and alerts`

**Delivers:** Extend the resource file format beyond forms: string tables, images, icons, arbitrary files, menus, and alerts, each with a stable ID.

**Builds on:** step 61.

**Show on video:**

- [ ] Round trip (write, read) for every kind
- [ ] IDs stay stable when resources are added, removed, or reordered
- [ ] Older readers skip unknown kinds
- [ ] `docs/FORM_FORMAT.md` documents every table

### Step 66: Typed resource IDs: generated constants per resource kind

**Issue:** [#155](https://github.com/KenSuenobu/rust-pushrod/issues/155) `pushrod: [6.12] Typed resource IDs: generated constants per resource kind`

**Delivers:** Generate typed ID constants for every resource kind and for the widgets inside each form, so code refers to resources by name.

**Builds on:** step 65, step 64.

**Show on video:**

- [ ] Constants generated for every kind and every named widget
- [ ] Passing an ID of the wrong kind is a compile error
- [ ] Generated output identical across runs

### Step 67: Resource manager: load resource sets and get resources by ID

**Issue:** [#153](https://github.com/KenSuenobu/rust-pushrod/issues/153) `pushrod: [6.10] Resource manager: load resource sets and get resources by ID`

**Delivers:** Load a resource file (from disk or embedded in the executable) and get its forms, strings, images, icons, files, menus, and alerts by ID.

**Builds on:** step 22, step 65, step 62, step 61.

**Show on video:**

- [ ] Load from a file and from embedded bytes; same results
- [ ] Every kind retrievable by typed ID; wrong-kind IDs fail to compile
- [ ] Stacked sets resolve in order; an application set overrides a library set
- [ ] Images upload once and are shared by every widget that uses them
- [ ] Malformed files return an error, never panic

### Step 68: Checkpoint: Pushrod v0.2 released: the Pushrod MVP is complete

**Video:** Publish v0.2; show a form designed in a resource file loading by ID with images, icons, and strings.

## Part 3: Chassis M1: core without UI

Project model, resource access, code generation, and a command-line generator, all testable without a window. Can start as soon as Pushrod's resource manager (6.10 to 6.12) lands. Pushrod work this phase needs comes first.

### Step 69: Event metadata in the widget registry

**Issue:** [#156](https://github.com/KenSuenobu/rust-pushrod/issues/156) `pushrod: [6.13] Event metadata in the widget registry`

**Delivers:** Each registered widget type declares the events it emits, its default event, and each event's payload, so tools like Chassis can list them.

**Builds on:** step 4, step 58.

**Show on video:**

- [ ] Every standard widget declares its events and default event
- [ ] Declared events match emitted events (test)
- [ ] Third-party widget libraries can declare theirs

### Step 70: Project model

**Issue:** [#164](https://github.com/KenSuenobu/rust-pushrod/issues/164) `chassis: [2.1] Project model`

**Delivers:** A project is a Cargo crate, its resource file, its handler files, and its generated files.

**Builds on:** step 67, step 65, step 66.

**Show on video:**

- [ ] Loads a project directory and reports its forms, resources, and handlers
- [ ] Ownership rules documented and enforced (Chassis writes only generated files and stubs)
- [ ] Unit tests cover the model without a window

### Step 71: Code generation rules

**Issue:** [#165](https://github.com/KenSuenobu/rust-pushrod/issues/165) `chassis: [6.1] Code generation rules`

**Delivers:** What Chassis writes, when, and what it never touches.

**Builds on:** step 70.

**Show on video:**

- [ ] Tests prove handler bodies survive every designer operation
- [ ] Rules documented

### Step 72: Event wiring generation

**Issue:** [#166](https://github.com/KenSuenobu/rust-pushrod/issues/166) `chassis: [5.7] Event wiring generation`

**Delivers:** Generated code registers each handler with the engine by widget ID.

**Builds on:** step 70, step 66, step 69.

**Show on video:**

- [ ] A project with handlers builds and every handler fires
- [ ] Deterministic output (same input, same output)

### Step 73: Command-line generator

**Issue:** [#167](https://github.com/KenSuenobu/rust-pushrod/issues/167) `chassis: [6.5] Command-line generator`

**Delivers:** `chassis generate` regenerates IDs and wiring without opening the UI.

**Builds on:** step 70, step 72, step 71.

**Show on video:**

- [ ] Output identical to what the UI generates
- [ ] `chassis check` exits non-zero on problems

### Step 74: Checkpoint: Chassis M1: core without UI complete

**Video:** Run `chassis generate` and `chassis check` on a project; show the generated typed IDs and wiring, and the tests that prove handler bodies are never touched.

## Part 4: Chassis M2: shell and project

The main window, three panes, Designer/Code tabs, commands, and the project drawer with resource editors. Pushrod work this phase needs comes first.

### Step 75: Actions and keyboard shortcuts: shared commands for menus, toolbars, and buttons

**Issue:** [#97](https://github.com/KenSuenobu/rust-pushrod/issues/97) `pushrod: [9.5] Actions and keyboard shortcuts: shared commands for menus, toolbars, and buttons`

**Delivers:** One action object drives every menu item, toolbar button, and shortcut that performs the same command, keeping enabled/checked state in sync.

**Builds on:** step 4, step 11.

**Show on video:**

- [ ] A Bold action toggled from the toolbar checks the Bold menu item
- [ ] Disabling an action disables every control
- [ ] Shortcuts trigger actions; platform-standard keys map correctly
- [ ] Exclusive action groups
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 76: Drag and drop within the application

**Issue:** [#100](https://github.com/KenSuenobu/rust-pushrod/issues/100) `pushrod: [9.8] Drag and drop within the application`

**Delivers:** Drag data between widgets (list to list, file icon to drop zone) with drop targets and feedback.

**Builds on:** step 17, step 37.

**Mockups:** [9.8-1.png](../mockups/9.8-1.png)

**Show on video:**

- [ ] Drag between two lists
- [ ] Drop highlighting and cancel
- [ ] OS file drops delivered
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 77: Undo and redo command stack

**Issue:** [#81](https://github.com/KenSuenobu/rust-pushrod/issues/81) `pushrod: [1.15] Undo and redo command stack`

**Delivers:** An application-level undo stack that text fields and actions can use.

**Builds on:** step 75.

**Show on video:**

- [ ] Undo/redo of text edits and application commands
- [ ] Consecutive typing merges into one undo step
- [ ] Undo/Redo actions enable and disable automatically

### Step 78: Autoresizing (keep margins or stretch on resize)

**Issue:** [#59](https://github.com/KenSuenobu/rust-pushrod/issues/59) `pushrod: [5.4] Autoresizing (keep margins or stretch on resize)`

**Delivers:** Children keep margins or stretch when their parent is resized.

**Builds on:** nothing earlier; can start immediately.

**Show on video:**

- [ ] Children follow their anchors on parent resize
- [ ] Stored in form files

### Step 79: List view

**Issue:** [#52](https://github.com/KenSuenobu/rust-pushrod/issues/52) `pushrod: [4.5] List view`

**Delivers:** Scrolling list with single/multiple selection, backed by a data source.

**Builds on:** step 38, step 43, step 11.

**Mockups:** [4.5-1.png](../mockups/4.5-1.png)

**Show on video:**

- [ ] Smooth with 10,000 rows
- [ ] Selection modes and keyboard navigation
- [ ] Model change notifications update the view
- [ ] Parts are retrievable and modifiable as described
- [ ] Range-based model notifications; row widgets recycled while scrolling

### Step 80: Tab view

**Issue:** [#51](https://github.com/KenSuenobu/rust-pushrod/issues/51) `pushrod: [4.4] Tab view`

**Delivers:** Tabs that switch between pages of controls.

**Builds on:** step 6, step 38.

**Mockups:** [4.4-1.png](../mockups/4.4-1.png)

**Show on video:**

- [ ] Switching hides/shows pages
- [ ] Hidden controls keep state and IDs
- [ ] Ctrl+Tab
- [ ] Parts are retrievable and modifiable as described

### Step 81: Table view

**Issue:** [#92](https://github.com/KenSuenobu/rust-pushrod/issues/92) `pushrod: [4.9] Table view`

**Delivers:** Multi-column list with headers, column resizing, and sorting.

**Builds on:** step 38, step 79.

**Mockups:** [4.9-1.png](../mockups/4.9-1.png)

**Show on video:**

- [ ] Columns resize and sort
- [ ] Only visible cells painted
- [ ] Parts are retrievable and modifiable as described

### Step 82: Expander (collapsible section)

**Issue:** [#90](https://github.com/KenSuenobu/rust-pushrod/issues/90) `pushrod: [3.10] Expander (collapsible section)`

**Delivers:** A titled section that shows or hides its contents when clicked.

**Builds on:** step 6, step 78.

**Mockups:** [3.10-1.png](../mockups/3.10-1.png)

**Show on video:**

- [ ] Click, Space, and mnemonic toggle
- [ ] Children hidden and shown

### Step 83: Menu bar and drop-down menus (drawn by Pushrod)

**Issue:** [#37](https://github.com/KenSuenobu/rust-pushrod/issues/37) `pushrod: [2.11] Menu bar and drop-down menus (drawn by Pushrod)`

**Delivers:** Menu bar with drop-down menus, separators, check marks, submenus, shortcuts, and mnemonics.

**Builds on:** step 37, step 6, step 57, step 75, step 38.

**Mockups:** [2.11-1.png](../mockups/2.11-1.png)

**Show on video:**

- [ ] Menus open, track, select, and close correctly
- [ ] Submenus and keyboard navigation
- [ ] No OS-native menu APIs on any platform
- [ ] Parts are retrievable and modifiable as described
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 84: Image button

**Issue:** [#125](https://github.com/KenSuenobu/rust-pushrod/issues/125) `pushrod: [2.21] Image button`

**Delivers:** A button that shows an image, or an image with a text label, with the full button behavior.

**Builds on:** step 38, step 31, step 29.

**Mockups:** [2.21-1.png](../mockups/2.21-1.png)

**Show on video:**

- [ ] Image only, and image with label in each position
- [ ] Per-state images or automatic pressed/disabled looks
- [ ] All button behaviors and mnemonics work

### Step 85: Toolbar

**Issue:** [#84](https://github.com/KenSuenobu/rust-pushrod/issues/84) `pushrod: [2.15] Toolbar`

**Delivers:** A row of icon/text buttons bound to actions.

**Builds on:** step 38, step 84, step 75.

**Mockups:** [2.15-1.png](../mockups/2.15-1.png)

**Show on video:**

- [ ] Buttons reflect action state
- [ ] Overflow menu when too narrow
- [ ] Parts are retrievable and modifiable as described

### Step 86: Status bar

**Issue:** [#85](https://github.com/KenSuenobu/rust-pushrod/issues/85) `pushrod: [2.16] Status bar`

**Delivers:** A strip at the bottom of the window for messages and status fields.

**Builds on:** step 20, step 38.

**Mockups:** [2.16-1.png](../mockups/2.16-1.png)

**Show on video:**

- [ ] Timed messages clear themselves
- [ ] Hovered action status text appears
- [ ] Parts are retrievable and modifiable as described

### Step 87: Icon widget (picture with caption)

**Issue:** [#34](https://github.com/KenSuenobu/rust-pushrod/issues/34) `pushrod: [2.8] Icon widget (picture with caption)`

**Delivers:** Picture with a caption, selectable and double-clickable, like desktop icons.

**Builds on:** step 29, step 11.

**Mockups:** [2.8-1.png](../mockups/2.8-1.png)

**Show on video:**

- [ ] Selection inverts the caption
- [ ] Double-click and Return emit `Activated`

### Step 88: File chooser dialog (drawn by Pushrod)

**Issue:** [#38](https://github.com/KenSuenobu/rust-pushrod/issues/38) `pushrod: [2.12] File chooser dialog (drawn by Pushrod)`

**Delivers:** Choose a file or directory with a consistent look on every platform.

**Builds on:** step 49, step 38, step 36, step 79.

**Mockups:** [2.12-1.png](../mockups/2.12-1.png)

**Show on video:**

- [ ] Navigate, filter, choose; keyboard operable
- [ ] Returns None on cancel
- [ ] Parts are retrievable and modifiable as described

### Step 89: Split view (panes with draggable dividers)

**Issue:** [#47](https://github.com/KenSuenobu/rust-pushrod/issues/47) `pushrod: [3.9] Split view (panes with draggable dividers)`

**Delivers:** Two or more panes side by side or stacked, separated by dividers the user drags to resize them.

**Builds on:** step 38, step 17, step 39.

**Mockups:** [3.9-1.png](../mockups/3.9-1.png)

**Show on video:**

- [ ] Dragging a divider resizes neighboring panes within their min/max
- [ ] Collapsible panes; double-click to collapse/restore
- [ ] Extra space distributed by stretch factor
- [ ] Positions settable and readable; outline mode supported

### Step 90: Chassis crate and main window

**Issue:** [#168](https://github.com/KenSuenobu/rust-pushrod/issues/168) `chassis: [1.1] Chassis crate and main window`

**Delivers:** Binary crate and the main window with menu bar, toolbar, and status bar.

**Builds on:** step 83, step 85, step 86, step 75, step 21.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Window frame matches the mockup (menu bar, toolbar, status bar)
- [ ] Menus open and toolbar buttons trigger their commands (stubs where later issues implement them)
- [ ] Status bar updates with the selection

### Step 91: Three-pane workspace

**Issue:** [#169](https://github.com/KenSuenobu/rust-pushrod/issues/169) `chassis: [1.2] Three-pane workspace`

**Delivers:** Project drawer, editor, and inspector in resizable, collapsible split views.

**Builds on:** step 90, step 89.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Dividers drag and collapse; minimum widths respected
- [ ] F4 toggles the inspector
- [ ] Layout matches the mockup at 1200 x 760 and scales up

### Step 92: Commands, menus, and shortcuts

**Issue:** [#170](https://github.com/KenSuenobu/rust-pushrod/issues/170) `chassis: [1.4] Commands, menus, and shortcuts`

**Delivers:** Every command as an action, reachable from menus, toolbar, and familiar shortcuts.

**Builds on:** step 90, step 75.

**Show on video:**

- [ ] Every command reachable from menu and shortcut, toolbar where shown
- [ ] Commands disabled when not applicable
- [ ] Shortcut list on the docs site

### Step 93: Designer and Code tabs

**Issue:** [#171](https://github.com/KenSuenobu/rust-pushrod/issues/171) `chassis: [1.3] Designer and Code tabs`

**Delivers:** Two tabs per open form; F12 switches; each keeps its scroll, selection, and caret.

**Builds on:** step 91, step 80.

**Mockups:** [chassis-4.png](../mockups/chassis-4.png)

**Show on video:**

- [ ] F12 and clicking switch tabs
- [ ] Selection, scroll, and caret preserved across switches
- [ ] Opening another form keeps the active tab

### Step 94: New, open, and save

**Issue:** [#172](https://github.com/KenSuenobu/rust-pushrod/issues/172) `chassis: [2.2] New, open, and save`

**Delivers:** Create a project from scratch, open an existing one, save the resource file and generated code.

**Builds on:** step 70, step 92, step 71, step 88, step 63.

**Show on video:**

- [ ] A new project builds and runs (an empty form) without edits
- [ ] Save then reopen restores every resource exactly
- [ ] Closing with unsaved changes asks to save

### Step 95: Project drawer

**Issue:** [#173](https://github.com/KenSuenobu/rust-pushrod/issues/173) `chassis: [2.3] Project drawer`

**Delivers:** Resource sections (Forms, Strings, Images, Icons, Files, Menus) as expanders with lists.

**Builds on:** step 91, step 70, step 82, step 79.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Sections expand and collapse; counts correct
- [ ] Double-click opens the right editor
- [ ] Selection follows the open form

### Step 96: Manage resources and IDs

**Issue:** [#174](https://github.com/KenSuenobu/rust-pushrod/issues/174) `chassis: [2.4] Manage resources and IDs`

**Delivers:** Add, rename, duplicate, and delete resources; see and copy each resource's ID.

**Builds on:** step 95, step 65, step 66, step 77.

**Show on video:**

- [ ] Renaming a form renames its handler file and generated code; the project still builds
- [ ] Copy ID Constant copies the exact generated name
- [ ] IDs never change when other resources are added or removed

### Step 97: String table editor

**Issue:** [#175](https://github.com/KenSuenobu/rust-pushrod/issues/175) `chassis: [2.5] String table editor`

**Delivers:** Edit the resource file's strings: name, text, and ID.

**Builds on:** step 95, step 67, step 65, step 81.

**Show on video:**

- [ ] Add, edit, delete, search strings
- [ ] Widgets can use a string resource for their text
- [ ] Deleting a used string warns

### Step 98: Images and icons

**Issue:** [#176](https://github.com/KenSuenobu/rust-pushrod/issues/176) `chassis: [2.6] Images and icons`

**Delivers:** Import images and icons into the resource file and use them in widgets by ID.

**Builds on:** step 95, step 67, step 65, step 29, step 87, step 76.

**Show on video:**

- [ ] Import, preview, rename, replace, delete images and icons
- [ ] Widgets reference them by ID
- [ ] Replacing keeps the ID and updates every widget

### Step 99: Files

**Issue:** [#177](https://github.com/KenSuenobu/rust-pushrod/issues/177) `chassis: [2.8] Files`

**Delivers:** Add arbitrary files (license text, templates, data) to the resource file and load them by ID.

**Builds on:** step 95, step 67, step 65.

**Show on video:**

- [ ] Add, rename, replace, export, delete files
- [ ] Text files preview
- [ ] Files load at run time by ID

### Step 100: Checkpoint: Chassis M2: shell and project complete

**Video:** Open Chassis: create a project, browse the project drawer, add strings, images, icons, and files.

## Part 5: Chassis M3: form designer

Design surface, palette, drag and drop, selection, snapping, containers, layouts, and undo. Pushrod work this phase needs comes first.

### Step 101: Event filters: observe or intercept another widget's events

**Issue:** [#95](https://github.com/KenSuenobu/rust-pushrod/issues/95) `pushrod: [9.3] Event filters: observe or intercept another widget's events`

**Delivers:** Install a filter that sees a widget's (or the whole application's) events first and can stop them.

**Builds on:** step 16.

**Show on video:**

- [ ] Widget, subtree, and application filters
- [ ] Stop prevents delivery
- [ ] Removal works

### Step 102: Drawing modes: live or outline feedback while moving and resizing

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

### Step 103: Design-mode hooks for Chassis

**Issue:** [#68](https://github.com/KenSuenobu/rust-pushrod/issues/68) `pushrod: [6.9] Design-mode hooks for Chassis`

**Delivers:** Select, move, resize, and edit widgets without triggering their behavior.

**Builds on:** step 59, step 101, step 77, step 48, step 38.

**Show on video:**

- [ ] No widget reacts in design mode
- [ ] Select/move/resize/reparent, undoable
- [ ] Back to run mode restores behavior

### Step 104: Widget libraries: extend or replace pushrod-widgets

**Issue:** [#131](https://github.com/KenSuenobu/rust-pushrod/issues/131) `pushrod: [1.22] Widget libraries: extend or replace pushrod-widgets`

**Delivers:** Make it straightforward for a separate crate to add widgets alongside pushrod-widgets, or to replace it with a different look.

**Builds on:** step 1, step 58.

**Show on video:**

- [ ] A separate crate adds a widget that works with IDs, events, callbacks, and focus like a standard one
- [ ] A replacement library registers standard type names and a form loads with its look
- [ ] Docs page explains extending and replacing

### Step 105: Design surface

**Issue:** [#178](https://github.com/KenSuenobu/rust-pushrod/issues/178) `chassis: [3.1] Design surface`

**Delivers:** The open form on a dotted grid, in Pushrod design mode.

**Builds on:** step 93, step 70, step 103, step 62.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Forms look exactly as at run time
- [ ] Widgets don't react in the designer
- [ ] Grid size and visibility adjustable

### Step 106: Widget palette

**Issue:** [#179](https://github.com/KenSuenobu/rust-pushrod/issues/179) `chassis: [3.2] Widget palette`

**Delivers:** Every registered widget type, searchable and grouped, including layouts.

**Builds on:** step 91, step 58, step 104.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] All registered types listed and searchable
- [ ] A widget library added to the project appears in the palette

### Step 107: Selection, move, and resize

**Issue:** [#180](https://github.com/KenSuenobu/rust-pushrod/issues/180) `chassis: [3.4] Selection, move, and resize`

**Delivers:** Click, Shift+click, marquee; eight handles; keyboard nudge.

**Builds on:** step 105, step 102.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] All gestures work and update the inspector live
- [ ] Multi-selection moves together

### Step 108: Drag and drop placement

**Issue:** [#181](https://github.com/KenSuenobu/rust-pushrod/issues/181) `chassis: [3.3] Drag and drop placement`

**Delivers:** Drag from the palette; outline preview; drop creates and selects.

**Builds on:** step 105, step 106, step 76.

**Mockups:** [chassis-2.png](../mockups/chassis-2.png)

**Show on video:**

- [ ] Outline follows the pointer and snaps; drop creates and selects
- [ ] Escape cancels; one undo removes the widget

### Step 109: Snapping and alignment

**Issue:** [#182](https://github.com/KenSuenobu/rust-pushrod/issues/182) `chassis: [3.5] Snapping and alignment`

**Delivers:** Grid snap, guides to neighbors, align and distribute commands.

**Builds on:** step 107.

**Mockups:** [chassis-2.png](../mockups/chassis-2.png)

**Show on video:**

- [ ] Guides appear when edges or centers line up
- [ ] Align and distribute work on multi-selections

### Step 110: Clipboard and undo

**Issue:** [#183](https://github.com/KenSuenobu/rust-pushrod/issues/183) `chassis: [3.7] Clipboard and undo`

**Delivers:** Cut, copy, paste, duplicate; every designer change undoable.

**Builds on:** step 107, step 77.

**Show on video:**

- [ ] Every change undoes and redoes exactly
- [ ] Pasted widgets get unique names

### Step 111: Containers, z-order, and tab order

**Issue:** [#184](https://github.com/KenSuenobu/rust-pushrod/issues/184) `chassis: [3.6] Containers, z-order, and tab order`

**Delivers:** Drop into group boxes and containers; bring forward and back; set the Tab order.

**Builds on:** step 107, step 18, step 28, step 43.

**Show on video:**

- [ ] Moving a group box moves its children
- [ ] Tab order edits persist to the resource file

### Step 112: Layout managers in the designer

**Issue:** [#185](https://github.com/KenSuenobu/rust-pushrod/issues/185) `chassis: [3.8] Layout managers in the designer`

**Delivers:** Insert widgets into Column, Row, Form, and Grid layouts; spacers; breadcrumb; live resize.

**Builds on:** step 108, step 107, step 39, step 40, step 41.

**Mockups:** [chassis-layout-1.png](../mockups/chassis-layout-1.png), [chassis-layout-2.png](../mockups/chassis-layout-2.png), [chassis-layout-3.png](../mockups/chassis-layout-3.png), [chassis-layout-4.png](../mockups/chassis-layout-4.png), [chassis-layout-5.png](../mockups/chassis-layout-5.png), [chassis-layout-6.png](../mockups/chassis-layout-6.png)

**Show on video:**

- [ ] The contact dialog can be built entirely with layouts, as in the layout mockups
- [ ] Resizing in the designer matches the running application
- [ ] Reordering by drag, and undo, work inside layouts
- [ ] Breadcrumb and Escape select parents

### Step 113: Checkpoint: Chassis M3: form designer complete

**Video:** Build the contact dialog by dragging widgets, then again with layouts; resize it live.

## Part 6: Chassis M4: inspector, events, and code

Property inspector, events menu and indicators, handler creation, and the code view. Pushrod work this phase needs comes first.

### Step 114: Choice button: pop-up list and click-to-cycle modes

**Issue:** [#44](https://github.com/KenSuenobu/rust-pushrod/issues/44) `pushrod: [3.6] Choice button: pop-up list and click-to-cycle modes`

**Delivers:** Pick one of several options compactly, as a pop-up list or a click-to-cycle button.

**Builds on:** step 38, step 37, step 11.

**Mockups:** [3.6-1.png](../mockups/3.6-1.png), [3.6-2.png](../mockups/3.6-2.png)

**Show on video:**

- [ ] Popup opens over the current item; selects by mouse/keyboard
- [ ] Cycle mode advances and reverses
- [ ] Parts are retrievable and modifiable as described

### Step 115: Context menus

**Issue:** [#88](https://github.com/KenSuenobu/rust-pushrod/issues/88) `pushrod: [2.19] Context menus`

**Delivers:** Right-click (or long-press) pop-up menus for a widget.

**Builds on:** step 17, step 83.

**Mockups:** [2.19-1.png](../mockups/2.19-1.png)

**Show on video:**

- [ ] Opens on secondary click, long-press, and Menu key
- [ ] Dynamic menus via callback

### Step 116: Layers and invalidation: cache hints, texture memory budget, incremental pipeline

**Issue:** [#113](https://github.com/KenSuenobu/rust-pushrod/issues/113) `pushrod: [1.20] Layers and invalidation: cache hints, texture memory budget, incremental pipeline`

**Delivers:** Decide which widgets get their own GPU texture, cap texture memory, and invalidate layout, paint, and compositing separately.

**Builds on:** step 5, step 45.

**Show on video:**

- [ ] Cache hints honored; Auto measurably reduces texture count on the gallery
- [ ] Memory budget enforced with eviction
- [ ] Moving a widget re-composites without repainting
- [ ] Golden images unchanged

### Step 117: Chassis: Code editor crate and editing core

**Issue:** [#142](https://github.com/KenSuenobu/rust-pushrod/issues/142) `pushrod: [11.1] Chassis: Code editor crate and editing core`

**Delivers:** Create `chassis-code-editor` with its text buffer, cursor and selection model, and edit commands, independent of drawing.

**Builds on:** step 1, step 104.

**Show on video:**

- [ ] Edits, moves, and selections covered by unit tests, including multi-byte characters
- [ ] A 100,000-line file opens and edits without noticeable delay
- [ ] Core has no dependency on drawing or the backend

### Step 118: Chassis: Code editor view: gutter, visible-line rendering, caret

**Issue:** [#143](https://github.com/KenSuenobu/rust-pushrod/issues/143) `pushrod: [11.2] Chassis: Code editor view: gutter, visible-line rendering, caret`

**Delivers:** The `CodeEditor` widget: draws line numbers, the visible lines, the current line, and the caret, and scrolls through files of any length.

**Builds on:** step 117, step 43, step 23, step 46, step 116, step 14, step 13.

**Mockups:** [11.2-1.png](../mockups/11.2-1.png)

**Show on video:**

- [ ] Smooth scrolling through a 5,000-line file
- [ ] Typing repaints only the edited line
- [ ] Sharp at 1x and 2x (1.10)
- [ ] Line numbers and current-line band as in the mockup

### Step 119: Chassis: Code editor input: keyboard, mouse selection, clipboard, input methods

**Issue:** [#144](https://github.com/KenSuenobu/rust-pushrod/issues/144) `pushrod: [11.3] Chassis: Code editor input: keyboard, mouse selection, clipboard, input methods`

**Delivers:** Editing with the keyboard and mouse the way users expect from a code editor.

**Builds on:** step 118, step 17, step 56, step 75, step 18.

**Mockups:** [11.3-1.png](../mockups/11.3-1.png)

**Show on video:**

- [ ] Every listed key and gesture works
- [ ] Composition text from an input method appears inline
- [ ] `TextChanged` and `SelectionChanged` carry the changed range

### Step 120: Chassis: Code editor undo and redo

**Issue:** [#145](https://github.com/KenSuenobu/rust-pushrod/issues/145) `pushrod: [11.4] Chassis: Code editor undo and redo`

**Delivers:** Undo and redo every edit, with consecutive typing grouped into one step.

**Builds on:** step 77, step 117.

**Show on video:**

- [ ] Typing a word and undoing removes the whole word
- [ ] Undo restores the previous selection
- [ ] Chassis edits undo as one step

### Step 121: Chassis: Code editor syntax highlighting with tree-sitter

**Issue:** [#146](https://github.com/KenSuenobu/rust-pushrod/issues/146) `pushrod: [11.5] Chassis: Code editor syntax highlighting with tree-sitter`

**Delivers:** Highlight code incrementally as it is edited, with a small, flat color scheme.

**Builds on:** step 118.

**Mockups:** [11.5-1.png](../mockups/11.5-1.png)

**Show on video:**

- [ ] Highlighting stays correct while typing inside strings and comments
- [ ] Editing re-parses incrementally, not the whole file
- [ ] Rust highlighted out of the box; other languages registrable

### Step 122: Chassis: Code editor API for the Chassis designer

**Issue:** [#149](https://github.com/KenSuenobu/rust-pushrod/issues/149) `pushrod: [11.8] Chassis: Code editor API for the Chassis designer`

**Delivers:** Let Chassis insert, find, and protect generated code: double-click a control and land in its event handler.

**Builds on:** step 121, step 120.

**Mockups:** [11.8-1.png](../mockups/11.8-1.png)

**Show on video:**

- [ ] Double-click flow from the mockup works end to end in an example
- [ ] Guarded ranges refuse edits and track surrounding edits
- [ ] Generated insertions undo as one step

### Step 123: Inspector drawer

**Issue:** [#186](https://github.com/KenSuenobu/rust-pushrod/issues/186) `chassis: [4.1] Inspector drawer`

**Delivers:** Identity header and Properties/Events tabs for the selection.

**Builds on:** step 91.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Follows the selection in the designer and the caret's handler in the code view
- [ ] Collapses and restores

### Step 124: Property grid

**Issue:** [#187](https://github.com/KenSuenobu/rust-pushrod/issues/187) `chassis: [4.2] Property grid`

**Delivers:** Every property from introspection, grouped and labeled.

**Builds on:** step 123, step 59, step 41.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png), [chassis-layout-4.png](../mockups/chassis-layout-4.png)

**Show on video:**

- [ ] Any registered widget's properties appear, including third-party widgets
- [ ] Changes show in the designer immediately and undo
- [ ] Layout settings for the selection's parent layout appear in an "In <layout>" section

### Step 125: Typed property editors

**Issue:** [#188](https://github.com/KenSuenobu/rust-pushrod/issues/188) `chassis: [4.3] Typed property editors`

**Delivers:** Text, number, check, choice, color, font, string-resource, and image editors.

**Builds on:** step 124, step 97, step 98, step 36, step 32, step 114, step 23.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Each kind has its editor; invalid values can't be applied
- [ ] Behavior flags appear as grouped check boxes
- [ ] Text and images can reference resources by ID

### Step 126: Code view

**Issue:** [#189](https://github.com/KenSuenobu/rust-pushrod/issues/189) `chassis: [5.4] Code view`

**Delivers:** The Code tab: the form's handler file in the Chassis code editor.

**Builds on:** step 93, step 118, step 119, step 122.

**Mockups:** [chassis-4.png](../mockups/chassis-4.png)

**Show on video:**

- [ ] Guarded lines can't be edited; bodies can
- [ ] Caret in a handler selects its widget

### Step 127: Create and open handlers

**Issue:** [#190](https://github.com/KenSuenobu/rust-pushrod/issues/190) `chassis: [5.3] Create and open handlers`

**Delivers:** Choosing an event (or double-clicking a widget) creates or opens its handler.

**Builds on:** step 126, step 70, step 69, step 122.

**Mockups:** [chassis-3.png](../mockups/chassis-3.png), [chassis-4.png](../mockups/chassis-4.png)

**Show on video:**

- [ ] Existing handlers open; missing ones are created once, never duplicated
- [ ] New stubs compile immediately
- [ ] Undo removes the stub and its wiring

### Step 128: Names and IDs

**Issue:** [#191](https://github.com/KenSuenobu/rust-pushrod/issues/191) `chassis: [4.4] Names and IDs`

**Delivers:** Validated, unique Rust identifiers; renames carried into constants and handlers.

**Builds on:** step 124, step 127, step 66, step 122.

**Show on video:**

- [ ] Invalid or duplicate names refused with a message
- [ ] Renaming renames handlers and the project still builds

### Step 129: Events context menu

**Issue:** [#192](https://github.com/KenSuenobu/rust-pushrod/issues/192) `chassis: [5.1] Events context menu`

**Delivers:** Right-click lists the widget's events, with a dot where code exists.

**Builds on:** step 127, step 69, step 115.

**Mockups:** [chassis-3.png](../mockups/chassis-3.png)

**Show on video:**

- [ ] Lists exactly the widget's events, default first
- [ ] Dots and handler names reflect the code file
- [ ] Choosing an event opens or creates its handler

### Step 130: Inspector Events tab

**Issue:** [#193](https://github.com/KenSuenobu/rust-pushrod/issues/193) `chassis: [5.2] Inspector Events tab`

**Delivers:** Every event with its handler name or "add..."; count on the tab.

**Builds on:** step 123, step 127, step 69.

**Mockups:** [chassis-4.png](../mockups/chassis-4.png)

**Show on video:**

- [ ] Matches the context menu's information
- [ ] Count updates as handlers are added and removed

### Step 131: Code indicator on the design surface

**Issue:** [#194](https://github.com/KenSuenobu/rust-pushrod/issues/194) `chassis: [5.5] Code indicator on the design surface`

**Delivers:** Corner mark on widgets that have handlers.

**Builds on:** step 105, step 70.

**Mockups:** [chassis-1.png](../mockups/chassis-1.png)

**Show on video:**

- [ ] Appears and disappears as handlers are added and removed

### Step 132: Keep handlers in sync

**Issue:** [#195](https://github.com/KenSuenobu/rust-pushrod/issues/195) `chassis: [5.6] Keep handlers in sync`

**Delivers:** Renames, deletions, and orphaned handlers.

**Builds on:** step 127, step 72.

**Show on video:**

- [ ] Renames update handler names and wiring
- [ ] Deleting a widget never silently deletes code

### Step 133: Checkpoint: Chassis M4: inspector, events, and code complete

**Video:** Select widgets, edit properties, right-click for events, create handlers, and write code in the Code tab.

## Part 7: Chassis M5: build, run, and release (MVP)

Build and run from Chassis, CI, documentation, and the sample project. Completes the Chassis MVP. Pushrod work this phase needs comes first.

### Step 134: Build and run

**Issue:** [#196](https://github.com/KenSuenobu/rust-pushrod/issues/196) `chassis: [6.2] Build and run`

**Delivers:** Run (F5) builds with Cargo and launches the application; output panel.

**Builds on:** step 94, step 71.

**Show on video:**

- [ ] F5 builds and launches the app
- [ ] Build output visible; failures reported

### Step 135: CI and golden screenshots

**Issue:** [#197](https://github.com/KenSuenobu/rust-pushrod/issues/197) `chassis: [7.1] CI and golden screenshots`

**Delivers:** Build, test, and compare Chassis's own screens with goldens.

**Builds on:** step 90, step 44, step 21.

**Show on video:**

- [ ] Core tests from M1 onward
- [ ] Golden images for designer, events menu, and code view

### Step 136: Chassis documentation

**Issue:** [#198](https://github.com/KenSuenobu/rust-pushrod/issues/198) `chassis: [7.2] Chassis documentation`

**Delivers:** A Chassis section on the docs site with a start-to-finish tutorial.

**Builds on:** step 134, step 3.

**Show on video:**

- [ ] All pages published with screenshots from Chassis itself
- [ ] Tutorial reproduces the sample project

### Step 137: Sample project

**Issue:** [#199](https://github.com/KenSuenobu/rust-pushrod/issues/199) `chassis: [7.3] Sample project`

**Delivers:** The contacts sample, built with Chassis, fictional data only.

**Builds on:** step 134, step 73.

**Show on video:**

- [ ] Builds and runs in CI
- [ ] Matches the tutorial

### Step 138: Checkpoint: Chassis MVP released

**Video:** Press F5: the contact dialog builds and runs. Publish the Chassis MVP.

## Part 8: Chassis example projects

Example projects built with Chassis once it reaches the MVP: small, complete applications that exercise its features and that anyone can build and test against.

### Step 139: Hello, Chassis

**Issue:** [#208](https://github.com/KenSuenobu/rust-pushrod/issues/208) `chassis: [8.1] Hello, Chassis`

**Delivers:** The smallest Chassis application: one form, a label, and a button that counts clicks.

**Builds on:** step 134, step 73, step 135, step 31, step 49.

**Mockups:** [chassis-example-hello-1.png](../mockups/chassis-example-hello-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 140: Temperature converter

**Issue:** [#209](https://github.com/KenSuenobu/rust-pushrod/issues/209) `chassis: [8.2] Temperature converter`

**Delivers:** Two linked fields that convert between Celsius and Fahrenheit as you type.

**Builds on:** step 134, step 73, step 135, step 36, step 35, step 41.

**Mockups:** [chassis-example-temperature-1.png](../mockups/chassis-example-temperature-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 141: Tip calculator

**Issue:** [#210](https://github.com/KenSuenobu/rust-pushrod/issues/210) `chassis: [8.3] Tip calculator`

**Delivers:** Enter a bill, pick a tip percentage, and split it between people.

**Builds on:** step 134, step 73, step 135, step 33, step 35, step 28, step 40.

**Mockups:** [chassis-example-tip-calculator-1.png](../mockups/chassis-example-tip-calculator-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 142: Settings dialog

**Issue:** [#211](https://github.com/KenSuenobu/rust-pushrod/issues/211) `chassis: [8.4] Settings dialog`

**Delivers:** A classic preferences dialog with OK, Cancel, and Apply.

**Builds on:** step 134, step 73, step 135, step 49, step 31, step 32, step 33, step 28.

**Mockups:** [chassis-example-settings-dialog-1.png](../mockups/chassis-example-settings-dialog-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 143: Image viewer

**Issue:** [#212](https://github.com/KenSuenobu/rust-pushrod/issues/212) `chassis: [8.5] Image viewer`

**Delivers:** Browse a set of pictures stored in the resource file.

**Builds on:** step 134, step 73, step 135, step 29, step 43, step 67.

**Mockups:** [chassis-example-image-viewer-1.png](../mockups/chassis-example-image-viewer-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 144: Stopwatch

**Issue:** [#213](https://github.com/KenSuenobu/rust-pushrod/issues/213) `chassis: [8.6] Stopwatch`

**Delivers:** Start, stop, lap, and reset, with a progress bar for the current minute.

**Builds on:** step 134, step 73, step 135, step 20, step 34.

**Mockups:** [chassis-example-stopwatch-1.png](../mockups/chassis-example-stopwatch-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 145: Quiz

**Issue:** [#214](https://github.com/KenSuenobu/rust-pushrod/issues/214) `chassis: [8.7] Quiz`

**Delivers:** A short multiple-choice quiz whose questions live in the resource file.

**Builds on:** step 134, step 73, step 135, step 33, step 34, step 50, step 67.

**Mockups:** [chassis-example-quiz-1.png](../mockups/chassis-example-quiz-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 146: Dice roller

**Issue:** [#215](https://github.com/KenSuenobu/rust-pushrod/issues/215) `chassis: [8.8] Dice roller`

**Delivers:** Roll one to four dice and keep a running total.

**Builds on:** step 134, step 73, step 135, step 87, step 35, step 40, step 67.

**Mockups:** [chassis-example-dice-1.png](../mockups/chassis-example-dice-1.png)

**Show on video:**

- [ ] Created and maintained in Chassis (no hand-edited generated code)
- [ ] CI regenerates with `chassis generate`, builds, and runs a scripted test through `Engine::dispatch` on the headless backend
- [ ] Golden screenshots match the mockup
- [ ] README lists the Chassis and Pushrod features it exercises

### Step 147: Checkpoint: All example projects built

**Video:** Tour the eight examples; show each one's CI test and golden screenshots.

## Part 9: Chassis v2

The second release: settings, menu editor, form preview, multi-selection, error list, templates, packaging, code intelligence, localization, form inheritance, bindings, project search, multiple windows, accessibility checks, source control, and more build targets. Pushrod work it needs comes first.

### Step 148: Property bindings: keep two properties in sync

**Issue:** [#98](https://github.com/KenSuenobu/rust-pushrod/issues/98) `pushrod: [9.6] Property bindings: keep two properties in sync`

**Delivers:** Bind a source widget property to a target property, one-way or two-way, with optional conversion.

**Builds on:** step 59.

**Show on video:**

- [ ] One-way and two-way bindings with transforms
- [ ] No infinite loops
- [ ] Unbind

### Step 149: Multiple windows: separate OS windows, each with its own engine

**Issue:** [#82](https://github.com/KenSuenobu/rust-pushrod/issues/82) `pushrod: [1.16] Multiple windows: separate OS windows, each with its own engine`

**Delivers:** Open additional operating-system windows (dialogs, palettes, inspectors), each driven by its own engine instance and widget tree.

**Builds on:** step 4.

**Show on video:**

- [ ] Two OS windows, each with its own engine and widget tree, render and receive input independently
- [ ] Shared application state; callbacks can open, find, and close other windows
- [ ] Closing one window doesn't end the application unless it's the last
- [ ] Works in both callback and event-loop styles

### Step 150: Bind actions by name from form files

**Issue:** [#67](https://github.com/KenSuenobu/rust-pushrod/issues/67) `pushrod: [6.8] Bind actions by name from form files`

**Delivers:** Form files name the action a control triggers; applications register handlers by name.

**Builds on:** step 62, step 75.

**Show on video:**

- [ ] Actions bind at load time
- [ ] Missing actions reported

### Step 151: no_std + alloc core crate

**Issue:** [#107](https://github.com/KenSuenobu/rust-pushrod/issues/107) `pushrod: [10.3] no_std + alloc core crate`

**Delivers:** Split the widget tree, events, and widgets into a core crate that builds without the standard library.

**Builds on:** step 55, step 21.

**Show on video:**

- [ ] `pushrod-core` builds for a `no_std` target with `alloc`
- [ ] `pushrod` API unchanged

### Step 152: Experimental: Atari ST / TOS backend

**Issue:** [#108](https://github.com/KenSuenobu/rust-pushrod/issues/108) `pushrod: [10.4] Experimental: Atari ST / TOS backend`

**Delivers:** Run Pushrod applications on an Atari ST: 68000 target, TOS input, monochrome or low-color drawing, outline mode, direct drawing.

**Builds on:** step 5, step 102, step 151.

**Show on video:**

- [ ] A Pushrod form with a button and callback runs in the Hatari emulator
- [ ] Size and speed measured and documented
- [ ] Runs on real ST hardware

### Step 153: SDL3 backend (pushrod-sdl3)

**Issue:** [#135](https://github.com/KenSuenobu/rust-pushrod/issues/135) `pushrod: [10.5] SDL3 backend (pushrod-sdl3)`

**Delivers:** A second backend crate on SDL3, a drop-in replacement for pushrod-sdl2.

**Builds on:** step 44, step 21.

**Show on video:**

- [ ] The gallery and reference application run on SDL3 unchanged
- [ ] Golden images match the SDL2 backend
- [ ] pushrod-sdl2 keeps working; applications choose either

### Step 154: Workspace settings

**Issue:** [#200](https://github.com/KenSuenobu/rust-pushrod/issues/200) `chassis: [1.5] Workspace settings`

**Delivers:** Remember window size, pane widths, open forms, and recent projects.

**Builds on:** step 91, step 94.

**Show on video:**

- [ ] Restarting restores the layout and reopens the last project
- [ ] Recent projects in the File menu

### Step 155: Menu editor

**Issue:** [#201](https://github.com/KenSuenobu/rust-pushrod/issues/201) `chassis: [2.7] Menu editor`

**Delivers:** Design menu bars and context menus stored in the resource file.

**Builds on:** step 95, step 65, step 150, step 83.

**Show on video:**

- [ ] Create, nest, reorder menu items
- [ ] Items bind to actions by name
- [ ] Menus load at run time by ID

### Step 156: Form preview

**Issue:** [#202](https://github.com/KenSuenobu/rust-pushrod/issues/202) `chassis: [3.9] Form preview`

**Delivers:** Run the form live inside Chassis without building.

**Builds on:** step 105, step 149.

**Show on video:**

- [ ] Preview behaves like the running form except for handlers
- [ ] Tab order and resizing can be tested

### Step 157: Multiple selection editing

**Issue:** [#203](https://github.com/KenSuenobu/rust-pushrod/issues/203) `chassis: [4.5] Multiple selection editing`

**Delivers:** Edit the properties several selected widgets share.

**Builds on:** step 124.

**Show on video:**

- [ ] Shared properties shown; differing values blank
- [ ] One edit applies to all, undoable in one step

### Step 158: Error list

**Issue:** [#204](https://github.com/KenSuenobu/rust-pushrod/issues/204) `chassis: [6.3] Error list`

**Delivers:** Compiler errors listed and linked to the code view.

**Builds on:** step 134, step 126.

**Show on video:**

- [ ] Errors listed and navigable
- [ ] Handler errors marked in the events list

### Step 159: Project templates

**Issue:** [#205](https://github.com/KenSuenobu/rust-pushrod/issues/205) `chassis: [6.4] Project templates`

**Delivers:** Start from an empty app, a dialog app, or the contacts sample.

**Builds on:** step 94.

**Show on video:**

- [ ] Each template builds and runs unchanged

### Step 160: Packaging and releases

**Issue:** [#206](https://github.com/KenSuenobu/rust-pushrod/issues/206) `chassis: [7.4] Packaging and releases`

**Delivers:** `cargo install chassis` and downloadable builds.

**Builds on:** step 135, step 136, step 53, step 47.

**Show on video:**

- [ ] `cargo install chassis` works
- [ ] Release builds start on a clean machine

### Step 161: Find in project

**Issue:** [#216](https://github.com/KenSuenobu/rust-pushrod/issues/216) `chassis: [1.6] Find in project`

**Delivers:** Search forms, widget names and properties, resources, and code from one place.

**Builds on:** step 70, step 126, step 79.

**Show on video:**

- [ ] One search covers forms, resources, and code
- [ ] Every result navigates to its source

### Step 162: Multiple windows per project

**Issue:** [#217](https://github.com/KenSuenobu/rust-pushrod/issues/217) `chassis: [1.7] Multiple windows per project`

**Delivers:** Design applications with several windows, each its own form, and choose which opens first.

**Builds on:** step 94, step 72, step 149.

**Show on video:**

- [ ] A project with two windows builds and runs both
- [ ] Startup window configurable

### Step 163: Localization

**Issue:** [#218](https://github.com/KenSuenobu/rust-pushrod/issues/218) `chassis: [2.9] Localization`

**Delivers:** String tables per language, a language switch in the designer, and missing-translation reports.

**Builds on:** step 97, step 65.

**Show on video:**

- [ ] Strings editable per language
- [ ] Designer previews any language
- [ ] Missing translations listed

### Step 164: Source control status

**Issue:** [#219](https://github.com/KenSuenobu/rust-pushrod/issues/219) `chassis: [2.10] Source control status`

**Delivers:** Show which forms, resources, and code files have changed since the last commit.

**Builds on:** step 95.

**Show on video:**

- [ ] Changed and new items marked
- [ ] Branch shown in the status bar
- [ ] No effect outside Git repositories

### Step 165: Visual form inheritance

**Issue:** [#220](https://github.com/KenSuenobu/rust-pushrod/issues/220) `chassis: [3.10] Visual form inheritance`

**Delivers:** Base a form on another form and override only what differs.

**Builds on:** step 112, step 70, step 65.

**Show on video:**

- [ ] Derived forms update when the base changes
- [ ] Overrides survive base changes
- [ ] Inherited widgets visibly marked

### Step 166: Accessibility and usability checks

**Issue:** [#221](https://github.com/KenSuenobu/rust-pushrod/issues/221) `chassis: [3.11] Accessibility and usability checks`

**Delivers:** Warn about missing labels, Tab order gaps, mnemonic conflicts, and tiny click targets.

**Builds on:** step 111, step 73, step 60, step 57.

**Show on video:**

- [ ] Each check detects its problem on a test form
- [ ] Problems navigate to the widget
- [ ] `chassis check` reports them

### Step 167: Data bindings in the inspector

**Issue:** [#222](https://github.com/KenSuenobu/rust-pushrod/issues/222) `chassis: [4.6] Data bindings in the inspector`

**Delivers:** Bind a widget property to another property or to application state without writing code.

**Builds on:** step 124, step 148.

**Show on video:**

- [ ] A slider-to-label binding works without code
- [ ] Bindings round-trip through the resource file
- [ ] Bound properties marked in the inspector

### Step 168: Code intelligence with rust-analyzer

**Issue:** [#223](https://github.com/KenSuenobu/rust-pushrod/issues/223) `chassis: [5.8] Code intelligence with rust-analyzer`

**Delivers:** Completion, hover, signature help, and inline errors in the Code tab, through the Language Server Protocol.

**Builds on:** step 126, step 122.

**Show on video:**

- [ ] Completion and hover work in handler files
- [ ] Errors appear inline while typing
- [ ] Chassis works when rust-analyzer is absent

### Step 169: Build targets, including the Atari ST

**Issue:** [#224](https://github.com/KenSuenobu/rust-pushrod/issues/224) `chassis: [6.6] Build targets, including the Atari ST`

**Delivers:** Choose a build target and backend: desktop SDL2, SDL3 when available, and the experimental Atari ST backend.

**Builds on:** step 134, step 153, step 152.

**Show on video:**

- [ ] Switching targets builds the right backend
- [ ] Atari ST build produces a runnable program in an emulator

### Step 170: Checkpoint: Chassis v2 released

**Video:** Show the v2 features, including an Atari ST build running in an emulator.

## Part 10: Remaining Pushrod backlog

Pushrod issues not needed by Chassis, in roadmap order. Theming (Epic 8) stays last, as the distant future it was planned to be.

### Step 171: Callback handles: disconnect, block, and unblock

**Issue:** [#99](https://github.com/KenSuenobu/rust-pushrod/issues/99) `pushrod: [9.7] Callback handles: disconnect, block, and unblock`

**Delivers:** Every registration returns a handle to remove or temporarily silence the callback.

**Builds on:** step 11.

**Show on video:**

- [ ] Disconnect removes; block suppresses until unblock
- [ ] Scoped blocking restores state even on early return

### Step 172: Pointer cursor per widget

**Issue:** [#101](https://github.com/KenSuenobu/rust-pushrod/issues/101) `pushrod: [9.9] Pointer cursor per widget`

**Delivers:** Widgets choose the pointer shape (I-beam over text, resize arrows, hand over links).

**Builds on:** step 4.

**Show on video:**

- [ ] Cursor changes on hover and restores on exit
- [ ] Busy cursor while `ui.set_busy(true)`

### Step 173: Application lifecycle: startup and close-request callbacks

**Issue:** [#26](https://github.com/KenSuenobu/rust-pushrod/issues/26) `pushrod: [1.11] Application lifecycle: startup and close-request callbacks`

**Delivers:** Run code before the first frame; confirm or veto closing the window.

**Builds on:** step 4.

**Show on video:**

- [ ] `on_start` runs once before the first draw
- [ ] `on_close_request` can veto
- [ ] Tests via `Engine::dispatch`

### Step 174: Grid layout

**Issue:** [#119](https://github.com/KenSuenobu/rust-pushrod/issues/119) `pushrod: [5.8] Grid layout`

**Delivers:** Rows and columns of cells with spans, per-row/column stretch, and alignment.

**Builds on:** step 39.

**Mockups:** [5.8-1.png](../mockups/5.8-1.png)

**Show on video:**

- [ ] Spans, stretch, alignment, spacing
- [ ] Correct min/preferred sizes from cell hints

### Step 175: Stack layout (one child visible at a time)

**Issue:** [#120](https://github.com/KenSuenobu/rust-pushrod/issues/120) `pushrod: [5.9] Stack layout (one child visible at a time)`

**Delivers:** Children occupy the same area; one is shown at a time (pages, wizards, tab contents).

**Builds on:** step 39.

**Show on video:**

- [ ] Switching pages hides others and keeps their state
- [ ] Size policy: largest or current

### Step 176: Flow layout (wrapping rows)

**Issue:** [#121](https://github.com/KenSuenobu/rust-pushrod/issues/121) `pushrod: [5.10] Flow layout (wrapping rows)`

**Delivers:** Children placed left to right, wrapping to the next row when the width runs out.

**Builds on:** step 39.

**Mockups:** [5.10-1.png](../mockups/5.10-1.png)

**Show on video:**

- [ ] Reflows on resize
- [ ] Height-for-width measurement

### Step 177: Column browser (multi-column hierarchical browser)

**Issue:** [#45](https://github.com/KenSuenobu/rust-pushrod/issues/45) `pushrod: [3.7] Column browser (multi-column hierarchical browser)`

**Delivers:** Columns of lists where selecting an item in one column fills the next, as in the NeXTSTEP browser and the Mac Finder's column view.

**Builds on:** step 38, step 79, step 43.

**Mockups:** [3.7-1.png](../mockups/3.7-1.png), [3.7-2.png](../mockups/3.7-2.png)

**Show on video:**

- [ ] Selecting in column n fills column n+1 and clears columns to its right
- [ ] Leaf selection clears columns to the right; optional preview pane
- [ ] Horizontal scrolling keeps the newest column visible; column widths are resizable
- [ ] Keyboard navigation: Up/Down, Right into a branch, Left to parent, type-ahead, Return
- [ ] `path()`/`set_path()`, `columns()` part access
- [ ] Works with a filesystem model and an in-memory tree model

### Step 178: Spin box

**Issue:** [#49](https://github.com/KenSuenobu/rust-pushrod/issues/49) `pushrod: [4.2] Spin box`

**Delivers:** Numeric field with up/down arrows, range, and step.

**Builds on:** step 38, step 36, step 30.

**Mockups:** [4.2-1.png](../mockups/4.2-1.png)

**Show on video:**

- [ ] Arrows, keys, wheel adjust within range
- [ ] Typed values clamp
- [ ] Parts are retrievable and modifiable as described

### Step 179: Combo box

**Issue:** [#53](https://github.com/KenSuenobu/rust-pushrod/issues/53) `pushrod: [4.6] Combo box`

**Delivers:** Text field with a drop-down list of suggestions.

**Builds on:** step 38, step 36, step 37, step 79.

**Mockups:** [4.6-1.png](../mockups/4.6-1.png)

**Show on video:**

- [ ] Typing filters; arrow opens; choosing fills
- [ ] Parts are retrievable and modifiable as described

### Step 180: Date picker

**Issue:** [#91](https://github.com/KenSuenobu/rust-pushrod/issues/91) `pushrod: [3.11] Date picker`

**Delivers:** A date field with a drop-down month calendar.

**Builds on:** step 38, step 36, step 37.

**Mockups:** [3.11-1.png](../mockups/3.11-1.png)

**Show on video:**

- [ ] Calendar navigation by mouse and keyboard
- [ ] Typed and picked dates agree
- [ ] Parts are retrievable and modifiable as described

### Step 181: Tooltips

**Issue:** [#77](https://github.com/KenSuenobu/rust-pushrod/issues/77) `pushrod: [2.13] Tooltips`

**Delivers:** Short help text shown after the pointer rests on a widget.

**Builds on:** step 20, step 37.

**Mockups:** [2.13-1.png](../mockups/2.13-1.png)

**Show on video:**

- [ ] Appears after the delay and hides correctly
- [ ] Stays inside the window; never under the pointer
- [ ] Emits the corresponding `EngineEvent` (9.10)

### Step 182: Switch (on/off slide toggle)

**Issue:** [#86](https://github.com/KenSuenobu/rust-pushrod/issues/86) `pushrod: [2.17] Switch (on/off slide toggle)`

**Delivers:** A sliding on/off switch for settings that take effect immediately.

**Builds on:** step 6, step 11.

**Mockups:** [2.17-1.png](../mockups/2.17-1.png)

**Show on video:**

- [ ] Click, drag, Space toggle

### Step 183: Multi-line text area

**Issue:** [#87](https://github.com/KenSuenobu/rust-pushrod/issues/87) `pushrod: [2.18] Multi-line text area`

**Delivers:** Multi-line plain-text editing with scrolling, selection, clipboard, and undo.

**Builds on:** step 36, step 14, step 43.

**Mockups:** [2.18-1.png](../mockups/2.18-1.png)

**Show on video:**

- [ ] Editing, selection, clipboard, undo
- [ ] Smooth with 10,000 lines

### Step 184: Message bar (non-modal inline notice)

**Issue:** [#89](https://github.com/KenSuenobu/rust-pushrod/issues/89) `pushrod: [2.20] Message bar (non-modal inline notice)`

**Delivers:** A dismissible banner for warnings and information that doesn't block the window.

**Builds on:** step 20, step 38, step 31.

**Mockups:** [2.20-1.png](../mockups/2.20-1.png)

**Show on video:**

- [ ] Shows, hides, and dismisses
- [ ] Action buttons fire callbacks
- [ ] Parts are retrievable and modifiable as described

### Step 185: Link label

**Issue:** [#126](https://github.com/KenSuenobu/rust-pushrod/issues/126) `pushrod: [2.22] Link label`

**Delivers:** Clickable text that looks like a link and opens a URL or runs a callback.

**Builds on:** step 18, step 11.

**Mockups:** [2.22-1.png](../mockups/2.22-1.png)

**Show on video:**

- [ ] Click, Return, and Space activate
- [ ] URL targets open the system browser unless the callback cancels
- [ ] Hover cursor and focus outline

### Step 186: Dial (rotary knob)

**Issue:** [#127](https://github.com/KenSuenobu/rust-pushrod/issues/127) `pushrod: [3.12] Dial (rotary knob)`

**Delivers:** A round control turned by dragging, the wheel, or the keyboard, for values like volume, pan, or angle.

**Builds on:** step 17, step 11.

**Mockups:** [3.12-1.png](../mockups/3.12-1.png)

**Show on video:**

- [ ] Drag, wheel, and keys change the value within range
- [ ] Wrapping mode for angles
- [ ] Notches and optional value text

### Step 187: Busy spinner

**Issue:** [#128](https://github.com/KenSuenobu/rust-pushrod/issues/128) `pushrod: [4.10] Busy spinner`

**Delivers:** A small animated indicator for work of unknown length, for use inline or on buttons.

**Builds on:** step 20.

**Mockups:** [4.10-1.png](../mockups/4.10-1.png)

**Show on video:**

- [ ] Animates only while running and visible
- [ ] Static fallback in outline mode
- [ ] Fits inline with text and inside buttons

### Step 188: Dirty-rectangle redraw

**Issue:** [#24](https://github.com/KenSuenobu/rust-pushrod/issues/24) `pushrod: [1.9] Dirty-rectangle redraw`

**Delivers:** Only re-blit the screen regions that changed.

**Builds on:** step 44.

**Show on video:**

- [ ] One changed widget copies only intersecting textures
- [ ] Golden tests (7.2) pass
- [ ] Benchmark in the PR

### Step 189: Chassis: Code editor find

**Issue:** [#147](https://github.com/KenSuenobu/rust-pushrod/issues/147) `pushrod: [11.6] Chassis: Code editor find`

**Delivers:** A find bar inside the editor: type to search, step through matches.

**Builds on:** step 36, step 30, step 119.

**Mockups:** [11.6-1.png](../mockups/11.6-1.png)

**Show on video:**

- [ ] Matches update as the search text changes
- [ ] Stepping wraps around the file
- [ ] Escape returns focus to the editor at the current match

### Step 190: Chassis: Code editor auto-indent and bracket matching

**Issue:** [#148](https://github.com/KenSuenobu/rust-pushrod/issues/148) `pushrod: [11.7] Chassis: Code editor auto-indent and bracket matching`

**Delivers:** New lines keep and adjust indentation, and the bracket matching the one at the caret is boxed.

**Builds on:** step 121, step 119.

**Mockups:** [11.7-1.png](../mockups/11.7-1.png)

**Show on video:**

- [ ] Return after '{' indents one level
- [ ] Typing '}' on a blank line outdents it
- [ ] Matching brackets boxed; brackets in strings ignored

### Step 191: Chassis: Code editor documentation, example, and tests

**Issue:** [#150](https://github.com/KenSuenobu/rust-pushrod/issues/150) `pushrod: [11.9] Chassis: Code editor documentation, example, and tests`

**Delivers:** Document the code editor on the site, ship an example editor, and cover it with golden-image tests.

**Builds on:** step 44, step 122, step 189, step 190.

**Show on video:**

- [ ] Docs pages published with screenshots from the example
- [ ] Golden images for each mockup state
- [ ] Benchmarks in CI

### Step 192: Theme for colors and metrics (future)

**Issue:** [#74](https://github.com/KenSuenobu/rust-pushrod/issues/74) `pushrod: [8.1] Theme for colors and metrics (future)`

**Delivers:** Move hard-coded colors and metrics into a theme; the default reproduces today's look.

**Builds on:** step 44.

**Show on video:**

- [ ] (Future) classic theme matches goldens

### Step 193: Skins: image-based widget rendering (future)

**Issue:** [#75](https://github.com/KenSuenobu/rust-pushrod/issues/75) `pushrod: [8.2] Skins: image-based widget rendering (future)`

**Delivers:** Let a skin replace paint with nine-slice images.

**Builds on:** step 192.

**Show on video:**

- [ ] (Future) sample skin renders the gallery
