#!/usr/bin/env bash
#
# Builds the application

cargo build --workspace
cargo doc --workspace --no-deps

