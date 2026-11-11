//! Fake agents for integration tests.
mod fake;

pub use fake::FakeAgent;
pub(crate) use fake::SessionSetup;
pub(crate) use fake::serve;
