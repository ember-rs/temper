# 0006: Agent identity on GitHub

- **Status:** accepted (spike #9; implementation in follow-up stories)
- **Date:** 2026-11-07
- **Amends:** the "when agents get their own identity" consequence of [0005](0005-auto-merged-prs.md)

## Context

Agents act through the maintainer's own `gh` login and SSH key today. GitHub
can't tell their PRs apart, so `CODEOWNERS` can't bind and an agent could run
`just ship` on its own work. The aim is for an agent's PR to merge only after
the maintainer approves it, while the maintainer's PRs keep auto-merging.

ADR 0005 planned to require one approval and put the maintainer on the
ruleset's bypass list. That doesn't work. Community reports say the merge API
honours bypass, but auto-merge and the merge queue never do
([community #208718](https://github.com/orgs/community/discussions/208718)).
GitHub doesn't document this either way, so we test it ourselves before relying
on it. With that plan, `just ship` would stall on the maintainer's own PRs,
because nobody can approve their own PR.

## Options

| | GitHub App (`temper-agent`) | Machine user |
|---|---|---|
| Credential | Private key mints installation tokens | Fine-grained PAT, or classic PAT plus SSH key |
| Token lifetime | 1 hour, minted per session | Up to a year; revocation is manual |
| Scope | Per-permission (`contents`, `pull_requests`, `issues`) and per-repo | Repo-scoped fine-grained PAT, but the account is a full user |
| Can edit `.github/workflows/` | Only if granted `workflows` (we don't grant it) | Yes with `workflow` scope; easy to over-grant |
| Shows up as | `temper-agent[bot]`, clearly not a human | Looks like a person |
| Cost | Free | Free, but GitHub allows one machine account per person |
| Ergonomics | Needs a helper to mint tokens (JWT, then installation token) | `gh auth login` and done |

## Decision

Agents use a **GitHub App** installed only on `ember-rs/temper`, with
`contents: write`, `pull_requests: write`, `issues: write`, and `metadata:
read`. It gets **no `workflows` permission**, so the agent can't change CI or
the gate below. A helper mints a fresh installation token for each agent
session and passes it in as `GH_TOKEN`.

Review is enforced by a required status check rather than by the ruleset's
approval count, so auto-merge keeps working:

- An `agent-gate` workflow on `pull_request_target` and `pull_request_review`
  passes when the PR author isn't the App. It also passes when the author is
  the App and the head commit has an approving review from a code owner.
- `pull_request_target` runs the workflow as it exists on `main`, not the PR's
  copy, so a PR can't rewrite its own gate. The workflow only reads PR
  metadata and never checks out PR code; the danger with
  `pull_request_target` is running untrusted code with a privileged token.
  `pull_request_review` runs the PR's copy of the workflow, so on that path
  the protection is that the App can't push workflow changes at all. The
  follow-up story has to confirm both paths.
- The ruleset adds `agent-gate` to the required checks and keeps 0 required
  approvals. The maintainer's PRs pass the gate immediately and auto-merge
  as before. Agent PRs wait until the maintainer approves them.

## Consequences

- An agent's changes, including ones that weaken a gate, can't reach `main`
  without the maintainer's approval. The App also can't change any workflow.
- **The App only helps if agents can't reach the maintainer's credentials.** An
  agent running as the maintainer's OS user can read the `gh` keychain entry,
  use the SSH agent, and read the App's private key. Until agents run in a
  sandbox that holds only the minted token (the planned `crucible` crate),
  the identity marks agent PRs but doesn't contain the agent.
- The App's private key must live outside the agent's reach: the macOS
  keychain or a password manager, never the repo or `.env`.
- Inside its hour, a token can still push branches, open and close issues, and
  comment. It can't merge past `agent-gate`, approve its own PR, or edit
  workflows.
- Follow-ups: create the App and token helper, add `agent-gate` and verify it
  against a real agent PR, and keep maintainer credentials out of agent
  sessions (part of the sandbox work).
