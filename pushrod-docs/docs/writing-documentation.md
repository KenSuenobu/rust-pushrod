---
sidebar_position: 6
title: Writing documentation
---

# Writing documentation

This site lives in `pushrod-docs/` and is built with [Docusaurus](https://docusaurus.io). Documentation changes ship in the same pull request as the code they describe.

## Running the site

```bash
cd pushrod-docs
npm install
npm start          # live preview at http://localhost:3000/rust-pushrod/
npm run build      # must pass: broken links fail the build
```

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

Code samples must compile against the current library. Prefer copying from an example in `examples/` or a test, and keep samples complete enough to paste.

## Sample data

Demos, examples, screenshots, and documentation use fictional placeholders only, never real names or personal information: `Jane Doe`, `jane.doe@example.com`, `Anytown`, `/home/user`. See [CONTRIBUTING.md](https://github.com/KenSuenobu/rust-pushrod/blob/main/CONTRIBUTING.md).
