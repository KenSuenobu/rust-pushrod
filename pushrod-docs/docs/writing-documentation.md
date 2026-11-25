---
sidebar_position: 6
title: Writing documentation
---

# Writing documentation

This site lives in `pushrod-docs/` and is built with [Docusaurus](https://docusaurus.io).

:::important[Documentation is required]
Every contribution that changes what users see or do includes its documentation in the same pull request: the pages on this site, rustdoc for every public item, screenshots, and a changelog entry. Pull requests without it are not merged. Documentation must be **clean** (plain language, working samples, no placeholders), **up to date** (if your change makes any page wrong, fix that page too), and **maintained** (wrong documentation is a bug). The full rules are in [CONTRIBUTING.md](https://github.com/KenSuenobu/rust-pushrod/blob/main/CONTRIBUTING.md#documentation-is-part-of-every-contribution).
:::

## Running the site

```bash
cd pushrod-docs
npm install
npm start          # live preview at http://localhost:3003/
npm run build      # must pass: broken links fail the build
```

## How CI checks the documentation

Two workflows keep the site honest. Both run on every pull request:

| Check | What fails it | Run it yourself |
|---|---|---|
| **Site build** (`docs.yml`) | A broken link or anchor, or a page that does not build | `cd pushrod-docs && npm run build` |
| **API reference** (`docs.yml`) | Any rustdoc warning | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` |
| **Widget pages** (`ci.yml`) | A `pushrod-widgets/src/<name>_widget.rs` with no `docs/widgets/<name>.md` (the missing pages are listed) | `python3 scripts/check_widget_pages.py` |
| **Checked samples** (`ci.yml`) | A `rust checked` code sample that no longer compiles | `cargo test -p pushrod-doc-samples` |

When a change lands on `main`, `docs.yml` builds the site and the rustdoc API reference (served at `/api/`, the **API reference** link in the navbar) into the `pushrod-docs` Docker image (an nginx image serving the site at `/`) and pushes it as `<DOCKER_REPOSITORY>/pushrod-docs` with the tags `latest` and the commit SHA. Pull requests build the site but do not publish. The repository secrets `DOCKER_REPOSITORY`, `DOCKER_USERNAME`, and `DOCKER_PASSWORD` must be set.

## Where pages go

| Content | Location | Sidebar group |
|---|---|---|
| A widget | `docs/widgets/<widget>.md`, titled with the widget's plain name (`Button`, `Check Box`, `Scroll Bar`) | Widgets |
| A layout manager | `docs/layouts/<layout>.md` | Layouts |
| How-to guides | `docs/guides/` | Guides |
| How things work | `docs/concepts/` | Concepts |
| Events and callbacks | `docs/events/` | Events |
| Forms and form files | `docs/forms/` | Forms |
| Backends and platforms | `docs/platforms/` | Platforms |

Create a folder's `_category_.json` (label and position) when you add the first page to it.

## Widget page template

Every widget page uses the same sections, in this order (see [Button](./widgets/button.md)):

1. One-line summary naming the type and module, and a screenshot or mockup.
2. **Creating it**: a code sample that compiles, and a parameter table.
3. **Methods**: the widget's own methods.
4. **Common methods**: link to the shared `Widget` methods.
5. **Events and callbacks**: every event the widget emits and the callback that receives it.
6. **Notes**: behavior worth knowing (how it reacts to the mouse and keyboard, how it draws).
7. **Planned**: links to roadmap issues that will change it.

## Screenshots and mockups

- Screenshots go in `static/img/screenshots/` and are referenced as `/img/screenshots/<name>.png`. Take them from a running example (for instance under Xvfb) so they show what Pushrod really draws.
- Until a widget exists, use its roadmap mockup from `docs/mockups/` in the repository.

## Code samples

Code samples must compile against the current library. Mark a complete sample as checked by writing `rust checked` on its opening fence (for example ```` ```rust checked title="src/main.rs" ````). The `pushrod-doc-samples` crate compiles every checked sample, and CI fails when one breaks. A checked sample must be a whole program or module: its own `use` lines and no code outside functions. Fragments that rely on an `engine` defined elsewhere stay plain `rust`. Prefer copying from an example (`pushrod-widgets/examples/`) or a test, and keep samples complete enough to paste. Import standard widgets from `pushrod_widgets` and everything else from `pushrod`.

## Documenting the code

The site and the code are documented together. Every non-private function, method, and type in
the library has a full rustdoc header: a summary, then `# Arguments` (every parameter),
`# Returns`, `# Errors`, `# Panics`, and `# Examples` where they apply. The format and the
checks that enforce it are in [CONTRIBUTING.md](https://github.com/KenSuenobu/rust-pushrod/blob/main/CONTRIBUTING.md#code-documentation).

## Keeping documentation current

- When you change a name, search the whole site (`grep -r` in `pushrod-docs/docs/`) and the rustdoc for it, and update every mention.
- When something looks different, retake its screenshots.
- When you remove a feature, remove its documentation and any links to it.
- Found something wrong or out of date? Fix it, or open an issue with the `documentation` label. Documentation-only pull requests are welcome.

## Sample data

Demos, examples, screenshots, and documentation use fictional placeholders only, never real names or personal information: `Jane Doe`, `jane.doe@example.com`, `Anytown`, `/home/user`. See [CONTRIBUTING.md](https://github.com/KenSuenobu/rust-pushrod/blob/main/CONTRIBUTING.md).
