# valkyr

Co-op action-survival RPG set in a Norse-inspired world. Written in Rust.
Up to about 8 players per server. Class-less builds defined by armor,
upgrades and race.

## Status

M0 is done. M1 (simulation core) is implemented without external dependencies:
fixed tick, seeded RNG, entity model, command/event step, state hash and an
in-memory replay recorder and runner. Replay files on disk, property tests and
criterion benchmarks wait for the dependency decisions (serialization format,
proptest, criterion).

## Layout

- `crates/sim`: deterministic game rules
- `crates/data`: content schemas, loaders, validators
- `crates/net`: protocol, snapshots, prediction
- `crates/server`: headless authoritative server
- `crates/client`: Bevy client
- `crates/tools`: validation, balance sim, replay runner
- `content/`: gameplay data
- `docs/`: design doc, ADRs, performance budgets

## Milestones

| Milestone | Scope                                         |
|-----------|-----------------------------------------------|
| M0        | Foundations: workspace, CI, ADRs, docs        |
| M1        | Simulation core: tick loop, RNG, hash, replay |
| M2        | Stats, items, loadout                         |
| M3        | Crafting and economy                          |
| M4        | Movement and combat in sim                    |
| M5        | Net foundation                                |
| M6        | Playable vertical slice                       |
| M7        | Mounts and creatures                          |
| M8        | World, building, defense                      |
| M9        | Dungeons and caves                            |
| M10       | Content pipeline and tooling                  |
