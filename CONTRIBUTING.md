# Contributing to Pushrod

Thanks for helping. The plan of record is the roadmap in
[`docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md`](docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md);
each issue there describes the problem, scope, acceptance criteria, mockups, and the
documentation it needs.

## Before opening a pull request

- `cargo fmt`, `cargo clippy --all-targets`, and `cargo test` pass.
- Documentation changes ship in the same pull request as the code: the Docusaurus site in
  `pushrod-docs/` (each widget has its own page under **Widgets**), rustdoc, examples,
  screenshots, and the changelog. Check the site with `cd pushrod-docs && npm install &&
  npm run build` (broken links fail the build). Each issue's *Documentation (Docusaurus)*
  section names the pages to add or update.
- New widgets follow the existing drawing style (`paint::fill`, `paint::border`,
  `paint::text`) and draw everything through Pushrod; no OS-native controls.
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
