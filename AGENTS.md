# Agent rules

Rules for any coding agent working in this repo. Humans follow them too.

## Definition of done

1. `just check` is green. Run it before saying a task is finished.
2. If you changed `crates/ingot` or `spec/`, `just verify` is green too. CI
   only runs the fast tier (`just ci`), so this is on you.
3. Report failures with their output. Never describe a failing gate as passing.

## Environment

- Use Hermit tools: `. bin/activate-hermit`, or call `bin/just`, `bin/cargo`.
  Do not use globally installed `cargo`, `rustc`, or `quint`.
- Do not install tools or add dependencies outside `just bootstrap` and
  `just bootstrap-heavy`.

## Never, without explicit human approval

These files define what "correct" means (see `.github/CODEOWNERS`):

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

## Planning and issues

Work is tracked on GitHub (`ember-rs/temper`) as
**milestones → epics → stories**.

- **Milestones** are ordered outcomes, each ending in something demoable. No
  due dates; order is the priority.
- **Epics** are issues of type `Epic`. Their stories are linked as GitHub
  sub-issues, so the epic shows progress.
- Every issue has exactly one **issue type** (`Epic`, `Story`, `Spike`, `Bug`,
  `Chore`), belongs to a milestone, and has one or more `area/*` labels
  (`ingot`, `bellows`, `ember`, `temper`, `spec`, `infra`).
- Stories use `.github/ISSUE_TEMPLATE/story.md`; spikes use `spike.md`.
  Stories have Context, Acceptance criteria (checkboxes that become tests),
  Verification (which gates prove it), Out of scope, and Learning.
- One story is sized to one PR. Commits and PRs reference the stories they
  close (`Closes #N`).
- Commits follow [Conventional Commits](https://www.conventionalcommits.org):
  `type(scope): summary`, where `type` is `feat`, `fix`, `refactor`, `test`,
  `docs`, `ci`, `build`, or `chore`, and `scope` is the `area/*` (e.g.
  `feat(ember): ...`). Put `Closes #N` in the footer.
- Distribution is mainly a learning goal, so dogfooding (Temper working its own
  stories) comes before the distributed milestone. Early dogfooding output is
  for comparison, not merged by default.

## Learning

This project is also how its maintainer learns Rust. Learning happens inside
regular stories, not a separate track.

- Each story's **Learning** section lists the concepts and Rust details it
  exercises, plus check-in questions.
- When implementing, explain non-obvious Rust choices (ownership, borrowing,
  traits, error conversion, async) and the alternatives you rejected. Don't
  silently hand over finished code.
- Check that both the idea and the Rust intricacies landed, for example by
  asking the check-in questions in the PR discussion.

## Writing tests

- A new `ingot` state machine needs: unit tests for named scenarios, a bolero
  property over event sequences, Kani proofs that the invariant is inductive,
  and a Quint spec with a quint-connect replay test.
- A surviving mutant (`just mutants`) means a test is missing. Add the test;
  don't exclude the mutant.
- A fuzz crash becomes a fixed bug plus a named regression test.
