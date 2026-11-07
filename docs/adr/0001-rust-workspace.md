# 0001: Rust workspace with a Hermit-pinned toolchain

- **Status:** accepted
- **Date:** 2026-11-07

## Context

Temper succeeds Praxis, a Go orchestrator. The rewrite is partly about learning
Rust and partly about seeing how far tooling can go toward a "correct by
construction" codebase that agents can work in safely. That only works if every
contributor (human, CI, or agent) runs the exact same tools.

## Decision

- A single Cargo workspace, edition 2024, `resolver = "3"`.
- Hermit pins rustup, just, nextest, cargo-deny, cargo-binstall, Node, and a JDK.
  `rust-toolchain.toml` is the single source of truth for the Rust version.
- Tools Hermit doesn't package are installed by `just bootstrap` at versions
  pinned in the `justfile`.
- `RUSTUP_HOME`, `CARGO_HOME`, `KANI_HOME`, and `QUINT_HOME` all live under
  `.hermit/`, so the repo never reads or writes global toolchain state.
- `just` is the one entrypoint. CI runs `just` recipes and nothing else.
- Start with four crates and add more only when real code needs them.

## Consequences

- A fresh clone downloads several GB on first bootstrap (two Rust toolchains,
  Kani, a JDK). CI caches `.hermit/`.
- Stable Rust everywhere except fuzzing, which uses the nightly that Kani pins.
