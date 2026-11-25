---
name: create-roadmap
description: Creates a ROADMAP file, its issues in GitHub, and notes the issue numbers, MVP flags, and descriptions in the ROADMAP.
---

# Implement (`/create-roadmap <description>`)

When user invokes **create-roadmap** with description, treat the description as the type of work to perform for the roadmap file.

## Guidelines

- Create `docs/ROADMAP_(description_in_snake_case_in_all_capital_letters).md`
- First section of Roadmap MUST contain the description that was provided to create the roadmap.
- Roadmap should include sections containing:
  - MVP Definition
  - Epics
  - Detailed Issue Descriptions per issue, grouped by Epic
  - Work to be done in order it must be done

## Phase 1: ROADMAP file

- Review description, use web to clarify detail for features
- Compare issues in GitHub that exist for this same feature while avoiding duplicate work
- Create document outlining features that should be created for this roadmap to be implemented completely
- Issue names are formatted: `<project>: [<epic number.issue number in epic>] <title of issue>`
- Identify issues that must be created, noting:
  - Problem Statement
  - Solution/Scope - indicate sources when used
  - Acceptance Criteria
  - Parallelism/Dependencies
  - Technical Stack
  - Documentation — the `ouroboros-docs` pages to create or update, named by section (User Guide / Administration / CLI) and path, and the screenshot ids to capture; "none — internal-only change" when nothing a person meets changes (`docs/CONVENTIONS.md` § 11). Every issue has this line.
  - Epic grouping
  - ASCII or Mermaid drawing for visual detail
- Each roadmap epic section must contain a table indicating:
  - Issue number
  - Title of issue
  - Single line summary of issue
  - Labels to use
  - Parallelism Indicator (Y/N)
  - MVP Indicator (Y/N)
  - Complexity
  - Affected Modules — list `ouroboros-docs` whenever the issue is user-visible (a UI route, label, flow or setting; a CLI flag; an `OURO_*` variable; an API behaviour), beside the module that changes
- Do not be shy about number of issues to create: be thorough and descriptive
- Do not create the issues, simply document them so the document can be validated before moving forward

