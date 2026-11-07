// -*- mode: Bluespec; -*-
/**
 * Retryable unit-of-work lifecycle.
 *
 * Scaffold exemplar mirroring crates/ingot/src/lifecycle.rs. It is replayed
 * against the Rust implementation by crates/ingot/tests/lifecycle_mbt.rs via
 * quint-connect, so the action names and state variables here are a contract:
 * rename them in both places or neither.
 */
module lifecycle {
  pure val MAX_ATTEMPTS = 3

  type Status = Pending | Running | Succeeded | Failed

  var status: Status
  var attempts: int

  // ---- actions ------------------------------------------------------------

  action init = all {
    status' = Pending,
    attempts' = 0,
  }

  action Start = all {
    status == Pending,
    status' = Running,
    attempts' = attempts + 1,
  }

  action Succeed = all {
    status == Running,
    status' = Succeeded,
    attempts' = attempts,
  }

  action Fail = all {
    status == Running,
    status' = Failed,
    attempts' = attempts,
  }

  action Retry = all {
    status == Failed,
    attempts < MAX_ATTEMPTS,
    status' = Pending,
    attempts' = attempts,
  }

  // Terminal states have no enabled action; stutter so traces can end there.
  action Done = all {
    isTerminal,
    status' = status,
    attempts' = attempts,
  }

  action step = any { Start, Succeed, Fail, Retry, Done }

  // ---- properties ---------------------------------------------------------

  val isTerminal = or {
    status == Succeeded,
    status == Failed and attempts >= MAX_ATTEMPTS,
  }

  /// Must match `Lifecycle::is_valid` in Rust.
  val inv = and {
    attempts >= 0,
    attempts <= MAX_ATTEMPTS,
    status == Pending implies attempts < MAX_ATTEMPTS,
    status != Pending implies attempts >= 1,
  }

  // ---- tests --------------------------------------------------------------

  run happyPathTest =
    init.then(Start).then(Succeed)
      .then(all { assert(status == Succeeded), assert(attempts == 1), Done })

  run exhaustRetriesTest =
    init
      .then(Start).then(Fail).then(Retry)
      .then(Start).then(Fail).then(Retry)
      .then(Start).then(Fail)
      .then(all { assert(isTerminal), assert(attempts == MAX_ATTEMPTS), Done })

  run cannotRetryAfterLastAttemptTest =
    init
      .then(Start).then(Fail).then(Retry)
      .then(Start).then(Fail).then(Retry)
      .then(Start).then(Fail)
      .then(Retry)
      .fail()
}
