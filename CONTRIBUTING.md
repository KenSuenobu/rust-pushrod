# Contributing to Pushrod

Thanks for helping. Read [DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md) first: Pushrod keeps
everything simple, flat, and predictable.

The plan of record is the roadmap in
[`docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md`](docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md);
each issue there describes the problem, scope, acceptance criteria, mockups, and the
documentation it needs. Each widget has its own roadmap in [`docs/widgets/`](docs/widgets/README.md),
which is the specification for that widget. To change how a widget looks or behaves, edit its
roadmap file (and its mockup in `docs/mockups/generate.py`) in a pull request, then update the issue.

## Where code goes

- **`pushrod` (core, `src/`):** engine, widget tree, events, `paint` helpers, caches, and shared
  widget behavior (flags, states, focus, clipping). No widgets except `RootWidget`.
- **`pushrod-widgets` (`pushrod-widgets/src/`):** the standard widgets. Use only the core's public
  API; if a widget needs something the core doesn't expose, add it to the core in its own change.

## Documentation is part of every contribution

Pushrod's documentation is part of the library, not an afterthought. **Every contribution that
changes what users see or do must include its documentation in the same pull request**, and pull
requests without it are not merged. There are no "docs to follow" pull requests.

**What has to be documented**

| Change | Documentation required |
|---|---|
| New widget | Its own page in `pushrod-docs/docs/widgets/`, following the widget page template |
| New or changed public API (types, functions, events, callbacks, features) | Rustdoc on every public item, and the guide or concept page on the Docusaurus site that covers it |
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

**The only exception** is a change with no user-visible effect: an internal refactor, a test,
or a CI change. Say so in the pull request: "No documentation changes: internal refactor of ...".

## Before opening a pull request

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace` pass on the current Rust release (Rust 1.100 or newer, edition 2024).
- Both crates use `#![warn(missing_docs)]`, and CI treats warnings as errors: every public item
  needs rustdoc. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` passes.
- The documentation site builds: `cd pushrod-docs && npm install && npm run build` (broken links
  fail the build).
- New widgets follow the existing drawing style (`paint::fill`, `paint::border`,
  `paint::text`) and draw everything through Pushrod; no OS-native controls, and no glass, shadows,
  bevels, gradients, or rounded corners ([DESIGN_PHILOSOPHY.md](DESIGN_PHILOSOPHY.md)).
- Any internal action you add emits an engine event (see roadmap issue 9.10).
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
