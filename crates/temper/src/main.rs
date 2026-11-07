//! The `temper` binary: composition root wiring `ingot`, `bellows`, and `ember`.

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut out = io::stdout().lock();
    let lifecycle = ingot::lifecycle::Lifecycle::new();
    match writeln!(
        out,
        "temper {} (scaffold; initial state: {:?})",
        env!("CARGO_PKG_VERSION"),
        lifecycle.status()
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
