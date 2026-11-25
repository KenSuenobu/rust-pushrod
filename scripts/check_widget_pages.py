#!/usr/bin/env python3
"""Checks that every widget module has a documentation page.

Every `<name>_widget.rs` in `pushrod-widgets/src/` needs `pushrod-docs/docs/widgets/<name>.md`
(`button_widget.rs` -> `button.md`).  Missing pages are listed and the exit status is 1.
Run from anywhere inside the repository:

    python3 scripts/check_widget_pages.py
"""

import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
MODULES = ROOT / "pushrod-widgets" / "src"
PAGES = ROOT / "pushrod-docs" / "docs" / "widgets"


def main():
    """Reports each widget module without a page; returns the process exit status."""
    modules = sorted(p.stem[: -len("_widget")] for p in MODULES.glob("*_widget.rs"))
    missing = [name for name in modules if not (PAGES / f"{name}.md").exists()]

    for name in missing:
        print(f"missing page: pushrod-docs/docs/widgets/{name}.md (for {name}_widget.rs)")

    print(f"{len(modules) - len(missing)} of {len(modules)} widget modules have a page")
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
