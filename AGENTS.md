# Agent rules

Rules for any coding agent working in this repo. Humans follow them too.

## Definition of done

1. `just check` is green. Run it before saying a task is finished.
2. If you changed `crates/ingot` or `spec/`, `just verify` is green too.
3. Report failures with their output. Never describe a failing gate as passing.

## Environment

- Use Hermit tools: `. bin/activate-hermit`, or call `bin/just`, `bin/cargo`.
  Do not use globally installed `cargo`, `rustc`, or `quint`.
- Do not install tools or add dependencies outside `just bootstrap`.

## Never, without explicit human approval

These files define what "correct" means (see `.gitea/CODEOWNERS`):

- Weaken or delete a test, property, Kani proof, or Quint invariant to make a
  build pass. Fix the code instead, or stop and explain why the spec is wrong.
- Edit `spec/`, `xtask/`, `deny.toml`, `clippy.toml`, `[workspace.lints]`,
  `.cargo/`, `.config/`, `justfile`, `rust-toolchain.toml`, `bin/hermit.hcl`,
  or `crates/*/tests/*_mbt.rs`.
- Add `#[allow(...)]` or `#[expect(...)]` without a `reason = "..."`.
- Add a crate to `ingot`'s `[dependencies]`.
- Add a license to `deny.toml` or a `cargo-machete` ignore entry.
- Use `unsafe`. It is forbidden workspace-wide.

## Crate boundaries

| Crate | May depend on | Must not depend on |
|---|---|---|
| `ingot` | nothing | everything (std only) |
| `bellows` | `ingot`, `graph-flow` | `ember`, `rig-core` |
| `ember` | `ingot`, `rig-core` | `bellows`, `graph-flow` |
| `temper` | all of the above | — |

`cargo xtask layers` enforces this. Do not let `graph-flow` or `rig` types appear
in another crate's public API.

## Writing code

- Put domain logic in `ingot` as pure functions over plain data. IO goes at
  the edges (`bellows`, `ember`, `temper`).
- Make invalid states unrepresentable: private fields, constructors that
  validate, enums over booleans, newtypes for IDs.
- No `unwrap`, `expect`, `panic!`, `todo!`, or `unimplemented!` outside tests.
  Return typed errors.
- Every public item has a doc comment.

## Writing tests

- A new `ingot` state machine needs: unit tests for named scenarios, a bolero
  property over event sequences, Kani proofs that the invariant is inductive,
  and a Quint spec with a quint-connect replay test.
- A surviving mutant (`just mutants`) means a test is missing. Add the test;
  don't exclude the mutant.
- A fuzz crash becomes a fixed bug plus a named regression test.
