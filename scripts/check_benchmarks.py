#!/usr/bin/env python3
"""Runs the performance benchmarks and compares them with the stored baseline (issue #115).

The benchmarks are `pushrod-sdl2/benches/performance.rs` (timings, texture memory) plus the
release size of the gallery example.  The baseline is `benches/baseline.json`, one entry per
platform (`linux`, `macos`, `windows`), because timings and binary sizes differ between them.

Two kinds of number are checked:

* Budgets (`texture_bytes_per_widget`, `gallery_widgets`, `gallery_binary_bytes`) are stable
  from run to run, so growing past the tolerance FAILS the check.
* Timings vary with the machine, so getting slower than the timing tolerance only WARNS
  (a GitHub `::warning::` annotation).

Usage (from anywhere inside the repository):

    python3 scripts/check_benchmarks.py            # run and compare; exit 1 on a budget failure
    python3 scripts/check_benchmarks.py --update   # run and store this platform's baseline
    python3 scripts/check_benchmarks.py --report   # run and print the README table (Markdown)
    python3 scripts/check_benchmarks.py --from-json results.json   # use a saved result, skip the run

A platform with no stored baseline only warns; record one with `--update` (CI uploads its
results as an artifact for this).  Exit status: 0 when nothing failed, 1 on a budget failure.
"""

import argparse
import json
import platform
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "benches" / "baseline.json"

# Numbers that must not grow past the budget tolerance; the rest are timings.
BUDGETS = ("texture_bytes_per_widget", "gallery_widgets", "gallery_binary_bytes")
# Default tolerances: a budget may grow 10%; a timing may be 2.5x slower before it warns.
BUDGET_TOLERANCE = 0.10
TIMING_TOLERANCE = 1.5
# Human-readable names and units for the report table.
LABELS = {
    "event_dispatch_ns": ("Event dispatch (pointer event through the 148-widget gallery)", "ns/event"),
    "frame_interactive_us": ("Gallery frame under interaction (one slider changes, one click)", "µs/frame"),
    "frame_full_us": ("Gallery full redraw (every widget repainted and blitted)", "µs/frame"),
    "text_atlas_warmup_us": ("Text: first layout (font load and glyph atlas warm-up)", "µs"),
    "text_layout_cached_ns": ("Text: repeat layout of the same text", "ns"),
    "form_load_us": ("Form load (40 label and field rows, first frame)", "µs"),
    "texture_bytes_per_widget": ("Texture memory per widget (gallery average)", "bytes"),
    "gallery_widgets": ("Gallery widgets", "count"),
    "gallery_binary_bytes": ("Gallery example binary (release)", "bytes"),
}


def system_key():
    """Returns this platform's key in the baseline file: `linux`, `macos`, or `windows`."""
    return {"Darwin": "macos", "Windows": "windows"}.get(platform.system(), "linux")


def run_benchmarks():
    """Runs the benchmark binary and measures the gallery's release size.

    Returns a dict of metric name to number.
    """
    bench = subprocess.run(
        ["cargo", "bench", "-p", "pushrod-sdl2", "--bench", "performance"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    )
    result = json.loads(bench.stdout.strip().splitlines()[-1])

    subprocess.run(
        ["cargo", "build", "--release", "-p", "pushrod-sdl2", "--example", "gallery"],
        cwd=ROOT, check=True,
    )
    exe = ROOT / "target" / "release" / "examples" / ("gallery.exe" if platform.system() == "Windows" else "gallery")
    result["gallery_binary_bytes"] = float(exe.stat().st_size)
    return result


def compare(result, entry):
    """Compares `result` with a baseline `entry`.

    Returns (failures, warnings), each a list of message strings.
    """
    failures, warnings = [], []
    budget_tol = entry.get("budget_tolerance", BUDGET_TOLERANCE)
    timing_tol = entry.get("timing_tolerance", TIMING_TOLERANCE)

    for name, base in entry["numbers"].items():
        value = result.get(name)
        if value is None or base <= 0:
            continue
        change = value / base - 1.0
        text = f"{name}: {value:.1f} vs baseline {base:.1f} ({change:+.0%})"
        if name in BUDGETS and change > budget_tol:
            failures.append(f"{text}, over the {budget_tol:.0%} budget tolerance")
        elif name not in BUDGETS and change > timing_tol:
            warnings.append(f"{text}, slower than the {timing_tol:.0%} timing tolerance")
    return failures, warnings


def report(result, entry):
    """Prints the README table for `result` (with the baseline column when `entry` is given)."""
    print("| Benchmark | Current | Unit |")
    print("|---|---:|---|")
    for name, (label, unit) in LABELS.items():
        if name in result:
            print(f"| {label} | {result[name]:,.0f} | {unit} |")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--update", action="store_true", help="store this platform's baseline")
    parser.add_argument("--report", action="store_true", help="print the README table")
    parser.add_argument("--from-json", help="use a saved result instead of running")
    parser.add_argument("--output", help="also save the result JSON here")
    args = parser.parse_args()

    result = json.load(open(args.from_json)) if args.from_json else run_benchmarks()
    key = system_key()

    if args.output:
        Path(args.output).write_text(json.dumps(result, indent=2) + "\n")
    for name, value in result.items():
        print(f"{name:>26}  {value:>16,.1f}")

    baseline = json.loads(BASELINE.read_text()) if BASELINE.exists() else {"platforms": {}}

    if args.update:
        baseline["platforms"][key] = {
            "budget_tolerance": BUDGET_TOLERANCE,
            "timing_tolerance": TIMING_TOLERANCE,
            "numbers": {k: round(v, 1) for k, v in result.items()},
        }
        BASELINE.write_text(json.dumps(baseline, indent=2) + "\n")
        print(f"Stored the {key} baseline in {BASELINE.relative_to(ROOT)}")
        return 0

    if args.report:
        report(result, baseline["platforms"].get(key))
        return 0

    entry = baseline["platforms"].get(key)
    if entry is None:
        print(f"::warning::No stored benchmark baseline for {key}; run with --update and commit benches/baseline.json")
        return 0

    failures, warnings = compare(result, entry)
    for message in warnings:
        print(f"::warning::Benchmark slower than baseline - {message}")
    for message in failures:
        print(f"::error::Benchmark budget exceeded - {message}")
    if not failures and not warnings:
        print("All benchmarks are within tolerance of the baseline.")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
