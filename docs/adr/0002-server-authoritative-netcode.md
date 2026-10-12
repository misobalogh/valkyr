# 0002. Server-authoritative netcode

Status: accepted

## Context

Co-op play with about 8 players per server. Cheating between friends is a
minor concern, but a single source of truth keeps desyncs debuggable.

## Decision

- The server owns the game state.
- The client predicts and reconciles only the local player.
- All other entities are interpolated from snapshots.
- The server runs the same `sim` crate as the client.

## Consequences

- Prediction needs rollback and replay of local inputs, so `sim` steps must be
  cheap and side-effect free.
- Remote players always appear slightly in the past.
