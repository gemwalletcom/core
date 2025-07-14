---
description: "Automatically create branch, commit changes, and create a pull request"
usage: "/push_pr [issue-number]"
examples:
  - "/push_pr"
  - "/push_pr 123"
---

# Push PR Workflow (Automatic)

Automatically create a new branch, commit all changes, and create a pull request with smart defaults.

## Arguments
- `[issue-number]`: Optional issue number to reference in the PR (e.g., `/push_pr 123`)

## Automatic Behavior

The command will automatically:

1. **Generate branch name** based on changed files and content
2. **Create commit message** by analyzing the changes
3. **Add all changes** to staging area
4. **Create new branch** with generated name
5. **Commit changes** with generated message
6. **Push branch** to remote repository
7. **Create pull request** with descriptive title and body

## Smart Defaults

- **Branch naming:** `feat/auto-TIMESTAMP` or `fix/auto-TIMESTAMP` based on changes
- **Commit messages:** Generated from file changes and content analysis
- **PR titles:** Descriptive titles based on the changes made
- **PR descriptions:** Includes summary of changes and test information

## Usage Examples

```bash
# Simple usage - everything automated
/push_pr

# With issue reference
/push_pr 456
```

## Implementation

I'll analyze the current git changes, generate appropriate branch names and commit messages, then execute the full workflow automatically.

The commit message will automatically include the Claude Code attribution as per the repository's commit conventions.

---

**Arguments received:** `$ARGUMENTS`

Let me execute the automated push PR workflow, analyzing the current changes to generate appropriate branch name and commit message.