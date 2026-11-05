# valkyr

Co-op action-survival RPG set in a Norse-inspired world. Written in Rust.
Up to about 8 players per server. Class-less builds defined by armor,
upgrades and race.

## Status

- M0 Foundations: done.
- M1 Simulation core: done. Fixed tick, seeded RNG, entity model, command/event
  step, state hash, in-memory replay recorder and runner.
- M2 Stats, items, loadout: modifier pipeline, equipment slots, set bonuses,
  upgrade tracks, race modifiers, affix rolling. In progress: content loading.
- M3 Crafting and economy: recipe graph with cycle detection and raw cost,
  weight-limited inventory, loot tables with rarity tiers.
- M4 Movement and combat: controller (sprint, slide, wall-run, glide, jump),
  hit detection, damage types, poise and stagger, status effects. These are
  pure modules for now and are not wired into `World` yet.

Everything so far uses no external dependencies. Content loading (RON), replay
files on disk, property tests and criterion benchmarks wait for the dependency
decisions.

## Try it

Two text demos, no graphics yet. One line is one simulation tick.

    cargo run -p tools -- move   # sprint, slide, jump, glide
    cargo run -p tools -- wolf   # sword, fire, frost against a frost wolf

Example for `move`: `10 f sprint`, then `slide f`, then `jump f sprint`, then
`30 f glide`. Type `q` to quit.

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
