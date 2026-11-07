//! Model-based test: replay traces generated from `spec/lifecycle.qnt`
//! against `ingot::lifecycle` and check the states match after every step.
//!
//! Requires `quint` on PATH (provided by `npm ci` under Hermit).

use ingot::lifecycle::{Event, Lifecycle, Status};
use quint_connect::{Driver, Result, State, Step, quint_run, switch};
use serde::Deserialize;

/// Spec-side view of the state, decoded from Quint's ITF trace format.
#[derive(Debug, PartialEq, Eq, Deserialize)]
struct SpecState {
    status: SpecStatus,
    attempts: u8,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "tag")]
enum SpecStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl From<Status> for SpecStatus {
    fn from(s: Status) -> Self {
        match s {
            Status::Pending => Self::Pending,
            Status::Running => Self::Running,
            Status::Succeeded => Self::Succeeded,
            Status::Failed => Self::Failed,
        }
    }
}

impl State<LifecycleDriver> for SpecState {
    fn from_driver(driver: &LifecycleDriver) -> Result<Self> {
        Ok(Self {
            status: driver.lifecycle.status().into(),
            attempts: driver.lifecycle.attempts(),
        })
    }
}

#[derive(Default)]
struct LifecycleDriver {
    lifecycle: Lifecycle,
}

impl LifecycleDriver {
    /// The spec only takes enabled actions, so the implementation must accept them.
    fn apply(&mut self, event: Event) -> Result {
        self.lifecycle = self.lifecycle.next(event)?;
        Ok(())
    }
}

impl Driver for LifecycleDriver {
    type State = SpecState;

    fn step(&mut self, step: &Step) -> Result {
        switch!(step {
            init => self.lifecycle = Lifecycle::new(),
            Start => self.apply(Event::Start)?,
            Succeed => self.apply(Event::Succeed)?,
            Fail => self.apply(Event::Fail)?,
            Retry => self.apply(Event::Retry)?,
            Done => assert!(self.lifecycle.is_terminal()),
        })
    }
}

// Named `run` tests in the spec are executed by `quint test` (see `just spec`);
// `quint_test` replay needs `mbt::actionTaken`, which Quint test mode omits.
#[quint_run(spec = "../../spec/lifecycle.qnt", max_samples = 200, max_steps = 12)]
fn simulated_traces_match_spec() -> impl Driver {
    LifecycleDriver::default()
}
