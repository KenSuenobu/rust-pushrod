#!/usr/bin/env python3
"""Checks that the published crates are ready to release (issue #73).

The crates `pushrod`, `pushrod-widgets`, and `pushrod-sdl2` are released together, so this
script checks that:

1. All three have the same version.
2. Every dependency between them asks for that same version.
3. Each one has the metadata crates.io and docs.rs need: description, license, repository,
   homepage (the documentation site), documentation, and a readme file that exists.
4. CHANGELOG.md has an `## [Unreleased]` section or, with `--tag vX.Y.Z`, an `## [X.Y.Z]`
   section, and the tag matches the crate version.

The crate data comes from `cargo metadata`, so no TOML parser is needed.

Usage:
    python3 scripts/check_release.py              (CI: every pull request)
    python3 scripts/check_release.py --tag v0.1.0 (the release workflow, before publishing)

Exit status: 0 when every check passes, 1 when any fails (each problem is printed).
"""

import argparse
import json
import os
import re
import subprocess
import sys

# The crates published to crates.io, in dependency order.
PUBLISHED = ["pushrod", "pushrod-widgets", "pushrod-sdl2"]
HOMEPAGE = "https://pushrod.nobudata.com"
REPOSITORY = "https://github.com/KenSuenobu/rust-pushrod"
REQUIRED_FIELDS = ["description", "license", "repository", "homepage", "documentation", "readme"]


def load_packages(root):
    """Returns the workspace packages from `cargo metadata`, keyed by crate name.

    Args:
        root: The repository root, where the workspace `Cargo.toml` is.

    Returns:
        A dict mapping each package name to its `cargo metadata` package object.
    """
    output = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return {package["name"]: package for package in json.loads(output)["packages"]}


def check_packages(packages):
    """Checks versions, inter-crate dependencies, and metadata of the published crates.

    Args:
        packages: The dict returned by `load_packages`.

    Returns:
        A tuple `(version, problems)`: the version of `pushrod` (or None when it is missing) and
        a list of problem descriptions, empty when everything is correct.
    """
    problems = []
    missing = [name for name in PUBLISHED if name not in packages]
    if missing:
        return None, [f"crate not in the workspace: {name}" for name in missing]

    version = packages["pushrod"]["version"]
    for name in PUBLISHED:
        package = packages[name]
        if package["version"] != version:
            problems.append(f"{name} is {package['version']}; pushrod is {version}")
        if package.get("publish") == []:
            problems.append(f"{name} has publish = false")
        for field in REQUIRED_FIELDS:
            if not package.get(field):
                problems.append(f"{name} has no `{field}`")
        if package.get("homepage") and package["homepage"] != HOMEPAGE:
            problems.append(f"{name} homepage is {package['homepage']}, not {HOMEPAGE}")
        if package.get("repository") and package["repository"] != REPOSITORY:
            problems.append(f"{name} repository is {package['repository']}, not {REPOSITORY}")
        if package.get("readme"):
            readme = os.path.join(os.path.dirname(package["manifest_path"]), package["readme"])
            if not os.path.isfile(readme):
                problems.append(f"{name} readme {package['readme']} does not exist")
        for dependency in package["dependencies"]:
            if dependency["name"] in PUBLISHED and dependency["req"] != f"^{version}":
                problems.append(
                    f"{name} depends on {dependency['name']} {dependency['req']}, not {version}"
                )
    return version, problems


def check_changelog(text, version, tag):
    """Checks that the changelog has the section the release needs.

    Args:
        text: The contents of CHANGELOG.md.
        version: The crate version from `check_packages`.
        tag: The release tag (such as `v0.1.0`), or None when not releasing.

    Returns:
        A list of problem descriptions, empty when the changelog is ready.
    """
    if tag is None:
        if not re.search(r"^## \[Unreleased\]", text, re.MULTILINE):
            return ["CHANGELOG.md has no `## [Unreleased]` section"]
        return []
    problems = []
    if tag != f"v{version}":
        problems.append(f"tag {tag} does not match the crate version v{version}")
    if not re.search(rf"^## \[{re.escape(version)}\]", text, re.MULTILINE):
        problems.append(f"CHANGELOG.md has no `## [{version}]` section")
    return problems


def main():
    """Runs every check and prints the problems.

    Returns:
        The exit status: 0 when every check passes, 1 otherwise.
    """
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--tag", help="the release tag, such as v0.1.0")
    args = parser.parse_args()

    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    version, problems = check_packages(load_packages(root))
    if version is not None:
        with open(os.path.join(root, "CHANGELOG.md"), encoding="utf-8") as changelog:
            problems += check_changelog(changelog.read(), version, args.tag)

    for problem in problems:
        print(f"release check: {problem}")
    if problems:
        return 1
    print(f"release check: {', '.join(PUBLISHED)} {version} ready")
    return 0


if __name__ == "__main__":
    sys.exit(main())
