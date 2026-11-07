# 0003: Verification stack

- **Status:** accepted
- **Date:** 2026-11-07

## Context

We want the full testing pyramid where it pays off, plus formal verification of
the distributed protocol. We also want gates strict enough that an agent cannot
"pass" by writing weak tests or weakening a spec.

## Decision

| Concern | Tool | Why this one |
|---|---|---|
| Design-level protocol spec | **Quint** + Apalache | TLA semantics with typed, programmer-friendly syntax, a REPL, and in-spec tests. Weaker than TLA+ on liveness/fairness, no TLAPS. |
| Spec ↔ implementation | **quint-connect** | Replays Quint traces against the real Rust code, so the spec can't silently drift from the implementation. |
| Properties + fuzzing | **bolero** | One harness runs as a randomized test (`cargo test`), a coverage-guided fuzz target (`cargo bolero`), and a Kani harness. |
| Proofs | **Kani** (native `#[kani::proof]`) | Bounded model checking. Native proofs over `kani::any()` states prove invariants are *inductive*, which bolero's sequence harness can't do. Function contracts are available later. |
| Test strength | **cargo-mutants** | Catches tests that pass but assert nothing. Full run on `main`, changed lines on PRs. |
| Supply chain | **cargo-deny**, **cargo-machete** | License allowlist = exactly what's in use; any addition is a reviewed change. |

Later, once there are actors and a network: Stateright (model checking actual
Rust actor code) and turmoil or madsim (deterministic simulation). loom and
shuttle only if we write custom concurrency primitives.

Not chosen: proptest. Its shrinking and strategy combinators are better than
bolero's, and `proptest-state-machine` would cover stateful testing. Quint trace
replay covers the stateful case for us, and bolero's single-harness model wins
for the rest. Revisit if shrinking quality hurts.

## Consequences

- Fuzzing needs nightly. We reuse Kani's pinned nightly.
- `#[cfg(kani)]` proofs are excluded from mutation testing because they never
  run under `cargo test`. `just kani` covers them.
- quint-connect's `switch!` macro needs `anyhow` in the calling crate. Named
  Quint `run` tests are checked by `quint test`, not replayed, because Quint's
  test mode doesn't emit `mbt::actionTaken`.
