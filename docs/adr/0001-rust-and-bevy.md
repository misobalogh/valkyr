# 0001. Rust, with Bevy for the client

Status: accepted

## Context

The game needs a fast, memory-safe language, a single codebase for client and
server, and an engine that does not force its own simulation model on us.

## Decision

- The whole project is written in Rust.
- The client uses Bevy for rendering, input, UI and audio.
- The server is a headless binary. It does not depend on Bevy.
- Game rules live in the `sim` crate, which has no engine dependency. Bevy is
  only a presentation layer on top of it.

The exact Bevy version is recorded when the dependency is first added, together
with the reasons for it.

## Consequences

- Bevy releases often break APIs, so the client must stay thin to keep upgrades
  cheap.
- The server and tests never compile rendering code.
