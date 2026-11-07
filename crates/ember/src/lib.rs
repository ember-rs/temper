//! LLM access for Temper.
//!
//! The only crate allowed to depend on `rig-core`. Exposes provider-agnostic
//! types so the rest of the workspace never names a rig type.
//!
//! Must not depend on `bellows` or `graph-flow` (`cargo xtask layers`).
