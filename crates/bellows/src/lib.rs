//! Workflow graph engine for Temper.
//!
//! The only crate allowed to depend on `graph-flow`. Keep graph-flow types
//! from leaking through this crate's public API so the engine can be swapped
//! for a home-grown one later. Extract a trait when the second implementation
//! exists, not before.
//!
//! Must not depend on `ember` or `agent-client-protocol` (`cargo xtask layers`).
