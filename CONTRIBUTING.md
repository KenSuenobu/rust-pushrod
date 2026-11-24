# Contributing to Pushrod

Thanks for helping. Read [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md) first: Pushrod keeps
everything simple, flat, and predictable.

The plan of record is the roadmap in
[`docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md`](docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md);
each issue there describes the problem, scope, acceptance criteria, mockups, and the
documentation it needs. Each widget has its own roadmap in [`docs/widgets/`](docs/widgets/README.md),
which is the specification for that widget. To change how a widget looks or behaves, edit its
roadmap file (and its mockup in `docs/mockups/generate.py`) in a pull request, then update the issue.

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

## Where code goes

- **`pushrod` (core, `src/`):** engine, widget tree, events, `paint` helpers, caches, and shared
  widget behavior (flags, states, focus, clipping). No widgets except `RootWidget`.
- **`pushrod-widgets` (`pushrod-widgets/src/`):** the standard widgets. Use only the core's public
  API; if a widget needs something the core doesn't expose, add it to the core in its own change.

## Before opening a pull request

- `cargo fmt --all`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace` pass.
- Documentation changes ship in the same pull request as the code: the Docusaurus site in
  `pushrod-docs/` (each widget has its own page under **Widgets**), rustdoc, examples,
  screenshots, and the changelog. Check the site with `cd pushrod-docs && npm install &&
  npm run build` (broken links fail the build). Each issue's *Documentation (Docusaurus)*
  section names the pages to add or update.
- New widgets follow the existing drawing style (`paint::fill`, `paint::border`,
  `paint::text`) and draw everything through Pushrod; no OS-native controls, and no glass, shadows,
  bevels, gradients, or rounded corners ([DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md)).
- Any internal action you add emits an engine event (see roadmap issue 9.10).

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
