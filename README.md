# Temper

An agentic SDLC orchestrator, built Rust-first and verification-first.

Steel is tempered by controlled stress until it is tough. Temper applies the
same idea to software: every change passes through a pyramid of checks, from
types and unit tests up through property tests, fuzzing, bounded proofs, and
model-checked specifications. The goal is a codebase where agents can work
safely *because* the gates are strict.

**Status:** scaffold. The crates are empty except for a throwaway exemplar in
`ingot::lifecycle` that exists to prove every gate is wired.

## Quickstart

```sh
. bin/activate-hermit   # or prefix commands with bin/
just bootstrap          # one-time: tools for the fast tier
just check              # fast gate (what CI runs, plus `just deny`)

just bootstrap-heavy    # one-time, several GB: Kani, nightly, cargo-mutants, ...
just verify             # everything, including proofs and mutation testing
```

CI runs only the fast tier. The heavy tier runs locally before merging changes
to `crates/ingot` or `spec/` ([ADR 0004](docs/adr/0004-github-and-ci-tiers.md)).

All tools are pinned and installed inside the repo (`.hermit/`). Nothing
touches your global `~/.rustup`, `~/.cargo`, `~/.kani`, or `~/.quint`.

## Crate map

| Crate | Package | Role |
|---|---|---|
| `ingot` | `temper-ingot` | Pure domain core: state machines, invariants. No IO, no async, no deps. |
| `bellows` | `temper-bellows` | Workflow graph engine. The only crate that touches `graph-flow`. |
| `ember` | `temper-ember` | LLM access. The only crate that touches `rig`. |
| `temper` | `temper` | The binary and composition root. Wires the others together. |
| `xtask` | `xtask` | Repo automation (`cargo xtask layers`). |

Planned: `crucible` (sandbox), `assay` (review), `billet` (store), `sluice` (bus).

Dependency direction is enforced, not just documented: see
[ADR 0002](docs/adr/0002-crate-boundaries.md) and `cargo xtask layers`.

## Verification pyramid

| Layer | Tool | Command | In CI |
|---|---|---|---|
| Format, lints | rustfmt, clippy (pedantic, no `unwrap`/`panic`, `unsafe` forbidden) | `just lint` | ✓ |
| Architecture | `cargo xtask layers` | `just layers` | ✓ |
| Unit tests | nextest | `just test` | ✓ |
| Property tests | bolero (random under `cargo test`) | `just test` | ✓ |
| Spec ↔ code | Quint traces replayed against Rust via quint-connect | `just test` | ✓ |
| Supply chain | cargo-deny, cargo-machete | `just deny`, `just machete` | ✓ |
| Fuzzing | bolero + libFuzzer + ASan | `just fuzz <harness>` | local |
| Bounded proofs | Kani | `just kani` | local |
| Spec model checking | Quint + Apalache | `just spec` | local |
| Test strength | cargo-mutants | `just mutants` | local |
| Coverage | cargo-llvm-cov (report only) | `just cov` | local |

See [ADR 0003](docs/adr/0003-verification-stack.md) for why each layer exists.

## Layout

```text
bin/            Hermit (pinned toolchain)
crates/         workspace crates
spec/           Quint specifications
xtask/          repo automation
docs/adr/       architecture decision records
```

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE), at your option.
