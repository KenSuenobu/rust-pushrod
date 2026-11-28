# Contributing to Pushrod

Thanks for helping. Read [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md) first: Pushrod keeps
everything simple, flat, and predictable.

The plan of record is the roadmap in
[`docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md`](docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md);
each issue there describes the problem, scope, acceptance criteria, mockups, and the
documentation it needs. Each widget has its own roadmap in [`docs/widgets/`](docs/widgets/README.md),
which is the specification for that widget. To change how a widget looks or behaves, edit its
roadmap file (and its mockup in `docs/mockups/generate.py`) in a pull request, then update the issue.
Components built for the Chassis designer (label `chassis`) are specified the same way in
[`docs/chassis/`](docs/chassis/code-editor.md).

## Where code goes

- **`pushrod` (core, `src/`):** engine, widget tree, events, `paint` helpers, caches, and shared
  widget behavior (flags, states, focus, clipping). No widgets except `RootWidget`.
- **`pushrod-widgets` (`pushrod-widgets/src/`):** the standard widgets. Use only the core's public
  API; if a widget needs something the core doesn't expose, add it to the core in its own change.
- **`pushrod-extra-widgets` (planned, `pushrod-extra-widgets/src/`):** larger and compound widgets
  (pickers, tree view, property grid, panels, dialogs). Same flat look; public APIs of `pushrod` and
  `pushrod-widgets` only. A widget that is not one control with one job belongs here, not in the
  standard set ([roadmap](docs/ROADMAP_EXTRA_WIDGETS.md)).
- **`pushrod-showcase` (`pushrod-showcase/`):** the interactive demo of every widget and core
  feature. **Every pull request that adds a widget, standard or extra, adds its Showcase page** (list the widget's registered type names in `Page::widget_types`); the
  completeness test fails when a registered widget has none ([roadmap, Epic 12](docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md)).

## Documentation is part of every contribution

Pushrod's documentation is part of the library, not an afterthought. **Every contribution that
changes what users see or do must include its documentation in the same pull request**, and pull
requests without it are not merged. There are no "docs to follow" pull requests.

**What has to be documented**

| Change | Documentation required |
|---|---|
| New widget | Its own page in `pushrod-docs/docs/widgets/`, following the widget page template |
| New or changed API (types, functions, events, callbacks, features) | A full doc header on every non-private item (see [Code documentation](#code-documentation)), and the guide or concept page on the Docusaurus site that covers it |
| Changed behavior or appearance | Every page that describes it, including screenshots |
| New example | A link from the page it illustrates |
| Removed or renamed feature | Its documentation removed or renamed everywhere it appears |
| Anything user-visible | An entry under `Unreleased` in `CHANGELOG.md` |

Each roadmap issue's *Documentation (Docusaurus)* section names the pages to add or update.

**Documentation must be clean**

- Plain language and short sentences. Write for someone using Pushrod for the first time.
- Pages follow the templates and conventions on the site's
  [Writing documentation](pushrod-docs/docs/writing-documentation.md) page.
- Code samples compile against the current library and are complete enough to paste.
- No `TODO`s, placeholders, dead links, or screenshots of an older look.
- Fictional sample data only (see below).

**Documentation must be up to date**

The documentation describes the code as it is after your pull request merges. If your change
makes *any* existing page wrong, anywhere on the site and not only the page you're editing, fix
it in the same pull request. Search the site and the rustdoc for every name you added, changed,
or removed.

**Documentation must be maintained**

Outdated or wrong documentation is a bug, with the same priority as a bug in the code. If you
find one, fix it (documentation-only pull requests are always welcome) or open an issue with the
`documentation` label. Reviewers check documentation as carefully as code.

**The only exception** to the Docusaurus requirement is a change with no user-visible effect: an
internal refactor, a test, or a CI change. Say so in the pull request: "No documentation
changes: internal refactor of ...". Code doc headers are never optional: every non-private item
you add or change is documented in the code, user-visible or not.

## Code documentation

Every function, method, struct, enum, trait, type alias, and constant that isn't private is
fully documented in the code, in the same pull request that adds or changes it. That includes
`pub(crate)` items, every function declared in a trait, and every method in a trait
implementation. Only private items (no `pub`) may go without a header, though a short comment
is still welcome.

A function's header has a one-line summary, then whatever detail a caller needs, then these
sections, in this order, whenever they apply:

```rust
/// Adds a `Widget` as a child of another `Widget`.
///
/// The `Widget`'s origin is relative to its parent's origin, and it is drawn on top of the
/// parent's existing children.
///
/// # Arguments
///
/// * `parent` - The ID of the parent `Widget`.
/// * `widget` - The `Widget` to add.
///
/// # Returns
///
/// The ID assigned to the `Widget`.
///
/// # Errors
///
/// (For functions returning `Result`) when each error is returned.
///
/// # Panics
///
/// Panics if `parent` does not exist.
///
/// # Examples
///
/// (For the main entry points) a short example that compiles.
pub fn add_child(&mut self, parent: WidgetId, widget: Box<dyn Widget>) -> WidgetId {
```

- **`# Arguments`:** every parameter except `self`, as ``* `name` - description``.
- **`# Returns`:** whenever the function returns a value.
- **`# Errors`** and **`# Panics`:** whenever the function can fail or panic.
- **Types:** what the type is for, and how to create and use it. Public fields and enum
  variants get their own `///` line.

This is checked automatically: both crates enable `missing_docs`,
`clippy::missing_panics_doc`, and `clippy::missing_errors_doc`, and `python3
scripts/check_docs.py` checks the rest (headers on `pub(crate)` items and trait methods, every
parameter under `# Arguments`, and `# Returns`). Run it before opening a pull request.

## Before opening a pull request

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace` pass on the current Rust release (Rust 1.100 or newer, edition 2024).
- Every non-private item has a full doc header, and `python3 scripts/check_docs.py` passes. Both
  crates enable `missing_docs`, `clippy::missing_panics_doc`, and `clippy::missing_errors_doc`,
  and CI treats warnings as errors. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
  passes.
- The documentation site builds: `cd pushrod-docs && npm install && npm run build` (broken links
  fail the build).
- New widgets follow the existing drawing style (`paint::fill`, `paint::border`,
  `paint::text`) and draw everything through Pushrod; no OS-native controls, and no glass, shadows,
  bevels, gradients, or rounded corners ([DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md)).
- Any internal action you add emits an engine event: add its `EngineEvent` variant, emit it after the action completes, and test it (see [Engine events](pushrod-docs/docs/events/engine-events.md)).
- Public API changes follow semantic versioning: public enums are `#[non_exhaustive]`, public types implement `Debug`, and constructors take few required arguments (chain setters for the rest). After the first release, `cargo-semver-checks` runs on pull requests in CI.
- The pull request template's checklist is filled in.

## Sample data and privacy

Demos, examples, tests, mockups, screenshots, documentation, and issue text must not
contain real names or personal information (PII). This includes people's names,
e-mail addresses, phone numbers, street addresses and real home towns, account names,
home-directory paths that contain a user name, and the names of private projects.

Use clearly fictional placeholders instead:

| Kind of data | Use |
|---|---|
| Person | `Jane Doe`, `John Doe`, `Customer` |
| E-mail | `jane.doe@example.com` (`example.com`, `example.org`, and `example.net` are reserved for documentation) |
| Place | `Anytown`, `Springfield` |
| Phone | `555-0100` to `555-0199` |
| Path | `/home/user/...`, `C:\Users\user\...` |
| Files and projects | `report.txt`, `notes-app`, `sample-game` |
| Host names | `example.com`, `server.example.com` |

The mockup generator (`docs/mockups/generate.py`) follows the same rule. CI flags e-mail
addresses outside the reserved example domains.
