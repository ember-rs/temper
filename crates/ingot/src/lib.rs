//! Pure domain core for Temper: workflow state machines and their invariants.
//!
//! `ingot` performs no IO and has no async runtime. Everything here is a pure
//! function of its inputs, which is what makes it amenable to property tests,
//! fuzzing, Kani proofs, and model-based testing against `spec/`.
//!
//! `cargo xtask layers` enforces that this crate's dependency list stays empty
//! (or on an explicit allowlist).

pub mod lifecycle;
