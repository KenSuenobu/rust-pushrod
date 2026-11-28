#!/usr/bin/env bash
#
# Runs Chassis.  Chassis has no window yet, so this runs the project-model example, which
# prints a project's forms, resources, and handlers.
#
# Usage: ./chassis.sh <project directory>

cargo run -p chassis --example project -- "$@"
