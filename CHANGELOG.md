# Changelog

All notable changes to the Pushrod crates (`pushrod`, `pushrod-widgets`, and `pushrod-sdl2`) are
recorded here. The three crates are released together with matching versions. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

Each entry is one line: `- [#<issue>] <one-sentence summary> [<username>]`. `[0]` marks a change
with no issue. New entries go at the end of their section under **Unreleased**. When a release is
tagged, **Unreleased** becomes the version heading. See
[Changelog, Cargo metadata, and crates.io release](https://pushrod.nobudata.com/contributing/changelog-cargo-metadata-and-crates-io-release).

This changelog starts with the modernized core (pull request #10). Releases before that (up to
0.5.0) are not recorded here.

## [Unreleased]

### Added

- [#114] Specs in docs/specs and conformance tests in tests/conformance [kenji]
- [#112] IME: composition text, caret candidate window, composition events [kenji]
- [#76] Keyboard mnemonics: underlined letters, Alt+letter activation [kenji]
- [#60] WidgetRegistry: create widgets by type name; standard set registered [kenji]

### Changed

- [0] Publish to crates.io when a version bump is merged to main [kenji]

## [0.1.0] - 2026-11-26

### Added

- [#10] Modernized core: dyn Widget tree, WidgetCore, paint helpers, VB-style callbacks [kenji]
- [#93] Event vocabulary: modern Event enum replaces PushrodEvent [kenji]
- [#106] Backend-neutral Painter API with SDL2 backend and headless test painter [kenji]
- [#19] Widget behavior flags and visual states, disabled input routing [kenji]
- [#6] Event masks for the run loop and widgets [kenji]
- [#109] Engine events: EngineEvent, ENGINE mask, on_engine_event [kenji]
- [#124] Event-loop parity: next_event_with runs callbacks; same order as run [kenji]
- [#7] Broadcast window focus and resize events to every interested widget [kenji]
- [#22] Callback registry: on_press, on_toggle, on_hover and more, returning CallbackId [kenji]
- [#78] Custom events: ui.post, ctx.post, on_custom, thread-safe Sender [kenji]
- [#111] TextLayout trait, layout cache, and Parley engine behind a feature [kenji]
- [#3] GPU text: glyph atlas, fonts load once, WidgetCache owns FontCache [kenji]
- [#94] Event propagation: capture, target, bubble phases, Propagation::Stop [kenji]
- [#96] Input controllers: click, drag, scroll, hover, long-press, key [kenji]
- [#20] Keyboard focus: click, Tab, arrows, Return/Escape, FocusChanged [kenji]
- [#104] Per-state appearance: StateStyle and Appearance per visual state [kenji]
- [#289] Text truncation with an ellipsis: End, Start, Middle [kenji]
- [#290] Wrapped text: Word/Anywhere wrap, max_lines, height_for_width [kenji]
- [#21] Timers and idle callbacks: on_timer, on_idle, cancel_timer, Tick [kenji]
- [#138] Errors: pushrod::Error, Result APIs, Error engine event [kenji]
- [#137] Font handles, engine font registry, embedded default font [kenji]
- [#36] CustomPaint widget: closure-drawn, optional event closure [kenji]
- [#27] BoxedLabel widget: fill, centered text, border [kenji]
- [#83] Separator widget: horizontal/vertical line, optionally etched [kenji]
- [#41] GroupBox widget: titled frame that holds child widgets [kenji]
- [#33] ImageWidget: PNG/BMP with center/stretch/fit/tile, missing-file box [kenji]
- [#28] GlyphButton widget: one-character button, activate-on-press repeat [kenji]
- [#29] Button behaviors: default ring, cancel, toggle, press, dismiss [kenji]
- [#30] CheckBox widget: toggle by click or Space, indeterminate state [kenji]
- [#31] RadioButton and RadioGroup: exclusive checking, arrow-key selection [kenji]
- [#48] ProgressBar widget: determinate fill, busy animation, outline mode [kenji]
- [#40] Slider widget: drag, click, keys, step, ticks, value text; EventCtx::origin [kenji]
- [#32] TextField: caret, selection, clipboard, masks, validators [kenji]
- [#56] Children are clipped to their parent: cropped blits and hit tests [kenji]
- [#57] Composite widgets: named parts, add/insert/remove, part events [kenji]
- [#116] Layout trait, SizeHint, LayoutParams; measure/arrange per frame [kenji]
- [#117] Row and Column layouts with spacing, margins, stretch; Spacer [kenji]
- [#118] FormLayout: aligned labels, stretching fields, narrow stacking [kenji]
- [#39] ScrollBar: repeating arrows, track paging, proportional thumb [kenji]
- [#58] Container: scrollable viewport onto content with its own 0,0 [kenji]
- [#25] High-DPI rendering and window resize that re-runs the root layout [kenji]
- [#23] Tree editing at run time: remove, reparent, set_stacking_index [kenji]
- [#66] Modal dialogs: ui.open_modal, dimmed blurred layers, GEM border [kenji]
- [#35] Alert dialog: one-call ui.alert with restylable parts [kenji]

### Changed

- [#134] pushrod-sdl2 backend crate; core and widgets are SDL-free [kenji]
- [#136] Short widget constructors, preludes, non_exhaustive enums, Debug [kenji]
- [#73] Version 0.1.0 for all three crates; crates.io metadata, publish workflow [kenji]

### Fixed

- [0] macOS uses the OpenGL renderer, fixing the 1-pixel view shift on click [kenji]

### Examples, tests, and CI

- [#69] CI also runs on macOS and Windows; sample-data check flags home paths [kenji]
- [#71] Widget gallery example with --screenshot, used for the README [kenji]
- [#70] Golden-image tests: widgets rendered and compared with PNGs in CI [kenji]
- [#115] Headless performance benchmarks with stored baseline checked in CI [kenji]
- [#139] Bundled-SDL CI build on macOS/Windows; per-platform install guides [kenji]
- [#140] Reference app: contacts example, keyboard-only, golden images [kenji]

### Documentation and project

- [0] Added AGENTS.md file with rules for AI agents. [kenji]
- [#122] Docs site deploys to GitHub Pages with API reference; widget-page and sample checks [kenji]
- [0] Updated build workflow for pushrod-docs so it publishes from secrets data [kenji]
- [0] Additional updates to AGENTS.md to indicate any new behavior must be part of an executable test [kenji]
- [0] Roadmap for pushrod-extra-widgets: 44 widget issues with mockups; nine larger widgets moved out of the standard set [kenji]
- [0] Roadmap: text truncation and wrapping in the MVP, the Pushrod Showcase (Epic 12), and eight more extra widgets [kenji]
- [0] Roadmap: Chassis AI assistant through MCP (Epic 9, nine issues, v2) [kenji]
- [#72] Guide: writing widgets and composites, compiled samples per section [kenji]
