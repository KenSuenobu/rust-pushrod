# GitHub projects

The work is organized in five GitHub projects. This file lists what each contains; `scripts/github-projects.json` holds the same data, and `scripts/create_github_projects.py` creates or updates the projects from it.

| Project | Contains | Items |
|---|---|---|
| **Pushrod Core MVP** | Pushrod's MVP: v0.1 (dialog applications) and v0.2 (forms and resources). | 75 |
| **Chassis MVP** | The Chassis MVP (phases M1 to M5) and the post-MVP Pushrod work it depends on. | 80 |
| **Chassis v2** | Chassis v2 and the additional Pushrod work it needs. | 22 |
| **Extra Widgets** | The pushrod-extra-widgets crate: larger and compound widgets outside the Pushrod MVP. | 51 |
| **Chassis Examples** | Example projects built with Chassis once it reaches the MVP; open for anyone to pick up. | 9 |

For the exact step-by-step order across all five projects, see [`roadmaps/ORDER_OF_EXECUTION.md`](roadmaps/ORDER_OF_EXECUTION.md).

Order of work: **Pushrod Core MVP** first; then **Chassis MVP**, whose "Pushrod for Mx" phases are the post-MVP Pushrod issues each Chassis phase needs; then **Chassis Examples** and **Chassis v2**; then **Extra Widgets**.

## Creating the projects

GitHub projects belong to your account, so they are created from your machine with the GitHub CLI:

```bash
gh auth refresh -s project            # once: allow the CLI to manage projects
python3 scripts/create_github_projects.py --dry-run   # show what would happen
python3 scripts/create_github_projects.py             # create or update the five projects
```

The script is safe to re-run: it reuses projects with the same title, skips items already added, and updates each item's **Phase** and **Order** fields.

## Pushrod Core MVP

Every issue in Pushrod's MVP, in work order. Group by **Phase** (v0.1, v0.2) and sort by **Order**. Roadmap: docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md.

### Epics (9)

| Order | Issue | Title |
|---|---|---|
| 1 | [#11](https://github.com/KenSuenobu/rust-pushrod/issues/11) | pushrod: [1] Epic: Core: behavior flags, focus, timing, and performance |
| 2 | [#12](https://github.com/KenSuenobu/rust-pushrod/issues/12) | pushrod: [2] Epic: Core dialog widgets |
| 3 | [#13](https://github.com/KenSuenobu/rust-pushrod/issues/13) | pushrod: [3] Epic: Value and choice controls |
| 4 | [#14](https://github.com/KenSuenobu/rust-pushrod/issues/14) | pushrod: [4] Epic: Lists, tables, progress, spin boxes, and tabs |
| 5 | [#15](https://github.com/KenSuenobu/rust-pushrod/issues/15) | pushrod: [5] Epic: Composition, clipping, and layout managers |
| 6 | [#16](https://github.com/KenSuenobu/rust-pushrod/issues/16) | pushrod: [6] Epic: Forms and form files (foundation for Chassis) |
| 7 | [#17](https://github.com/KenSuenobu/rust-pushrod/issues/17) | pushrod: [7] Epic: Quality, documentation, and release |
| 8 | [#79](https://github.com/KenSuenobu/rust-pushrod/issues/79) | pushrod: [9] Epic: Event model and callbacks |
| 9 | [#103](https://github.com/KenSuenobu/rust-pushrod/issues/103) | pushrod: [10] Epic: Rendering backends, drawing modes, and vintage hardware |

### v0.1: dialog applications (53)

| Order | Issue | Title |
|---|---|---|
| 10 | [#130](https://github.com/KenSuenobu/rust-pushrod/issues/130) | pushrod: [1.21] Split the standard widgets into the pushrod-widgets crate |
| 11 | [#69](https://github.com/KenSuenobu/rust-pushrod/issues/69) | pushrod: [7.1] CI: build, test, clippy, fmt, doc with SDL2 |
| 12 | [#122](https://github.com/KenSuenobu/rust-pushrod/issues/122) | pushrod: [7.8] Documentation site: build pushrod-docs in CI and publish it |
| 13 | [#93](https://github.com/KenSuenobu/rust-pushrod/issues/93) | pushrod: [9.1] Event vocabulary: modern event names and structure |
| 14 | [#106](https://github.com/KenSuenobu/rust-pushrod/issues/106) | pushrod: [10.2] Backend-neutral paint API (SDL2 backend by default) |
| 15 | [#19](https://github.com/KenSuenobu/rust-pushrod/issues/19) | pushrod: [1.1] Widget behavior flags and visual states |
| 16 | [#6](https://github.com/KenSuenobu/rust-pushrod/issues/6) | pushrod: [1.2] Event masks for the run loop and for widgets |
| 17 | [#109](https://github.com/KenSuenobu/rust-pushrod/issues/109) | pushrod: [9.10] Engine events: every internal action Pushrod performs emits an event |
| 18 | [#124](https://github.com/KenSuenobu/rust-pushrod/issues/124) | pushrod: [9.11] Event-loop style: keep parity with callbacks |
| 19 | [#7](https://github.com/KenSuenobu/rust-pushrod/issues/7) | pushrod: [1.3] Broadcast events to all interested widgets |
| 20 | [#22](https://github.com/KenSuenobu/rust-pushrod/issues/22) | pushrod: [1.6] Callback registry for every widget event |
| 21 | [#78](https://github.com/KenSuenobu/rust-pushrod/issues/78) | pushrod: [1.13] Custom application events with payloads |
| 22 | [#111](https://github.com/KenSuenobu/rust-pushrod/issues/111) | pushrod: [1.18] Text layout engine: Parley for shaping, bidi, line breaking, and font fallback (with an SDL2 compatibility gate) |
| 23 | [#3](https://github.com/KenSuenobu/rust-pushrod/issues/3) | pushrod: [1.7] GPU text: font cache with a glyph atlas texture |
| 24 | [#71](https://github.com/KenSuenobu/rust-pushrod/issues/71) | pushrod: [7.3] Widget gallery example |
| 26 | [#94](https://github.com/KenSuenobu/rust-pushrod/issues/94) | pushrod: [9.2] Event propagation: capture, target, and bubble phases with handled/ignored |
| 27 | [#96](https://github.com/KenSuenobu/rust-pushrod/issues/96) | pushrod: [9.4] Reusable input controllers: click, drag, scroll, hover, long-press, key |
| 28 | [#20](https://github.com/KenSuenobu/rust-pushrod/issues/20) | pushrod: [1.4] Keyboard focus, Tab order, and Return/Escape |
| 30 | [#104](https://github.com/KenSuenobu/rust-pushrod/issues/104) | pushrod: [1.17] Per-state appearance: colors, border, and font for each visual state |
| 32 | [#21](https://github.com/KenSuenobu/rust-pushrod/issues/21) | pushrod: [1.5] Timers and idle callbacks |
| 33 | [#134](https://github.com/KenSuenobu/rust-pushrod/issues/134) | pushrod: [10.6] Backend crates: keep SDL out of the core and the widgets |
| 34 | [#138](https://github.com/KenSuenobu/rust-pushrod/issues/138) | pushrod: [1.25] Errors: return Result instead of panicking |
| 35 | [#137](https://github.com/KenSuenobu/rust-pushrod/issues/137) | pushrod: [1.24] Fonts: shared font handles and a built-in default font |
| 36 | [#136](https://github.com/KenSuenobu/rust-pushrod/issues/136) | pushrod: [1.23] Public API polish before publishing: builders, prelude, and stability |
| 37 | [#36](https://github.com/KenSuenobu/rust-pushrod/issues/36) | pushrod: [2.10] Custom paint widget (closure-drawn) |
| 38 | [#27](https://github.com/KenSuenobu/rust-pushrod/issues/27) | pushrod: [2.1] Boxed label widget |
| 39 | [#83](https://github.com/KenSuenobu/rust-pushrod/issues/83) | pushrod: [2.14] Separator line |
| 40 | [#41](https://github.com/KenSuenobu/rust-pushrod/issues/41) | pushrod: [3.3] Group box (titled frame) |
| 41 | [#33](https://github.com/KenSuenobu/rust-pushrod/issues/33) | pushrod: [2.7] Image widget |
| 42 | [#28](https://github.com/KenSuenobu/rust-pushrod/issues/28) | pushrod: [2.2] Glyph button (small single-character button) |
| 43 | [#29](https://github.com/KenSuenobu/rust-pushrod/issues/29) | pushrod: [2.3] Button behaviors: default, cancel, toggle, activate-on-press, dismiss dialog |
| 44 | [#30](https://github.com/KenSuenobu/rust-pushrod/issues/30) | pushrod: [2.4] Check box with indeterminate state |
| 45 | [#31](https://github.com/KenSuenobu/rust-pushrod/issues/31) | pushrod: [2.5] Radio buttons and radio groups |
| 46 | [#48](https://github.com/KenSuenobu/rust-pushrod/issues/48) | pushrod: [4.1] Progress bar |
| 47 | [#40](https://github.com/KenSuenobu/rust-pushrod/issues/40) | pushrod: [3.2] Slider |
| 48 | [#32](https://github.com/KenSuenobu/rust-pushrod/issues/32) | pushrod: [2.6] Text field: single-line entry with input masks, validation, and numeric mode |
| 49 | [#56](https://github.com/KenSuenobu/rust-pushrod/issues/56) | pushrod: [5.1] Clip children to their parent's bounds |
| 50 | [#57](https://github.com/KenSuenobu/rust-pushrod/issues/57) | pushrod: [5.2] Composite widgets: build widgets from parts that applications can retrieve and modify |
| 51 | [#116](https://github.com/KenSuenobu/rust-pushrod/issues/116) | pushrod: [5.5] Layout manager foundation: Layout trait, size hints, measure and arrange |
| 52 | [#117](https://github.com/KenSuenobu/rust-pushrod/issues/117) | pushrod: [5.6] Row and column layouts |
| 53 | [#118](https://github.com/KenSuenobu/rust-pushrod/issues/118) | pushrod: [5.7] Form layout: aligned label and field rows |
| 54 | [#39](https://github.com/KenSuenobu/rust-pushrod/issues/39) | pushrod: [3.1] Scroll bar |
| 55 | [#58](https://github.com/KenSuenobu/rust-pushrod/issues/58) | pushrod: [5.3] Container: a scrollable viewport onto fixed-size content |
| 56 | [#70](https://github.com/KenSuenobu/rust-pushrod/issues/70) | pushrod: [7.2] Golden-image rendering tests |
| 57 | [#115](https://github.com/KenSuenobu/rust-pushrod/issues/115) | pushrod: [7.7] Performance benchmarks and budgets |
| 58 | [#25](https://github.com/KenSuenobu/rust-pushrod/issues/25) | pushrod: [1.10] High-DPI rendering and window resize |
| 59 | [#139](https://github.com/KenSuenobu/rust-pushrod/issues/139) | pushrod: [7.9] Easy installation: bundled SDL, static linking, and platform guides |
| 70 | [#23](https://github.com/KenSuenobu/rust-pushrod/issues/23) | pushrod: [1.8] Edit the widget tree at run time: remove, reparent, reorder |
| 71 | [#66](https://github.com/KenSuenobu/rust-pushrod/issues/66) | pushrod: [6.7] Modal dialogs: run a form until a dismissing control is chosen |
| 72 | [#35](https://github.com/KenSuenobu/rust-pushrod/issues/35) | pushrod: [2.9] Alert dialog |
| 73 | [#140](https://github.com/KenSuenobu/rust-pushrod/issues/140) | pushrod: [7.10] Reference application: a real app built only with Pushrod |
| 74 | [#72](https://github.com/KenSuenobu/rust-pushrod/issues/72) | pushrod: [7.4] Guide: writing widgets and composites |
| 75 | [#73](https://github.com/KenSuenobu/rust-pushrod/issues/73) | pushrod: [7.5] Changelog, Cargo metadata, and crates.io release |

### v0.2: forms (MVP complete) (13)

| Order | Issue | Title |
|---|---|---|
| 25 | [#114](https://github.com/KenSuenobu/rust-pushrod/issues/114) | pushrod: [7.6] Module boundaries and written specs with conformance tests |
| 29 | [#112](https://github.com/KenSuenobu/rust-pushrod/issues/112) | pushrod: [1.19] Input method (IME) support: composition text and candidate window |
| 31 | [#76](https://github.com/KenSuenobu/rust-pushrod/issues/76) | pushrod: [1.12] Keyboard mnemonics: underlined shortcut letters |
| 60 | [#60](https://github.com/KenSuenobu/rust-pushrod/issues/60) | pushrod: [6.1] Widget registry and factory |
| 61 | [#61](https://github.com/KenSuenobu/rust-pushrod/issues/61) | pushrod: [6.2] Property introspection: list, get, and set widget properties by name |
| 62 | [#80](https://github.com/KenSuenobu/rust-pushrod/issues/80) | pushrod: [1.14] Accessibility: expose widgets to screen readers |
| 63 | [#62](https://github.com/KenSuenobu/rust-pushrod/issues/62) | pushrod: [6.3] Form file format (binary, versioned) |
| 64 | [#63](https://github.com/KenSuenobu/rust-pushrod/issues/63) | pushrod: [6.4] Form loader: build a form from a file with stable IDs |
| 65 | [#64](https://github.com/KenSuenobu/rust-pushrod/issues/64) | pushrod: [6.5] Form writer: save a widget subtree to a file |
| 66 | [#65](https://github.com/KenSuenobu/rust-pushrod/issues/65) | pushrod: [6.6] Generate ID constants from a form file |
| 67 | [#154](https://github.com/KenSuenobu/rust-pushrod/issues/154) | pushrod: [6.11] Resource kinds: strings, images, icons, files, menus, and alerts |
| 68 | [#155](https://github.com/KenSuenobu/rust-pushrod/issues/155) | pushrod: [6.12] Typed resource IDs: generated constants per resource kind |
| 69 | [#153](https://github.com/KenSuenobu/rust-pushrod/issues/153) | pushrod: [6.10] Resource manager: load resource sets and get resources by ID |

## Chassis MVP

Builds on the Pushrod Core MVP project. Each "Pushrod for Mx" phase holds the post-MVP Pushrod issues that Chassis phase Mx needs (directly or indirectly), and each "Extra widgets for Mx" phase the pushrod-extra-widgets issues it needs; they come before it in **Order**. Roadmap: docs/ROADMAP_CHASSIS.md (Action Plan).

### Epics (8)

| Order | Issue | Title |
|---|---|---|
| 1 | [#157](https://github.com/KenSuenobu/rust-pushrod/issues/157) | chassis: [1] Epic: Application shell |
| 2 | [#158](https://github.com/KenSuenobu/rust-pushrod/issues/158) | chassis: [2] Epic: Project and resource file |
| 3 | [#159](https://github.com/KenSuenobu/rust-pushrod/issues/159) | chassis: [3] Epic: Form designer |
| 4 | [#160](https://github.com/KenSuenobu/rust-pushrod/issues/160) | chassis: [4] Epic: Inspector |
| 5 | [#161](https://github.com/KenSuenobu/rust-pushrod/issues/161) | chassis: [5] Epic: Events and code |
| 6 | [#162](https://github.com/KenSuenobu/rust-pushrod/issues/162) | chassis: [6] Epic: Build and run |
| 7 | [#163](https://github.com/KenSuenobu/rust-pushrod/issues/163) | chassis: [7] Epic: Quality, documentation, and release |
| 8 | [#141](https://github.com/KenSuenobu/rust-pushrod/issues/141) | pushrod: [11] Epic: Chassis: code editor (chassis-code-editor) |

### Pushrod for M1 (1)

| Order | Issue | Title |
|---|---|---|
| 9 | [#156](https://github.com/KenSuenobu/rust-pushrod/issues/156) | pushrod: [6.13] Event metadata in the widget registry |

### M1: core without UI (4)

| Order | Issue | Title |
|---|---|---|
| 10 | [#164](https://github.com/KenSuenobu/rust-pushrod/issues/164) | chassis: [2.1] Project model |
| 11 | [#165](https://github.com/KenSuenobu/rust-pushrod/issues/165) | chassis: [6.1] Code generation rules |
| 12 | [#166](https://github.com/KenSuenobu/rust-pushrod/issues/166) | chassis: [5.7] Event wiring generation |
| 13 | [#167](https://github.com/KenSuenobu/rust-pushrod/issues/167) | chassis: [6.5] Command-line generator |

### Pushrod for M2 (16)

| Order | Issue | Title |
|---|---|---|
| 14 | [#97](https://github.com/KenSuenobu/rust-pushrod/issues/97) | pushrod: [9.5] Actions and keyboard shortcuts: shared commands for menus, toolbars, and buttons |
| 15 | [#100](https://github.com/KenSuenobu/rust-pushrod/issues/100) | pushrod: [9.8] Drag and drop within the application |
| 16 | [#81](https://github.com/KenSuenobu/rust-pushrod/issues/81) | pushrod: [1.15] Undo and redo command stack |
| 17 | [#59](https://github.com/KenSuenobu/rust-pushrod/issues/59) | pushrod: [5.4] Autoresizing (keep margins or stretch on resize) |
| 18 | [#52](https://github.com/KenSuenobu/rust-pushrod/issues/52) | pushrod: [4.5] List view |
| 19 | [#51](https://github.com/KenSuenobu/rust-pushrod/issues/51) | pushrod: [4.4] Tab view |
| 20 | [#92](https://github.com/KenSuenobu/rust-pushrod/issues/92) | pushrod: [4.9] Table view |
| 21 | [#90](https://github.com/KenSuenobu/rust-pushrod/issues/90) | pushrod: [3.10] Expander (collapsible section) |
| 22 | [#37](https://github.com/KenSuenobu/rust-pushrod/issues/37) | pushrod: [2.11] Menu bar and drop-down menus (drawn by Pushrod) |
| 23 | [#125](https://github.com/KenSuenobu/rust-pushrod/issues/125) | pushrod: [2.21] Image button |
| 24 | [#84](https://github.com/KenSuenobu/rust-pushrod/issues/84) | pushrod: [2.15] Toolbar |
| 25 | [#85](https://github.com/KenSuenobu/rust-pushrod/issues/85) | pushrod: [2.16] Status bar |
| 26 | [#34](https://github.com/KenSuenobu/rust-pushrod/issues/34) | pushrod: [2.8] Icon widget (picture with caption) |
| 27 | [#38](https://github.com/KenSuenobu/rust-pushrod/issues/38) | pushrod: [2.12] File chooser dialog (drawn by Pushrod) |
| 28 | [#47](https://github.com/KenSuenobu/rust-pushrod/issues/47) | pushrod: [3.9] Split view (panes with draggable dividers) |
| 29 | [#131](https://github.com/KenSuenobu/rust-pushrod/issues/131) | pushrod: [1.22] Widget libraries: extend or replace pushrod-widgets |

### Extra widgets for M2 (2)

| Order | Issue | Title |
|---|---|---|
| 30 | [#252](https://github.com/KenSuenobu/rust-pushrod/issues/252) | extra-widgets: [1.1] Create the pushrod-extra-widgets crate |
| 31 | [#257](https://github.com/KenSuenobu/rust-pushrod/issues/257) | extra-widgets: [5.1] Drawer (collapsible side panel) |

### M2: shell and project (10)

| Order | Issue | Title |
|---|---|---|
| 32 | [#168](https://github.com/KenSuenobu/rust-pushrod/issues/168) | chassis: [1.1] Chassis crate and main window |
| 33 | [#169](https://github.com/KenSuenobu/rust-pushrod/issues/169) | chassis: [1.2] Three-pane workspace |
| 34 | [#170](https://github.com/KenSuenobu/rust-pushrod/issues/170) | chassis: [1.4] Commands, menus, and shortcuts |
| 35 | [#171](https://github.com/KenSuenobu/rust-pushrod/issues/171) | chassis: [1.3] Designer and Code tabs |
| 36 | [#172](https://github.com/KenSuenobu/rust-pushrod/issues/172) | chassis: [2.2] New, open, and save |
| 37 | [#173](https://github.com/KenSuenobu/rust-pushrod/issues/173) | chassis: [2.3] Project drawer |
| 38 | [#174](https://github.com/KenSuenobu/rust-pushrod/issues/174) | chassis: [2.4] Manage resources and IDs |
| 39 | [#175](https://github.com/KenSuenobu/rust-pushrod/issues/175) | chassis: [2.5] String table editor |
| 40 | [#176](https://github.com/KenSuenobu/rust-pushrod/issues/176) | chassis: [2.6] Images and icons |
| 41 | [#177](https://github.com/KenSuenobu/rust-pushrod/issues/177) | chassis: [2.8] Files |

### Pushrod for M3 (5)

| Order | Issue | Title |
|---|---|---|
| 42 | [#95](https://github.com/KenSuenobu/rust-pushrod/issues/95) | pushrod: [9.3] Event filters: observe or intercept another widget's events |
| 43 | [#105](https://github.com/KenSuenobu/rust-pushrod/issues/105) | pushrod: [10.1] Drawing modes: live or outline feedback while moving and resizing |
| 44 | [#121](https://github.com/KenSuenobu/rust-pushrod/issues/121) | pushrod: [5.10] Flow layout (wrapping rows) |
| 45 | [#77](https://github.com/KenSuenobu/rust-pushrod/issues/77) | pushrod: [2.13] Tooltips |
| 46 | [#68](https://github.com/KenSuenobu/rust-pushrod/issues/68) | pushrod: [6.9] Design-mode hooks for Chassis |

### Extra widgets for M3 (2)

| Order | Issue | Title |
|---|---|---|
| 47 | [#42](https://github.com/KenSuenobu/rust-pushrod/issues/42) | extra-widgets: [2.6] Tool palette (grid of tool buttons) |
| 48 | [#256](https://github.com/KenSuenobu/rust-pushrod/issues/256) | extra-widgets: [4.9] Breadcrumb bar (path control) |

### M3: form designer (8)

| Order | Issue | Title |
|---|---|---|
| 49 | [#178](https://github.com/KenSuenobu/rust-pushrod/issues/178) | chassis: [3.1] Design surface |
| 50 | [#179](https://github.com/KenSuenobu/rust-pushrod/issues/179) | chassis: [3.2] Widget palette |
| 51 | [#180](https://github.com/KenSuenobu/rust-pushrod/issues/180) | chassis: [3.4] Selection, move, and resize |
| 52 | [#181](https://github.com/KenSuenobu/rust-pushrod/issues/181) | chassis: [3.3] Drag and drop placement |
| 53 | [#182](https://github.com/KenSuenobu/rust-pushrod/issues/182) | chassis: [3.5] Snapping and alignment |
| 54 | [#183](https://github.com/KenSuenobu/rust-pushrod/issues/183) | chassis: [3.7] Clipboard and undo |
| 55 | [#184](https://github.com/KenSuenobu/rust-pushrod/issues/184) | chassis: [3.6] Containers, z-order, and tab order |
| 56 | [#185](https://github.com/KenSuenobu/rust-pushrod/issues/185) | chassis: [3.8] Layout managers in the designer |

### Pushrod for M4 (9)

| Order | Issue | Title |
|---|---|---|
| 57 | [#44](https://github.com/KenSuenobu/rust-pushrod/issues/44) | pushrod: [3.6] Choice button: pop-up list and click-to-cycle modes |
| 58 | [#88](https://github.com/KenSuenobu/rust-pushrod/issues/88) | pushrod: [2.19] Context menus |
| 59 | [#113](https://github.com/KenSuenobu/rust-pushrod/issues/113) | pushrod: [1.20] Layers and invalidation: cache hints, texture memory budget, incremental pipeline |
| 60 | [#142](https://github.com/KenSuenobu/rust-pushrod/issues/142) | pushrod: [11.1] Chassis: Code editor crate and editing core |
| 61 | [#143](https://github.com/KenSuenobu/rust-pushrod/issues/143) | pushrod: [11.2] Chassis: Code editor view: gutter, visible-line rendering, caret |
| 62 | [#144](https://github.com/KenSuenobu/rust-pushrod/issues/144) | pushrod: [11.3] Chassis: Code editor input: keyboard, mouse selection, clipboard, input methods |
| 63 | [#145](https://github.com/KenSuenobu/rust-pushrod/issues/145) | pushrod: [11.4] Chassis: Code editor undo and redo |
| 64 | [#146](https://github.com/KenSuenobu/rust-pushrod/issues/146) | pushrod: [11.5] Chassis: Code editor syntax highlighting with tree-sitter |
| 65 | [#149](https://github.com/KenSuenobu/rust-pushrod/issues/149) | pushrod: [11.8] Chassis: Code editor API for the Chassis designer |

### Extra widgets for M4 (1)

| Order | Issue | Title |
|---|---|---|
| 66 | [#255](https://github.com/KenSuenobu/rust-pushrod/issues/255) | extra-widgets: [4.5] Property grid (name and value inspector) |

### M4: inspector / events and code (10)

| Order | Issue | Title |
|---|---|---|
| 67 | [#186](https://github.com/KenSuenobu/rust-pushrod/issues/186) | chassis: [4.1] Inspector drawer |
| 68 | [#187](https://github.com/KenSuenobu/rust-pushrod/issues/187) | chassis: [4.2] Property grid |
| 69 | [#188](https://github.com/KenSuenobu/rust-pushrod/issues/188) | chassis: [4.3] Typed property editors |
| 70 | [#189](https://github.com/KenSuenobu/rust-pushrod/issues/189) | chassis: [5.4] Code view |
| 71 | [#190](https://github.com/KenSuenobu/rust-pushrod/issues/190) | chassis: [5.3] Create and open handlers |
| 72 | [#191](https://github.com/KenSuenobu/rust-pushrod/issues/191) | chassis: [4.4] Names and IDs |
| 73 | [#192](https://github.com/KenSuenobu/rust-pushrod/issues/192) | chassis: [5.1] Events context menu |
| 74 | [#193](https://github.com/KenSuenobu/rust-pushrod/issues/193) | chassis: [5.2] Inspector Events tab |
| 75 | [#194](https://github.com/KenSuenobu/rust-pushrod/issues/194) | chassis: [5.5] Code indicator on the design surface |
| 76 | [#195](https://github.com/KenSuenobu/rust-pushrod/issues/195) | chassis: [5.6] Keep handlers in sync |

### M5: build / run and release (MVP) (4)

| Order | Issue | Title |
|---|---|---|
| 77 | [#196](https://github.com/KenSuenobu/rust-pushrod/issues/196) | chassis: [6.2] Build and run |
| 78 | [#197](https://github.com/KenSuenobu/rust-pushrod/issues/197) | chassis: [7.1] CI and golden screenshots |
| 79 | [#198](https://github.com/KenSuenobu/rust-pushrod/issues/198) | chassis: [7.2] Chassis documentation |
| 80 | [#199](https://github.com/KenSuenobu/rust-pushrod/issues/199) | chassis: [7.3] Sample project |

## Chassis v2

After the Chassis MVP. Pushrod prerequisites first, then the v2 issues. Roadmap: docs/ROADMAP_CHASSIS.md.

### Pushrod prerequisites (6)

| Order | Issue | Title |
|---|---|---|
| 1 | [#98](https://github.com/KenSuenobu/rust-pushrod/issues/98) | pushrod: [9.6] Property bindings: keep two properties in sync |
| 2 | [#82](https://github.com/KenSuenobu/rust-pushrod/issues/82) | pushrod: [1.16] Multiple windows: separate OS windows, each with its own engine |
| 3 | [#67](https://github.com/KenSuenobu/rust-pushrod/issues/67) | pushrod: [6.8] Bind actions by name from form files |
| 4 | [#107](https://github.com/KenSuenobu/rust-pushrod/issues/107) | pushrod: [10.3] no_std + alloc core crate |
| 5 | [#108](https://github.com/KenSuenobu/rust-pushrod/issues/108) | pushrod: [10.4] Experimental: Atari ST / TOS backend |
| 6 | [#135](https://github.com/KenSuenobu/rust-pushrod/issues/135) | pushrod: [10.5] SDL3 backend (pushrod-sdl3) |

### Chassis v2 (16)

| Order | Issue | Title |
|---|---|---|
| 7 | [#200](https://github.com/KenSuenobu/rust-pushrod/issues/200) | chassis: [1.5] Workspace settings |
| 8 | [#201](https://github.com/KenSuenobu/rust-pushrod/issues/201) | chassis: [2.7] Menu editor |
| 9 | [#202](https://github.com/KenSuenobu/rust-pushrod/issues/202) | chassis: [3.9] Form preview |
| 10 | [#203](https://github.com/KenSuenobu/rust-pushrod/issues/203) | chassis: [4.5] Multiple selection editing |
| 11 | [#204](https://github.com/KenSuenobu/rust-pushrod/issues/204) | chassis: [6.3] Error list |
| 12 | [#205](https://github.com/KenSuenobu/rust-pushrod/issues/205) | chassis: [6.4] Project templates |
| 13 | [#206](https://github.com/KenSuenobu/rust-pushrod/issues/206) | chassis: [7.4] Packaging and releases |
| 14 | [#216](https://github.com/KenSuenobu/rust-pushrod/issues/216) | chassis: [1.6] Find in project |
| 15 | [#217](https://github.com/KenSuenobu/rust-pushrod/issues/217) | chassis: [1.7] Multiple windows per project |
| 16 | [#218](https://github.com/KenSuenobu/rust-pushrod/issues/218) | chassis: [2.9] Localization |
| 17 | [#219](https://github.com/KenSuenobu/rust-pushrod/issues/219) | chassis: [2.10] Source control status |
| 18 | [#220](https://github.com/KenSuenobu/rust-pushrod/issues/220) | chassis: [3.10] Visual form inheritance |
| 19 | [#221](https://github.com/KenSuenobu/rust-pushrod/issues/221) | chassis: [3.11] Accessibility and usability checks |
| 20 | [#222](https://github.com/KenSuenobu/rust-pushrod/issues/222) | chassis: [4.6] Data bindings in the inspector |
| 21 | [#223](https://github.com/KenSuenobu/rust-pushrod/issues/223) | chassis: [5.8] Code intelligence with rust-analyzer |
| 22 | [#224](https://github.com/KenSuenobu/rust-pushrod/issues/224) | chassis: [6.6] Build targets, including the Atari ST |

## Extra Widgets

Everything in pushrod-extra-widgets not already in the Chassis MVP project (the crate, tool palette, property grid, breadcrumb bar, and drawer are there), with the post-MVP Pushrod work it needs first. Milestones 1 and 2 are the extra-widgets MVP. Roadmap: docs/ROADMAP_EXTRA_WIDGETS.md.

### Epics (6)

| Order | Issue | Title |
|---|---|---|
| 1 | [#246](https://github.com/KenSuenobu/rust-pushrod/issues/246) | extra-widgets: [1] Epic: Crate and foundations |
| 2 | [#247](https://github.com/KenSuenobu/rust-pushrod/issues/247) | extra-widgets: [2] Epic: Buttons and value controls |
| 3 | [#248](https://github.com/KenSuenobu/rust-pushrod/issues/248) | extra-widgets: [3] Epic: Pickers and special fields |
| 4 | [#249](https://github.com/KenSuenobu/rust-pushrod/issues/249) | extra-widgets: [4] Epic: Data views |
| 5 | [#250](https://github.com/KenSuenobu/rust-pushrod/issues/250) | extra-widgets: [5] Epic: Panels and navigation |
| 6 | [#251](https://github.com/KenSuenobu/rust-pushrod/issues/251) | extra-widgets: [6] Epic: Dialogs and assistants |

### Pushrod prerequisites (6)

| Order | Issue | Title |
|---|---|---|
| 7 | [#101](https://github.com/KenSuenobu/rust-pushrod/issues/101) | pushrod: [9.9] Pointer cursor per widget |
| 8 | [#120](https://github.com/KenSuenobu/rust-pushrod/issues/120) | pushrod: [5.9] Stack layout (one child visible at a time) |
| 9 | [#49](https://github.com/KenSuenobu/rust-pushrod/issues/49) | pushrod: [4.2] Spin box |
| 10 | [#53](https://github.com/KenSuenobu/rust-pushrod/issues/53) | pushrod: [4.6] Combo box |
| 11 | [#87](https://github.com/KenSuenobu/rust-pushrod/issues/87) | pushrod: [2.18] Multi-line text area |
| 12 | [#126](https://github.com/KenSuenobu/rust-pushrod/issues/126) | pushrod: [2.22] Link label |

### Extra widgets 1 - crate and Chassis widgets (2)

| Order | Issue | Title |
|---|---|---|
| 13 | [#253](https://github.com/KenSuenobu/rust-pushrod/issues/253) | extra-widgets: [1.2] Extra widgets gallery example |
| 14 | [#254](https://github.com/KenSuenobu/rust-pushrod/issues/254) | extra-widgets: [1.3] Popover: a floating panel anchored to a control |

### Extra widgets 2 - everyday controls and pickers (19)

| Order | Issue | Title |
|---|---|---|
| 15 | [#258](https://github.com/KenSuenobu/rust-pushrod/issues/258) | extra-widgets: [2.1] Menu button and split button |
| 16 | [#50](https://github.com/KenSuenobu/rust-pushrod/issues/50) | extra-widgets: [2.2] Segmented control |
| 17 | [#259](https://github.com/KenSuenobu/rust-pushrod/issues/259) | extra-widgets: [2.3] Stepper (up and down arrows) |
| 18 | [#127](https://github.com/KenSuenobu/rust-pushrod/issues/127) | extra-widgets: [2.4] Dial (rotary knob) |
| 19 | [#54](https://github.com/KenSuenobu/rust-pushrod/issues/54) | extra-widgets: [2.5] Level indicator (meter) |
| 20 | [#260](https://github.com/KenSuenobu/rust-pushrod/issues/260) | extra-widgets: [3.1] Calendar (month view) |
| 21 | [#91](https://github.com/KenSuenobu/rust-pushrod/issues/91) | extra-widgets: [3.2] Date picker |
| 22 | [#261](https://github.com/KenSuenobu/rust-pushrod/issues/261) | extra-widgets: [3.3] Time field and date-time field |
| 23 | [#46](https://github.com/KenSuenobu/rust-pushrod/issues/46) | extra-widgets: [3.4] Color well and color picker |
| 24 | [#129](https://github.com/KenSuenobu/rust-pushrod/issues/129) | extra-widgets: [3.5] Font chooser |
| 25 | [#262](https://github.com/KenSuenobu/rust-pushrod/issues/262) | extra-widgets: [3.6] Path field (file or folder with Browse) |
| 26 | [#55](https://github.com/KenSuenobu/rust-pushrod/issues/55) | extra-widgets: [4.1] Tree view (outline) |
| 27 | [#263](https://github.com/KenSuenobu/rust-pushrod/issues/263) | extra-widgets: [4.6] Check list |
| 28 | [#264](https://github.com/KenSuenobu/rust-pushrod/issues/264) | extra-widgets: [4.7] Editable list (add, remove, reorder) |
| 29 | [#265](https://github.com/KenSuenobu/rust-pushrod/issues/265) | extra-widgets: [5.2] Accordion (one section open at a time) |
| 30 | [#266](https://github.com/KenSuenobu/rust-pushrod/issues/266) | extra-widgets: [5.3] Sidebar (navigation list) |
| 31 | [#267](https://github.com/KenSuenobu/rust-pushrod/issues/267) | extra-widgets: [6.1] Input dialog (ask for one value) |
| 32 | [#268](https://github.com/KenSuenobu/rust-pushrod/issues/268) | extra-widgets: [6.2] Progress dialog |
| 33 | [#269](https://github.com/KenSuenobu/rust-pushrod/issues/269) | extra-widgets: [6.3] Task dialog (alert with details and options) |

### Extra widgets 3 - specialized views and assistants (18)

| Order | Issue | Title |
|---|---|---|
| 34 | [#270](https://github.com/KenSuenobu/rust-pushrod/issues/270) | extra-widgets: [2.7] Ruler |
| 35 | [#271](https://github.com/KenSuenobu/rust-pushrod/issues/271) | extra-widgets: [2.8] Shortcut field (key combination recorder) |
| 36 | [#272](https://github.com/KenSuenobu/rust-pushrod/issues/272) | extra-widgets: [3.7] Image well |
| 37 | [#273](https://github.com/KenSuenobu/rust-pushrod/issues/273) | extra-widgets: [3.8] New password (enter, confirm, strength) |
| 38 | [#274](https://github.com/KenSuenobu/rust-pushrod/issues/274) | extra-widgets: [3.9] Character picker |
| 39 | [#45](https://github.com/KenSuenobu/rust-pushrod/issues/45) | extra-widgets: [4.2] Column browser |
| 40 | [#275](https://github.com/KenSuenobu/rust-pushrod/issues/275) | extra-widgets: [4.3] Icon grid view |
| 41 | [#276](https://github.com/KenSuenobu/rust-pushrod/issues/276) | extra-widgets: [4.4] Data grid (editable table) |
| 42 | [#277](https://github.com/KenSuenobu/rust-pushrod/issues/277) | extra-widgets: [4.10] Filter field (live filter for lists, tables, and trees) |
| 43 | [#278](https://github.com/KenSuenobu/rust-pushrod/issues/278) | extra-widgets: [4.8] Dual list (list builder) |
| 44 | [#279](https://github.com/KenSuenobu/rust-pushrod/issues/279) | extra-widgets: [4.11] Rule editor (query builder) |
| 45 | [#280](https://github.com/KenSuenobu/rust-pushrod/issues/280) | extra-widgets: [4.12] Image viewer (zoom and pan) |
| 46 | [#281](https://github.com/KenSuenobu/rust-pushrod/issues/281) | extra-widgets: [5.4] Document tabs (closable and reorderable) |
| 47 | [#282](https://github.com/KenSuenobu/rust-pushrod/issues/282) | extra-widgets: [6.4] Wizard (step-by-step assistant) |
| 48 | [#283](https://github.com/KenSuenobu/rust-pushrod/issues/283) | extra-widgets: [6.5] Settings dialog |
| 49 | [#284](https://github.com/KenSuenobu/rust-pushrod/issues/284) | extra-widgets: [6.6] About dialog |
| 50 | [#285](https://github.com/KenSuenobu/rust-pushrod/issues/285) | extra-widgets: [6.7] Find and replace bar |
| 51 | [#286](https://github.com/KenSuenobu/rust-pushrod/issues/286) | extra-widgets: [6.8] Keyboard shortcuts editor |

## Chassis Examples

Small, complete applications built with Chassis, each with a mockup and a CI test, labeled help wanted. Roadmap: docs/ROADMAP_CHASSIS.md, Epic 8.

### Epics (1)

| Order | Issue | Title |
|---|---|---|
| 1 | [#207](https://github.com/KenSuenobu/rust-pushrod/issues/207) | chassis: [8] Epic: Example projects |

### Examples (8)

| Order | Issue | Title |
|---|---|---|
| 2 | [#208](https://github.com/KenSuenobu/rust-pushrod/issues/208) | chassis: [8.1] Hello, Chassis |
| 3 | [#209](https://github.com/KenSuenobu/rust-pushrod/issues/209) | chassis: [8.2] Temperature converter |
| 4 | [#210](https://github.com/KenSuenobu/rust-pushrod/issues/210) | chassis: [8.3] Tip calculator |
| 5 | [#211](https://github.com/KenSuenobu/rust-pushrod/issues/211) | chassis: [8.4] Settings dialog |
| 6 | [#212](https://github.com/KenSuenobu/rust-pushrod/issues/212) | chassis: [8.5] Image viewer |
| 7 | [#213](https://github.com/KenSuenobu/rust-pushrod/issues/213) | chassis: [8.6] Stopwatch |
| 8 | [#214](https://github.com/KenSuenobu/rust-pushrod/issues/214) | chassis: [8.7] Quiz |
| 9 | [#215](https://github.com/KenSuenobu/rust-pushrod/issues/215) | chassis: [8.8] Dice roller |
