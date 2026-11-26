#!/usr/bin/env python3
"""Unit tests for scripts/check_release.py.

Usage: python3 scripts/test_check_release.py
"""

import copy
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import check_release  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def package(name, version="0.1.0", dependencies=()):
    """Builds a minimal `cargo metadata` package object with correct metadata.

    Args:
        name: The crate name.
        version: The crate version.
        dependencies: `(name, req)` pairs for its dependencies.

    Returns:
        A dict shaped like one entry of `cargo metadata`'s `packages` list.
    """
    directory = ROOT if name == "pushrod" else os.path.join(ROOT, name)
    return {
        "name": name,
        "version": version,
        "publish": None,
        "description": f"{name} for tests",
        "license": "Apache-2.0",
        "repository": check_release.REPOSITORY,
        "homepage": check_release.HOMEPAGE,
        "documentation": f"https://docs.rs/{name}",
        "readme": "README.md",
        "manifest_path": os.path.join(directory, "Cargo.toml"),
        "dependencies": [{"name": dep, "req": req} for dep, req in dependencies],
    }


def good_packages():
    """Returns a correct set of the three published crates at version 0.1.0."""
    return {
        "pushrod": package("pushrod"),
        "pushrod-widgets": package("pushrod-widgets", dependencies=[("pushrod", "^0.1.0")]),
        "pushrod-sdl2": package(
            "pushrod-sdl2",
            dependencies=[("pushrod", "^0.1.0"), ("pushrod-widgets", "^0.1.0"), ("sdl2", "^0.38")],
        ),
    }


class CheckPackagesTest(unittest.TestCase):
    def test_correct_packages_pass(self):
        self.assertEqual(check_release.check_packages(good_packages()), ("0.1.0", []))

    def test_mismatched_version_fails(self):
        packages = good_packages()
        packages["pushrod-sdl2"]["version"] = "0.1.1"
        _, problems = check_release.check_packages(packages)
        self.assertIn("pushrod-sdl2 is 0.1.1; pushrod is 0.1.0", problems)

    def test_stale_dependency_version_fails(self):
        packages = good_packages()
        packages["pushrod-widgets"]["dependencies"][0]["req"] = "^0.0.41"
        _, problems = check_release.check_packages(packages)
        self.assertIn("pushrod-widgets depends on pushrod ^0.0.41, not 0.1.0", problems)

    def test_missing_metadata_fails(self):
        packages = good_packages()
        packages["pushrod"]["homepage"] = None
        _, problems = check_release.check_packages(packages)
        self.assertIn("pushrod has no `homepage`", problems)

    def test_wrong_homepage_fails(self):
        packages = good_packages()
        packages["pushrod"]["homepage"] = "https://example.com"
        _, problems = check_release.check_packages(packages)
        self.assertEqual(len(problems), 1)

    def test_missing_readme_file_fails(self):
        packages = good_packages()
        packages["pushrod-sdl2"]["readme"] = "MISSING.md"
        _, problems = check_release.check_packages(packages)
        self.assertIn("pushrod-sdl2 readme MISSING.md does not exist", problems)

    def test_unpublishable_crate_fails(self):
        packages = good_packages()
        packages["pushrod-widgets"]["publish"] = []
        _, problems = check_release.check_packages(packages)
        self.assertIn("pushrod-widgets has publish = false", problems)

    def test_missing_crate_fails(self):
        packages = good_packages()
        del packages["pushrod-sdl2"]
        self.assertEqual(
            check_release.check_packages(packages),
            (None, ["crate not in the workspace: pushrod-sdl2"]),
        )

    def test_workspace_passes(self):
        packages = check_release.load_packages(ROOT)
        self.assertEqual(check_release.check_packages(copy.deepcopy(packages))[1], [])


class CheckChangelogTest(unittest.TestCase):
    UNRELEASED = "# Changelog\n\n## [Unreleased]\n\n- [#73] Release [user]\n"
    RELEASED = "# Changelog\n\n## [Unreleased]\n\n## [0.1.0] - 2026-01-01\n\n- [#73] Release [user]\n"

    def test_unreleased_section_passes_without_tag(self):
        self.assertEqual(check_release.check_changelog(self.UNRELEASED, "0.1.0", None), [])

    def test_no_unreleased_section_fails_without_tag(self):
        self.assertEqual(len(check_release.check_changelog("# Changelog\n", "0.1.0", None)), 1)

    def test_tag_with_version_section_passes(self):
        self.assertEqual(check_release.check_changelog(self.RELEASED, "0.1.0", "v0.1.0"), [])

    def test_tag_without_version_section_fails(self):
        problems = check_release.check_changelog(self.UNRELEASED, "0.1.0", "v0.1.0")
        self.assertEqual(problems, ["CHANGELOG.md has no `## [0.1.0]` section"])

    def test_tag_that_does_not_match_version_fails(self):
        problems = check_release.check_changelog(self.RELEASED, "0.1.0", "v0.2.0")
        self.assertIn("tag v0.2.0 does not match the crate version v0.1.0", problems)

    def test_repository_changelog_passes(self):
        with open(os.path.join(ROOT, "CHANGELOG.md"), encoding="utf-8") as changelog:
            self.assertEqual(check_release.check_changelog(changelog.read(), "0.1.0", None), [])


if __name__ == "__main__":
    unittest.main()
