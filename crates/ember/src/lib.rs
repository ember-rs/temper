//! Coding agents for Temper, driven over the Agent Client Protocol (ACP).
//!
//! Temper doesn't run its own agent loop. It is an ACP client that drives
//! existing harnesses. This is the only crate allowed to depend on
//! `agent-client-protocol`, and it exposes its own types so the rest of the
//! workspace never names an ACP type.
//!
//! Must not depend on `bellows` or `graph-flow` (`cargo xtask layers`).
mod agent;
mod error;
pub use agent::{Agent, Session};
pub use error::Error;

#[cfg(feature = "test-support")]
/// Utilities for testing code that uses Ember.
pub mod test_support;
