---
sidebar_position: 1
title: CI
---

# CI

Every pull request, and every push to `main`, runs the workflow in
[`.github/workflows/ci.yml`](https://github.com/KenSuenobu/rust-pushrod/blob/main/.github/workflows/ci.yml).
A change cannot merge while any job fails. The badge at the top of the
[README](https://github.com/KenSuenobu/rust-pushrod/blob/main/README.md) shows the state of `main`.

## What runs

| Job | Where | What it checks |
|---|---|---|
| **Rust** | Ubuntu with Rust 1.100.0 (the minimum supported version) and latest stable; macOS and Windows with latest stable | `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `python3 scripts/check_docs.py` (doc headers), and `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` |
| **Documentation site** | Ubuntu | `npm ci && npm run build` in `pushrod-docs/`; broken links fail the build |
| **Golden-image rendering tests** | Ubuntu with Xvfb | `PUSHROD_GOLDEN=1 xvfb-run -a cargo test -p pushrod-sdl2 --test visual --test contacts` (every widget, and the [reference application](../guides/reference-application.md)'s screens); on failure the diff images are uploaded as the `golden-diff` artifact (see [Golden-image rendering tests](golden-image-rendering-tests.md)) |
| **Performance benchmarks** | Ubuntu | `python3 scripts/check_benchmarks.py`: a budget over its tolerance fails, a slow timing warns; the results are uploaded as the `benchmark-results` artifact (see [Performance benchmarks and budgets](performance-benchmarks-and-budgets.md)) |
| **Conformance** | the **Rust** job | `cargo test --workspace` includes `tests/conformance/`, which checks every guarantee in the [specifications](specs.md) |
| **Sample data** | Ubuntu | `python3 scripts/check_sample_data.py` |
| **Release readiness** | Ubuntu | `python3 scripts/test_check_release.py`, `python3 scripts/check_release.py` (matching versions, crates.io metadata, an `Unreleased` changelog section), and `cargo publish --workspace --dry-run` (see [Changelog, Cargo metadata, and crates.io release](changelog-cargo-metadata-and-crates-io-release.md)) |

Warnings fail the build: Clippy and rustdoc run with `-D warnings`, so fix every warning rather than silencing it.
SDL2, SDL2_ttf, and SDL2_image are installed with `apt` on Linux, Homebrew on macOS, and the
SDL development zips on Windows.

## Run the checks locally

From the repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 scripts/check_docs.py
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
python3 scripts/check_sample_data.py
python3 scripts/check_benchmarks.py
python3 scripts/check_release.py
cargo publish --workspace --dry-run
```

## The sample-data check

Examples, tests, mockups, and documentation use fictional data only (see
[CONTRIBUTING.md](https://github.com/KenSuenobu/rust-pushrod/blob/main/CONTRIBUTING.md#sample-data-and-privacy)).
`scripts/check_sample_data.py` scans every tracked text file and fails, printing the file and line, when it finds:

- an e-mail address outside the reserved `example.com`, `example.org`, and `example.net` domains, or
- a home-directory path (`/home/...`, `/Users/...`, `C:\Users\...`) whose user name is not the placeholder `user`.

Use `jane.doe@example.com` and `/home/user/notes.txt` instead.

## Keeping the minimum Rust version current

`rust-version` in the workspace `Cargo.toml` is the minimum supported Rust version, and CI tests it on
Linux. When a new stable Rust is released, raise `rust-version` and the `1.100.0` entry in the
workflow matrix together, in one pull request, so they never disagree.
