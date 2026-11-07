//! A retryable unit-of-work lifecycle.
//!
//! **Scaffold exemplar.** This module exists to prove that every verification
//! gate (unit, bolero, Kani, Quint trace replay, mutation) is wired up. Replace
//! it with the real workflow domain; keep the shape of its tests.
//!
//! ```text
//!   Pending --Start--> Running --Succeed--> Succeeded
//!      ^                  |
//!      |                 Fail
//!      |                  v
//!      +-----Retry----- Failed   (only while attempts < MAX_ATTEMPTS)
//! ```
//!
//! Modelled in `spec/lifecycle.qnt`; keep the two in lockstep.

use core::fmt;

/// Maximum number of times a unit of work may be started.
pub const MAX_ATTEMPTS: u8 = 3;

/// Where a unit of work is in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub enum Status {
    /// Waiting to be started.
    Pending,
    /// Currently executing.
    Running,
    /// Finished successfully. Terminal.
    Succeeded,
    /// Finished unsuccessfully. May be retried while attempts remain.
    Failed,
}

/// An input that may move a [`Lifecycle`] to a new state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(bolero::TypeGenerator))]
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub enum Event {
    /// Begin an attempt.
    Start,
    /// The running attempt succeeded.
    Succeed,
    /// The running attempt failed.
    Fail,
    /// Return a failed unit of work to pending.
    Retry,
}

/// Why an [`Event`] was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionError {
    /// The event is not valid from the current status.
    Invalid {
        /// Status at the time of the event.
        from: Status,
        /// The rejected event.
        event: Event,
    },
    /// A retry was requested but every attempt has been used.
    RetriesExhausted,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid { from, event } => write!(f, "cannot apply {event:?} while {from:?}"),
            Self::RetriesExhausted => write!(f, "all {MAX_ATTEMPTS} attempts used"),
        }
    }
}

impl core::error::Error for TransitionError {}

/// The state of a single unit of work.
///
/// Fields are private so the only way to reach a state is through
/// [`Lifecycle::new`] and [`Lifecycle::next`], which preserve
/// [`Lifecycle::is_valid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub struct Lifecycle {
    status: Status,
    attempts: u8,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl Lifecycle {
    /// A fresh, pending unit of work with no attempts.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            status: Status::Pending,
            attempts: 0,
        }
    }

    /// Current status.
    #[must_use]
    pub const fn status(self) -> Status {
        self.status
    }

    /// Number of attempts started so far.
    #[must_use]
    pub const fn attempts(self) -> u8 {
        self.attempts
    }

    /// Whether no further event can ever be accepted.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        match self.status {
            Status::Succeeded => true,
            Status::Failed => self.attempts >= MAX_ATTEMPTS,
            Status::Pending | Status::Running => false,
        }
    }

    /// The state invariant. Holds for every state reachable from [`Lifecycle::new`].
    #[must_use]
    pub const fn is_valid(self) -> bool {
        let bounded = self.attempts <= MAX_ATTEMPTS;
        let consistent = match self.status {
            Status::Pending => self.attempts < MAX_ATTEMPTS,
            Status::Running | Status::Succeeded | Status::Failed => self.attempts >= 1,
        };
        bounded && consistent
    }

    /// Apply `event`, returning the new state or why it was rejected.
    ///
    /// Pure: `self` is never modified, so a rejected event leaves no trace.
    ///
    /// # Errors
    ///
    /// [`TransitionError::Invalid`] if `event` is not accepted from the current
    /// status; [`TransitionError::RetriesExhausted`] if a retry is requested
    /// after the final attempt.
    pub const fn next(self, event: Event) -> Result<Self, TransitionError> {
        let status = match (self.status, event) {
            (Status::Pending, Event::Start) => {
                return Ok(Self {
                    status: Status::Running,
                    attempts: self.attempts + 1,
                });
            }
            (Status::Running, Event::Succeed) => Status::Succeeded,
            (Status::Running, Event::Fail) => Status::Failed,
            (Status::Failed, Event::Retry) if self.attempts >= MAX_ATTEMPTS => {
                return Err(TransitionError::RetriesExhausted);
            }
            (Status::Failed, Event::Retry) => Status::Pending,
            (from, event) => return Err(TransitionError::Invalid { from, event }),
        };
        Ok(Self { status, ..self })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(events: &[Event]) -> Result<Lifecycle, TransitionError> {
        events.iter().try_fold(Lifecycle::new(), |l, &e| l.next(e))
    }

    #[test]
    fn happy_path_succeeds_on_first_attempt() {
        let l = run(&[Event::Start, Event::Succeed]).unwrap();
        assert_eq!(l.status(), Status::Succeeded);
        assert_eq!(l.attempts(), 1);
        assert!(l.is_terminal());
    }

    #[test]
    fn retry_is_allowed_until_attempts_are_exhausted() {
        use Event::{Fail, Retry, Start};
        let l = run(&[Start, Fail, Retry, Start, Fail, Retry, Start, Fail]).unwrap();
        assert_eq!(l.status(), Status::Failed);
        assert_eq!(l.attempts(), MAX_ATTEMPTS);
        assert!(l.is_terminal());
        assert_eq!(l.next(Event::Retry), Err(TransitionError::RetriesExhausted));
    }

    #[test]
    fn invalid_event_is_rejected_with_context() {
        assert_eq!(
            Lifecycle::new().next(Event::Succeed),
            Err(TransitionError::Invalid {
                from: Status::Pending,
                event: Event::Succeed,
            })
        );
    }

    #[test]
    fn is_valid_rejects_unreachable_states() {
        let state = |status, attempts| Lifecycle { status, attempts };
        for (status, attempts) in [
            (Status::Pending, MAX_ATTEMPTS),
            (Status::Running, 0),
            (Status::Succeeded, 0),
            (Status::Failed, 0),
            (Status::Running, MAX_ATTEMPTS + 1),
        ] {
            assert!(!state(status, attempts).is_valid(), "{status:?}/{attempts}");
        }
        assert!(state(Status::Pending, MAX_ATTEMPTS - 1).is_valid());
    }

    #[test]
    fn errors_render_for_humans() {
        assert_eq!(
            TransitionError::RetriesExhausted.to_string(),
            "all 3 attempts used"
        );
        let invalid = TransitionError::Invalid {
            from: Status::Succeeded,
            event: Event::Start,
        };
        assert_eq!(invalid.to_string(), "cannot apply Start while Succeeded");
    }

    #[test]
    fn failed_with_attempts_left_is_not_terminal() {
        let l = run(&[Event::Start, Event::Fail]).unwrap();
        assert!(!l.is_terminal());
    }

    /// Property: from the initial state, any sequence of events (accepted or
    /// rejected) keeps the invariant, and terminal states accept nothing.
    ///
    /// Runs as a randomized test under `cargo test`, as a coverage-guided fuzz
    /// target under `cargo bolero test`, and as a bounded proof under Kani.
    #[test]
    #[cfg_attr(kani, kani::proof)]
    #[cfg_attr(kani, kani::unwind(9))]
    fn any_event_sequence_preserves_invariant() {
        bolero::check!()
            .with_type::<[Event; 8]>()
            .for_each(|events| {
                let mut l = Lifecycle::new();
                for &e in events {
                    let was_terminal = l.is_terminal();
                    if let Ok(n) = l.next(e) {
                        assert!(!was_terminal, "terminal state accepted {e:?}");
                        l = n;
                    }
                    assert!(l.is_valid(), "invariant broken: {l:?}");
                }
            });
    }
}

/// Native Kani proofs. Unlike the bolero harness above, these quantify over
/// *every* valid state, not just those reachable in N steps, so together they
/// prove `is_valid` is an inductive invariant.
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn initial_state_is_valid() {
        assert!(Lifecycle::new().is_valid());
    }

    #[kani::proof]
    fn next_preserves_validity() {
        let l: Lifecycle = kani::any();
        kani::assume(l.is_valid());
        let e: Event = kani::any();
        if let Ok(n) = l.next(e) {
            assert!(n.is_valid());
        }
    }

    #[kani::proof]
    fn terminal_states_reject_every_event() {
        let l: Lifecycle = kani::any();
        kani::assume(l.is_valid() && l.is_terminal());
        let e: Event = kani::any();
        assert!(l.next(e).is_err());
    }
}
