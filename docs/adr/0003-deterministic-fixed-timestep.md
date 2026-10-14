# 0003. Deterministic fixed-timestep simulation

Status: accepted

## Context

Prediction, reconciliation, replays and tests all need to rerun the same
inputs and get the same state.

## Decision

- The simulation advances in fixed ticks (60 per second to start).
- Sim code uses no wall-clock time, no unseeded randomness and no `HashMap`
  iteration order. Ordered collections only.
- Randomness comes from a seeded RNG owned by the world state.
- Floats are avoided in sim state unless a documented reason says the result
  cannot differ between platforms.
- A state hash over the whole world is the check for determinism.

## Consequences

- Every sim feature must be testable by stepping the world headlessly.
- A changed replay hash is a failure until explained.
