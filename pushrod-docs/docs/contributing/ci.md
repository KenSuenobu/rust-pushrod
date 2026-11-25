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
| **Sample data** | Ubuntu | `python3 scripts/check_sample_data.py` |

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
