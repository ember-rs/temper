# 0005: Auto-merged draft PRs

- **Status:** accepted
- **Date:** 2026-11-07
- **Supersedes:** the direct-to-`main` workflow in [0004](0004-github-and-ci-tiers.md)

## Context

ADR 0004 had commits going straight to `main`, which meant CI reported after the
fact instead of gating. Requiring review would gate, but in a one-person project
it only adds waiting. Later, agents will open PRs that *should* need review.

## Decision

- Every change reaches `main` through a PR. A ruleset on `main` requires PRs
  (0 approvals), the `check` and `pr-title` status checks, and linear history,
  and blocks force pushes and deletion. No bypass actors.
- Squash merge only. The squash commit takes the PR title and body, so the
  history on `main` is one Conventional Commit per story with `Closes #N`.
- The `pr-title` workflow rejects titles that aren't `type(area): summary`.
- `just pr` opens PRs as **drafts**, which can't auto-merge, so work in progress
  never merges by accident. `just ship` marks the PR ready and enables
  auto-merge. `just unship` turns it back into a draft.
- `gh` is pinned by Hermit so the recipes behave the same everywhere.

## Consequences

- CI gates `main`, but only the fast tier. `just verify` remains a local
  responsibility before shipping changes to `crates/ingot` or `spec/`.
- When agents get their own identity (#9): raise required approvals to 1, turn on
  code-owner review, and add the maintainer to the ruleset's bypass list. The
  maintainer's PRs keep auto-merging, while agent PRs wait for review.
- Admin settings (ruleset, merge options) live in GitHub rather than the repo.
  They're recorded here so they can be recreated.
