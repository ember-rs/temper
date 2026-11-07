# 0004: GitHub hosting and tiered CI

- **Status:** accepted; the no-PR workflow is superseded by [0005](0005-auto-merged-prs.md)
- **Date:** 2026-11-07

## Context

The project started on a self-hosted Gitea. The first CI run passed every gate
but then hung for 17 minutes uploading a single ~5 GB cache entry (rustup
toolchains, Kani, Quint, and `target/`), and the job failed. Two design
mistakes caused it, neither specific to Gitea: the fast `check` job bootstrapped
the entire heavy toolchain, and everything went into one cache entry.

The project is open source (MIT OR Apache-2.0), which makes public GitHub
hosting free for CI. GitHub also provides native sub-issues and, for
organization-owned repos, issue types.

## Decision

- Host at `github.com/ember-rs/temper`, a public repo in an organization so
  issues can use native types (Epic, Story, Spike, Bug, Chore).
- Planning is milestones → epics → stories, with stories as sub-issues of their
  epic.
- One-person project: commits go directly to `main` with Conventional Commit
  messages and `Closes #N` footers. No PRs, so no PR template.
- CI runs only the fast tier: `just ci` = `just check` + `just deny`.
- The heavy tier (Quint + Apalache, Kani, cargo-mutants, fuzzing, coverage)
  runs locally via `just verify`, which is required before merging changes to
  `crates/ingot` or `spec/`.
- `just bootstrap` installs only what the fast tier needs; `just
  bootstrap-heavy` adds the rest.
- CI caches are small and separate: Hermit packages and toolchains in one,
  cargo registry and `target/` via `Swatinem/rust-cache`.
- Third-party actions are pinned to commit SHAs.

## Consequences

- Work is committed directly to `main` (one-person project, no PRs), so CI
  reports after the fact rather than gating. A commit can pass CI while
  breaking a Kani proof or the Apalache check; `AGENTS.md` makes running
  `just verify` part of the definition of done.
- Revisit putting the heavy tier in CI (nightly schedule, or a self-hosted
  runner with a persistent tool volume) once it's worth the minutes.
