## What this changes

<!-- One or two sentences, and the roadmap issue it closes (for example "Closes #29"). -->

## Documentation

<!-- Every user-visible change ships with its documentation. See CONTRIBUTING.md. -->

- [ ] Docusaurus pages added or updated (list them):
- [ ] Every existing page this change makes wrong is fixed (I searched the site for the names I changed)
- [ ] Full doc header (summary, `# Arguments`, `# Returns`, `# Errors`/`# Panics`) on every new or changed non-private function and type
- [ ] Screenshots updated if anything looks different
- [ ] Code samples compile
- [ ] `CHANGELOG.md` entry under `Unreleased`
- [ ] Fictional sample data only (no real names or personal information)
- [ ] Or: **No documentation changes**, because:

## Checks

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `python3 scripts/check_docs.py`
- [ ] `cd pushrod-docs && npm run build`
