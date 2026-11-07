# Temper task runner. `just` lists recipes.
#
# Run through Hermit (`. bin/activate-hermit`, or `bin/just ...`) so every tool
# below is the pinned version.
#
# Tiers:
#   just check    fast inner loop (seconds): fmt, clippy, layers, tests, unused deps
#   just ci       what CI runs: check + deny
#   just verify   everything, local only: ci + spec + kani + mutants

set shell := ["bash", "-euo", "pipefail", "-c"]

# Tools not packaged by Hermit, installed into the project-local CARGO_HOME.
mutants_version := "27.1.0"
llvm_cov_version := "0.9.1"
machete_version := "0.9.2"
bolero_version := "0.13.5"
kani_version := "0.68.0"

# Nightly used only for fuzzing. Matches Kani's pinned nightly to avoid a second download.
nightly := "nightly-2026-08-21"

default:
    @just --list --unsorted

# Setup for the fast tier (`just check`, `just ci`). Enough for CI.
bootstrap:
    cargo binstall --no-confirm --locked --disable-telemetry \
        cargo-machete@{{machete_version}}
    npm ci --no-fund --no-audit

# Setup for the heavy tier (`just verify`, fuzzing, coverage). Local only; several GB.
bootstrap-heavy: bootstrap
    cargo binstall --no-confirm --locked --disable-telemetry \
        cargo-llvm-cov@{{llvm_cov_version}} \
        cargo-bolero@{{bolero_version}} \
        kani-verifier@{{kani_version}}
    # Prebuilt cargo-mutants is x86_64-only on macOS; build it natively.
    cargo binstall --no-confirm --locked --disable-telemetry --strategies compile \
        cargo-mutants@{{mutants_version}}
    cargo kani setup
    rustup toolchain install {{nightly}} --profile minimal

# --- fast tier -----------------------------------------------------------------

# Fast gate. Done means this is green.
check: fmt-check lint layers test machete

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets --locked -- -D warnings

# Architecture rules on the crate dependency graph.
layers:
    cargo xtask layers

# Unit, property (bolero), and Quint trace-replay tests, plus doctests.
test *args:
    cargo nextest run --workspace --locked {{args}}
    cargo test --workspace --locked --doc

machete:
    cargo machete

# Supply chain: advisories, licenses, bans, sources.
deny:
    cargo deny --locked check

# What CI runs: the fast gate plus the supply-chain check.
ci: check deny

# --- heavy tier (local; needs `just bootstrap-heavy`) -----------------------------

# Everything. Run before merging changes to `crates/ingot` or `spec/`.
verify: ci spec kani mutants

# Quint: typecheck, spec unit tests, randomized simulation, then Apalache model check.
spec: spec-quick
    cd spec && for f in *.qnt; do quint verify "$f" --invariant=inv --max-steps=12; done

# Quint without Apalache (no JVM startup): typecheck, tests, simulation.
spec-quick:
    cd spec && for f in *.qnt; do \
        quint typecheck "$f" && \
        quint test "$f" && \
        quint run "$f" --invariant=inv --max-samples=2000; \
    done

# Bounded model checking of ingot (native proofs + bolero harnesses).
kani:
    cd crates/ingot && cargo kani --tests

# Mutation testing. Surviving mutants mean missing or weak tests.
mutants *args:
    cargo mutants --workspace {{args}}

# Mutation testing restricted to lines changed against `base`.
mutants-diff base="origin/main":
    git diff {{base}} > target/mutants.diff
    cargo mutants --workspace --in-diff target/mutants.diff

# Coverage-guided fuzzing of one bolero harness, e.g. `just fuzz lifecycle::tests::any_event_sequence_preserves_invariant`.
fuzz target runs="1000000" package="temper-ingot":
    cargo bolero test --package {{package}} --toolchain {{nightly}} --runs {{runs}} {{target}}

# List bolero harnesses.
fuzz-list package="temper-ingot":
    cargo bolero list --package {{package}}

# HTML coverage report at target/llvm-cov/html.
cov:
    cargo llvm-cov nextest --workspace --html
