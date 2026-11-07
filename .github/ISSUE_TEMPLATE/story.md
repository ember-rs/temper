---
name: Story
about: A slice of work sized to one PR
type: Story
---

As a **<who>**, I want **<what>**, so that **<why>**.

## Context

Why this story exists and what it unlocks.

## Acceptance criteria

- [ ] Observable behaviour, not implementation. Each one becomes a test.

## Open decisions

Choices left for whoever picks this up. Record each answer here when it's
made, with the reason; write an ADR if it will outlive the story.

- [ ] Decision to make (options worth considering, if known)

## Verification

Which gates prove it (delete what doesn't apply):

- [ ] Unit tests
- [ ] bolero property
- [ ] Kani proof
- [ ] Quint spec + trace replay
- [ ] `just mutants` clean for the changed code

## Out of scope

What this story deliberately does not do.

## Learning

- **Concepts:** ideas worth understanding (e.g. typestate, inductive invariants).
- **Rust:** language details this story exercises (e.g. lifetimes in X, `?` with `From`).
- **Check-in:** questions to answer in the story comments to confirm it landed.
