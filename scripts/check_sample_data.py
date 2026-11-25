#!/usr/bin/env python3
"""Checks tracked files for real-looking personal data (see CONTRIBUTING.md, "Sample data and privacy").

Two rules are enforced:

1. E-mail addresses must use the reserved example.com, example.org, or example.net domains.
2. Home-directory paths must use the placeholder user name `user`
   (`/home/user/...`, `/Users/user/...`, `C:\\Users\\user\\...`), never a real account name.

Usage: python3 scripts/check_sample_data.py   (run from anywhere inside the repository)
Exit status: 0 when nothing is flagged, 1 when any line is flagged (each hit is printed).
"""

import re
import subprocess
import sys

# Files that legitimately contain "@" or are generated/binary.
SKIP_PATH = re.compile(
    r"^(Cargo\.toml|pushrod-widgets/Cargo\.toml|Cargo\.lock|LICENSE)$|package-lock\.json|\.(png|ttf)$"
)
EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
RESERVED_DOMAINS = {"example.com", "example.org", "example.net"}
# A home-directory prefix followed by a user name; the capture group is the name.
HOME_PATH = re.compile(r"(?:/home/|/Users/|[A-Za-z]:\\{1,2}Users\\{1,2})([^/\\\s\"'`<>)\]]+)", re.IGNORECASE)
# User names that are placeholders or shell/system variables, not real accounts.
ALLOWED_USERS = {"user", "$user", "${user}", "%username%", "<user>", "<username>", "username", "name", "you"}


def tracked_files():
    """Returns the repository's tracked file paths that should be scanned.

    Returns: a list of path strings relative to the repository root.
    """
    out = subprocess.run(["git", "ls-files"], check=True, capture_output=True, text=True).stdout
    return [p for p in out.splitlines() if not SKIP_PATH.search(p)]


def scan_line(line):
    """Returns the problems found in one line of text.

    Args: line - the text of a single line.
    Returns: a list of short descriptions (empty when the line is clean).
    """
    problems = []
    for match in EMAIL.finditer(line):
        if match.group(0).rsplit("@", 1)[1].lower() not in RESERVED_DOMAINS:
            problems.append(f"e-mail address {match.group(0)!r} (use name@example.com)")
    for match in HOME_PATH.finditer(line):
        if match.group(1).lower() not in ALLOWED_USERS:
            problems.append(f"home path with user name {match.group(1)!r} (use /home/user/...)")
    return problems


def main():
    """Scans every tracked text file and prints each flagged line.

    Returns: the process exit status (0 clean, 1 when anything was flagged).
    """
    failed = False
    for path in tracked_files():
        try:
            with open(path, encoding="utf-8") as handle:
                for number, line in enumerate(handle, 1):
                    for problem in scan_line(line):
                        print(f"{path}:{number}: {problem}")
                        failed = True
        except (UnicodeDecodeError, FileNotFoundError, IsADirectoryError):
            continue  # binary or removed file
    if failed:
        print("Use fictional placeholders only (see CONTRIBUTING.md, 'Sample data and privacy').")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
