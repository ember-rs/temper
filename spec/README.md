# Specifications

Quint models of Temper's behaviour. Each spec is a contract with a Rust
implementation, kept honest in two ways:

1. **Model checking.** `just spec` runs `quint test`, randomized simulation, and
   Apalache's bounded model check against each spec's `inv` invariant.
2. **Trace replay.** A `quint_run` test in `crates/<crate>/tests/<name>_mbt.rs`
   generates traces from the spec and replays them against the Rust code,
   comparing state after every step. If the code and the spec drift, `just test`
   fails.

| Spec | Implementation | Replay test |
|---|---|---|
| `lifecycle.qnt` | `ingot::lifecycle` | `crates/ingot/tests/lifecycle_mbt.rs` |

## Conventions

- Every spec defines `init`, `step`, and an invariant named `inv`. `just spec`
  relies on that name.
- Every action listed in `step` is a named action. quint-connect cannot replay
  anonymous `all { ... }` branches.
- Action names and state variables are part of the contract with the replay
  test. Rename them in both places or neither.
- Terminal states need an explicit stutter action (e.g. `Done`), or simulation
  deadlocks there.
