# 0005. Command and event pipeline

Status: accepted

## Context

Tests, replays and netcode should not each invent their own way to drive the
game.

## Decision

Everything goes through one path:

    Input -> Command -> Sim step -> State + Events

- Input is turned into commands outside of `sim`.
- `sim` consumes commands per tick and returns events.
- Invalid commands are rejected with an event, not with a panic.
- Replays are recorded commands plus a seed.

## Consequences

- Netcode sends commands and snapshots, never calls into sim internals.
- Any bug report can be reproduced from a seed and a command list.
