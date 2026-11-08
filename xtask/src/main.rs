//! Repo automation for Temper. Run with `cargo xtask <command>`.
//!
//! Commands:
//! - `layers`: fail if the crate dependency graph violates the architecture.

use std::collections::{BTreeSet, HashMap};
use std::process::ExitCode;

use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, PackageId};

/// A layering rule over a crate's transitive *normal* (non-dev, non-build) dependencies.
struct Rule {
    krate: &'static str,
    check: Check,
    why: &'static str,
}

enum Check {
    /// Every transitive dependency must be on this list.
    OnlyAllow(&'static [&'static str]),
    /// None of these may appear anywhere in the transitive closure.
    Forbid(&'static [&'static str]),
}

const RULES: &[Rule] = &[
    Rule {
        krate: "temper-ingot",
        check: Check::OnlyAllow(&[]),
        why: "ingot is the pure core: no IO, no async, no third-party runtime",
    },
    Rule {
        krate: "temper-bellows",
        check: Check::Forbid(&["temper-ember", "agent-client-protocol"]),
        why: "the graph engine stays agent-agnostic; the temper binary wires them together",
    },
    Rule {
        krate: "temper-ember",
        check: Check::Forbid(&["temper-bellows", "graph-flow"]),
        why: "agent access must not know about workflow graphs",
    },
];

#[expect(clippy::single_match_else, reason = "more subcommands will be added")]
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("layers") => report(layers()),
        _ => {
            eprintln!("usage: cargo xtask layers");
            ExitCode::FAILURE
        }
    }
}

fn report(result: Result<Vec<String>, cargo_metadata::Error>) -> ExitCode {
    match result {
        Ok(violations) if violations.is_empty() => {
            eprintln!("layers: ok ({} rules)", RULES.len());
            ExitCode::SUCCESS
        }
        Ok(violations) => {
            for v in &violations {
                eprintln!("layers: {v}");
            }
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("layers: cargo metadata failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn layers() -> Result<Vec<String>, cargo_metadata::Error> {
    let metadata = MetadataCommand::new().exec()?;
    let mut violations = Vec::new();

    for rule in RULES {
        let Some(closure) = normal_closure(&metadata, rule.krate) else {
            violations.push(format!("{}: crate not found in workspace", rule.krate));
            continue;
        };
        let offending: Vec<&str> = match rule.check {
            Check::OnlyAllow(allowed) => closure
                .iter()
                .map(String::as_str)
                .filter(|d| !allowed.contains(d))
                .collect(),
            Check::Forbid(forbidden) => closure
                .iter()
                .map(String::as_str)
                .filter(|d| forbidden.contains(d))
                .collect(),
        };
        if !offending.is_empty() {
            violations.push(format!(
                "{} must not depend on [{}]: {}",
                rule.krate,
                offending.join(", "),
                rule.why
            ));
        }
    }
    Ok(violations)
}

/// Names of every package reachable from `krate` through normal dependency edges.
fn normal_closure(metadata: &Metadata, krate: &str) -> Option<BTreeSet<String>> {
    let resolve = metadata.resolve.as_ref()?;
    let root = metadata
        .workspace_packages()
        .into_iter()
        .find(|p| p.name.as_str() == krate)?;
    let names: HashMap<&PackageId, &str> = metadata
        .packages
        .iter()
        .map(|p| (&p.id, p.name.as_str()))
        .collect();
    let nodes: HashMap<&PackageId, _> = resolve.nodes.iter().map(|n| (&n.id, n)).collect();

    let mut seen = BTreeSet::new();
    let mut stack = vec![&root.id];
    while let Some(id) = stack.pop() {
        let Some(node) = nodes.get(id) else { continue };
        for dep in &node.deps {
            let is_normal = dep
                .dep_kinds
                .iter()
                .any(|k| k.kind == DependencyKind::Normal);
            if is_normal && seen.insert(names.get(&dep.pkg)?.to_string()) {
                stack.push(&dep.pkg);
            }
        }
    }
    Some(seen)
}
