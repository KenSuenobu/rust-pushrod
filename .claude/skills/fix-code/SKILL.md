---
name: fix-code
description: >-
  Fix a GitHub pull request by number. Fetches PR notes/review comments,
  switches to the PR branch, implements requested fixes, builds, tests,
  pushes, and comments on the PR. Use when the user invokes fix-code with
  a PR number.
disable-model-invocation: true
---

# Fix Code (`/fix-code <number>`)

When user invokes **fix-code** with a PR number, treat it as a **GitHub pull request** in the **current repository** (`KenSuenobu/apiome`). Follow the workflow end to end unless the user terminates or the environment blocks (auth, permissions, missing `gh`, etc.).

## Guidelines

- Act as a senior software engineer.
- Apply **AGENTS.md** rules as the quality baseline.
- Only change what the PR notes require — avoid unnecessary refactors.
- Never commit credentials or tokens.
- If blocked, stop and explain, or switch to plan mode and ask for clarification.

## Phase 1: Fetch PR metadata and notes

Fetch the pull request details:

```bash
gh pr view <number> --repo KenSuenobu/apiome --json title,body,headRefName,baseRefName,author,state,reviewComments,comments
```

Extract:

1. **Branch name** from `headRefName`.
2. **PR body** — the description may contain fix requests or context.
3. **Review comments** (`reviewComments`) — inline code-review comments with specific fix requests.
4. **General comments** (`comments`) — top-level conversation comments with additional fix requests.

Collect every actionable fix request from the body, review comments, and general comments into a consolidated list. Summarize the list in chat so the full intent is in context.

## Phase 2: Switch to the PR branch

Switch to the branch:

```bash
git checkout <branch-name>
git pull origin <branch-name>
```

If the branch does not exist locally or the checkout fails, stop and explain.

## Phase 3: Implement fixes

Work through each fix request from the consolidated list:

- Read the relevant files and understand the surrounding context before making changes.
- Implement each fix as described in the PR notes.
- If a fix request is ambiguous or contradicts the codebase, **stop and ask** before proceeding.
- Keep changes minimal and focused — only touch what the notes require.

## Phase 4: Build and test

From the **repository root**, run the project's standard checks:

### Build

```bash
yarn build
```

Run any package-specific builds required by workspace rules.

### Test

```bash
yarn test
```

Run any package-specific tests that the changes touch.

- If builds or tests fail, **fix the failures** before proceeding.
- Re-run build and test until both pass cleanly.
- Test all code, not just changes, so regressions are caught.

## Phase 5: Commit and push

### Commit

```bash
git add -A
git commit -m "PR #<number> - apply review fixes"
```

### Push

```bash
git push origin <branch-name>
```

## Phase 6: Comment on the PR

Post a comment on the pull request confirming the fixes:

```bash
gh pr comment <number> --repo KenSuenobu/apiome --body "Pull request fixes applied as per notes by <editor-name, like Copilot or Cursor> using <model-name> model."
```

Replace `<model-name>` with the actual model name you are running as (e.g. `claude-sonnet-4-20250514`).

### EXTREMELY IMPORTANT

- DO NOT close the pull request
- DO NOT delete the branch

## Phase 7: Switch back to main

```bash
git checkout main
```
