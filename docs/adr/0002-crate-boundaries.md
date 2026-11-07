# 0002: Crate boundaries and third-party isolation

- **Status:** accepted
- **Date:** 2026-11-07

## Context

The first versions will lean on `graph-flow` (workflow graphs) and `rig`
(LLM providers). Either may be replaced: graph-flow by a home-grown engine if it
falls short, rig if a better option appears.

Found while scaffolding: graph-flow 0.8.1's optional `rig` feature pins
`rig-core` 0.42, while current rig is 0.46. Enabling it would build two rigs and
couple the graph engine to an LLM library version we don't control.

## Decision

- `ingot` is pure: std only, with an empty dependency list enforced by
  `cargo xtask layers`. Domain logic and invariants live here, where they can be
  proven and property-tested.
- `bellows` is the only crate that depends on `graph-flow`, with
  `default-features = false` (no Postgres yet, no `rig` feature). It must not
  depend on `ember` or `rig-core`.
- `ember` is the only crate that depends on `rig-core`. It must not depend on
  `bellows` or `graph-flow`.
- `temper` (the binary) is the composition root that wires the graph engine to
  LLM-backed tasks.
- No graph-flow or rig types appear in another crate's public API.
- Don't write a `GraphEngine` trait yet. Extract it when a second implementation
  exists, so its shape comes from two real engines instead of a guess.

## Consequences

- We give up graph-flow's built-in rig integration and write a thin adapter in
  `temper` instead.
- Swapping either library touches exactly one crate plus the composition root.
