---
name: code-review
description: >-
  Review a GitHub pull request by number, analyze all changed files, and post
  review comments with specific suggested diffs and reasoning. Use when the
  user asks to review a PR, check PR changes, or invokes code-review with a
  PR number.
disable-model-invocation: true
---

# Code Review (`/code-review <pr-number>`)

When user invokes **code-review** with a PR number, fetch the pull request from the current repository, analyze every changed file, and post review comments with actionable feedback and concrete diffs.

## Guidelines

- Act as a senior software engineer.
- Apply **AGENTS.md** rules as the quality baseline.
- Review only what the PR changes — do not critique unrelated code.
- Never approve or merge; only comment.
- If `gh` is not available or auth fails, stop and explain.

## Phase 1: Fetch PR metadata

Identify the current repository from workspace context and fetch the PR:

```bash
gh pr view <pr-number> --repo <owner>/<repo> --json title,body,baseRefName,headRefName,author,labels,files
```

Summarize title, author, and purpose so full intent is in context.

## Phase 2: Fetch the diff

Get the full diff for the PR:

```bash
gh pr diff <pr-number> --repo <owner>/<repo>
```

Also list changed files:

```bash
gh pr view <pr-number> --repo <owner>/<repo> --json files --jq '.files[].path'
```

Read each changed file in the HEAD branch to understand surrounding context beyond the diff hunks.

## Phase 3: Analyze changes

Review every changed file against these criteria:

### Correctness
- Logic errors, off-by-one, null/undefined handling
- Race conditions, concurrency issues
- Missing error handling or silent failures
- Major logic errors based on standards oftware engineering best practices for this codebase

### Security
- Injection vulnerabilities (SQL, XSS, command)
- Secrets or credentials in code
- Unsafe deserialization, path traversal

### Project standards (from AGENTS.md)
- TypeScript uses `yarn`; Python uses `uv` with `venv` and PEP 8
- Database tables use UUID IDs, soft deletes (`enabled`, `created_on`, `deleted_on`, `updated_on`)
- Semver versioning obeyed
- OpenAPI 3.2.0, JSON 2020-12 compliance where applicable
- Marketing pages updated when product-facing changes are introduced
- Round-trip fidelity rules for `apiome-rest` changes

### Code quality
- DRY violations — repeated logic that should be extracted
- Functions too long or doing too much
- Naming clarity and consistency
- Missing or inadequate tests for new/changed behavior
- Dead code or unused imports

### Performance
- Unnecessary allocations, N+1 queries, blocking I/O in async paths
- Missing pagination, unbounded collections

## Phase 4: Post review comments

Use `gh` to submit a pull request review with file-specific comments. Each comment must include:

1. **What** — the problem or improvement opportunity
2. **Why** — the reasoning or risk if left unchanged
3. **Suggested diff** — a concrete code change using GitHub's suggestion syntax

DO NOT Comment on style, formatting, minor improvements or documentation gaps.

### Comment format

Use GitHub suggestion blocks so the author can apply fixes with one click:

````
The `fetchUser` call doesn't handle the case where the user is not found,
which will throw an unhandled exception at line N.

```suggestion
const user = await fetchUser(id);
if (!user) {
  throw new NotFoundException(`User ${id} not found`);
}
```
````

### Severity levels

Prefix each comment with a severity tag:

- **🔴 Critical** — Must fix before merge (bugs, security, data loss)
- **🟡 Suggestion** — Should fix; improves quality or maintainability
- **🟢 Nit** — Optional polish; style, naming, minor simplification

### Submitting the review

Collect all comments and submit as a single review:

```bash
gh api repos/<owner>/<repo>/pulls/<pr-number>/reviews \
  --method POST \
  --field event="COMMENT" \
  --field body="Code review completed. See inline comments for details." \
  --field comments="$(cat comments.json)"
```

Where `comments.json` is a JSON array of objects:

```json
[
  {
    "path": "src/example.ts",
    "line": 42,
    "body": "🟡 **Suggestion**: ... \n\n```suggestion\n...\n```"
  }
]
```

If the GitHub review API is impractical for the number of comments, fall back to individual comments:

```bash
gh pr comment <pr-number> --body "<comment-body>"
```

## Phase 5: Summary comment

After posting inline comments, leave a top-level summary on the PR:

```bash
gh pr comment <pr-number> --body "<summary>"
```

The summary must include:

- Total files reviewed
- Breakdown by severity (critical / suggestion / nit)
- High-level assessment of the PR's readiness
- "Review performed by <editor-name, like Copilot or Cursor> using <model-name> model"
