#!/usr/bin/env python3
"""Creates or updates the GitHub projects described in scripts/github-projects.json.

Each project is linked to the repository and gets two fields: **Phase** (single select, one option
per phase in the JSON) and **Order** (number, the work order). Every issue is added and its fields
are set. Re-running is safe: projects are matched by title, existing items are reused, and fields
are overwritten with the values from the JSON.

Requirements: the GitHub CLI (`gh`) signed in with the `project` scope:

    gh auth refresh -s project

Usage (from the repository root):

    python3 scripts/create_github_projects.py --dry-run
    python3 scripts/create_github_projects.py
    python3 scripts/create_github_projects.py --only "Chassis MVP"
"""

import argparse
import json
import pathlib
import subprocess
import sys

DEFINITIONS = pathlib.Path(__file__).with_name("github-projects.json")


class Gh:
    """Runs `gh` commands, or prints them instead when dry-running."""

    def __init__(self, dry_run):
        """Creates the runner.

        # Arguments

        * `dry_run` - When true, commands are printed and not executed.
        """
        self.dry_run = dry_run

    def run(self, args, parse_json=True):
        """Runs `gh` with the given arguments.

        # Arguments

        * `args` - Arguments after `gh`, as a list of strings.
        * `parse_json` - Whether to parse standard output as JSON.

        # Returns

        The parsed JSON (or raw text when `parse_json` is false); `None` when dry-running.

        # Errors

        Exits the program with the command's error output if `gh` fails.
        """
        if self.dry_run:
            print("gh " + " ".join(f'"{a}"' if " " in a else a for a in args))
            return None
        result = subprocess.run(["gh"] + args, capture_output=True, text=True)
        if result.returncode != 0:
            sys.exit(f"gh {' '.join(args)} failed:\n{result.stderr.strip()}")
        if not parse_json:
            return result.stdout
        return json.loads(result.stdout) if result.stdout.strip() else {}


def find_project(gh, owner, title):
    """Finds one of the owner's projects by title.

    # Arguments

    * `gh` - The command runner.
    * `owner` - The account that owns the projects.
    * `title` - The project title to look for.

    # Returns

    The project as returned by `gh project list` (with `number` and `id`), or `None`.
    """
    listing = gh.run(["project", "list", "--owner", owner, "--limit", "200", "--format", "json"])
    if not listing:
        return None
    return next((p for p in listing.get("projects", []) if p["title"] == title), None)


def ensure_project(gh, owner, repo, spec):
    """Creates the project if it doesn't exist, then sets its description and readme and links it.

    # Arguments

    * `gh` - The command runner.
    * `owner` - The account that owns the project.
    * `repo` - The repository to link the project to (`owner/name`).
    * `spec` - The project's definition from the JSON file.

    # Returns

    The project's number and node ID as `(number, id)`; placeholders when dry-running.
    """
    project = find_project(gh, owner, spec["title"])
    if project is None:
        project = gh.run(["project", "create", "--owner", owner, "--title", spec["title"], "--format", "json"])
        if project is None:
            return ("<number>", "<id>")
    number = str(project["number"])
    gh.run(["project", "edit", number, "--owner", owner, "--description", spec["description"],
            "--readme", spec["readme"], "--format", "json"])
    gh.run(["project", "link", number, "--owner", owner, "--repo", repo], parse_json=False)
    return (number, project["id"])


def ensure_fields(gh, owner, number, phases):
    """Creates the Phase and Order fields when missing.

    # Arguments

    * `gh` - The command runner.
    * `owner` - The account that owns the project.
    * `number` - The project number.
    * `phases` - The Phase options, in order.

    # Returns

    `(phase_field_id, {phase name: option id}, order_field_id)`; empty values when dry-running.
    """
    def field_list():
        return gh.run(["project", "field-list", number, "--owner", owner, "--format", "json", "--limit", "100"]) or {}

    fields = {f["name"]: f for f in field_list().get("fields", [])}
    if "Phase" not in fields:
        gh.run(["project", "field-create", number, "--owner", owner, "--name", "Phase",
                "--data-type", "SINGLE_SELECT", "--single-select-options", ",".join(phases), "--format", "json"])
    if "Order" not in fields:
        gh.run(["project", "field-create", number, "--owner", owner, "--name", "Order",
                "--data-type", "NUMBER", "--format", "json"])
    if gh.dry_run:
        return ("<phase-field>", {p: f"<option {p}>" for p in phases}, "<order-field>")
    fields = {f["name"]: f for f in field_list().get("fields", [])}
    phase = fields["Phase"]
    options = {o["name"]: o["id"] for o in phase.get("options", [])}
    missing = [p for p in phases if p not in options]
    if missing:
        sys.exit(f"Project {number}: Phase field lacks options {missing}; add them in the project settings and re-run.")
    return (phase["id"], options, fields["Order"]["id"])


def add_items(gh, owner, number, project_id, items, phase_field, options, order_field):
    """Adds each issue to the project and sets its Phase and Order.

    # Arguments

    * `gh` - The command runner.
    * `owner` - The account that owns the project.
    * `number` - The project number.
    * `project_id` - The project's node ID.
    * `items` - The issues to add, from the JSON file.
    * `phase_field` - The Phase field's ID.
    * `options` - Phase option IDs by name.
    * `order_field` - The Order field's ID.
    """
    for item in items:
        added = gh.run(["project", "item-add", number, "--owner", owner, "--url", item["url"], "--format", "json"])
        item_id = added["id"] if added else "<item>"
        gh.run(["project", "item-edit", "--id", item_id, "--project-id", project_id,
                "--field-id", phase_field, "--single-select-option-id", options[item["phase"]], "--format", "json"])
        gh.run(["project", "item-edit", "--id", item_id, "--project-id", project_id,
                "--field-id", order_field, "--number", str(item["order"]), "--format", "json"])
        if not gh.dry_run:
            print(f"  {item['order']:>3}  #{item['number']:<4} {item['phase']}")


def main():
    """Reads the definitions and creates or updates every project (or the one named with --only).

    # Returns

    The process exit code.
    """
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dry-run", action="store_true", help="print the gh commands instead of running them")
    parser.add_argument("--only", help="create or update only the project with this title")
    args = parser.parse_args()

    definitions = json.loads(DEFINITIONS.read_text())
    gh = Gh(args.dry_run)
    for spec in definitions["projects"]:
        if args.only and spec["title"] != args.only:
            continue
        print(f"== {spec['title']} ({len(spec['items'])} items)")
        number, project_id = ensure_project(gh, definitions["owner"], definitions["repo"], spec)
        phase_field, options, order_field = ensure_fields(gh, definitions["owner"], number, spec["phases"])
        add_items(gh, definitions["owner"], number, project_id, spec["items"], phase_field, options, order_field)
    return 0


if __name__ == "__main__":
    sys.exit(main())
