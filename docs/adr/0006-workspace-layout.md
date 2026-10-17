# 0006. Workspace layout

Status: accepted

## Decision

Cargo workspace with one crate per concern:

- `crates/sim`: deterministic game rules, no rendering, no I/O.
- `crates/data`: content schemas, loaders, validators, content IDs.
- `crates/net`: protocol, serialization, prediction and reconciliation.
- `crates/server`: headless authoritative server.
- `crates/client`: Bevy rendering, input, UI, audio.
- `crates/tools`: CLI for validation, balance simulation, replay runner and
  save inspection.

Dependencies point inward: `client`, `server` and `tools` may depend on `sim`,
`data` and `net`. `sim` depends on nothing in the workspace except possibly
`data` types.

## Consequences

- `sim` can be benchmarked and tested without a window or a socket.
- A new crate needs a new ADR.
