# 0002: Crate boundaries and third-party isolation

- **Status:** accepted
- **Date:** 2026-11-07

## Context

Temper orchestrates coding agents; it doesn't implement them. Existing harnesses
(Claude Code, Codex, ...) already own the agent loop: prompts, context, tools,
and model choice. Temper drives them over the
[Agent Client Protocol](https://agentclientprotocol.com) (ACP), as the client.

Spike #37 drove Claude Code and Codex over ACP and found how much of a leash
the client actually holds:

- Both report every tool call, so the client sees what the agent does.
- Neither does its work through the client. They read, write, and run commands
  themselves, even when the client offers file and terminal access.
- They ask the client for permission only when the client puts the session in
  a strict mode. In their default modes they never ask, and denying has no
  effect. In a strict mode they ask before every write and command and respect
  a denial. They never ask before reads.

The first versions will lean on `graph-flow` (workflow graphs) and
`agent-client-protocol` (the protocol's official Rust SDK). Either may be
replaced: graph-flow by a home-grown engine if it falls short, the ACP SDK if
the protocol or a better library moves on. The protocol is young and its SDK
already at a 5.x major, so expect breaking changes.

graph-flow ships an optional `rig` feature for calling LLMs from graph tasks.
We don't use it: the graph engine never talks to a model or an agent directly.

## Decision

- `ingot` is pure: std only, with an empty dependency list enforced by
  `cargo xtask layers`. Domain logic and invariants live here, where they can be
  proven and property-tested.
- `bellows` is the only crate that depends on `graph-flow`, with
  `default-features = false` (no Postgres yet, no `rig` feature). It must not
  depend on `ember` or `agent-client-protocol`.
- `ember` is the only crate that depends on `agent-client-protocol`. It is the
  ACP client: it starts agents, holds sessions, and answers their requests. It
  must not depend on `bellows` or `graph-flow`.
- `temper` (the binary) is the composition root that wires the graph engine to
  agent-backed tasks.
- No graph-flow or ACP types appear in another crate's public API.
- Don't write a `GraphEngine` trait yet. Extract it when a second implementation
  exists, so its shape comes from two real engines instead of a guess.

## Consequences

- Temper doesn't control the agent loop. Prompts, context, token spend, and
  model choice belong to the harness, and Temper sees only what ACP exposes.
- What Temper relies on from an agent has to be written down and checked
  against each harness, because Temper can't enforce it by construction.
- The sandbox, not the protocol, is the boundary. ACP adds visibility and a
  veto over what the agent chooses to ask about, and only once Temper has set
  a strict mode itself.
- Swapping either library touches exactly one crate plus the composition root.
