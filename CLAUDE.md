# valkyr

Co-op action-survival RPG, Norse-inspired. Rust, Bevy client, headless
authoritative server. Class-less builds defined by armor, upgrades and race.

## Architecture

- `crates/sim`: deterministic game rules. No rendering, no I/O, no wall-clock time.
- `crates/data`: content schemas, loaders, validators.
- `crates/net`: protocol, snapshots, prediction/reconciliation.
- `crates/server`: headless server, runs the same sim as the client.
- `crates/client`: Bevy rendering, input, UI, audio. Keep it thin.
- `crates/tools`: CLI (validate, balance sim, replay runner, save inspector).
- `content/`: all gameplay data (RON). `docs/adr/`: architecture decisions.

Flow: Input -> Command -> Sim step -> State + Events. Tests, replays and
netcode all use this one path.

## Hard rules

- Sim code must be deterministic: fixed timestep, seeded RNG only, no
  HashMap iteration order, no system time, no floats in places where
  platforms could differ without a documented reason.
- Gameplay content is data. Never hardcode an item, enemy, recipe or stat.
- Server is authoritative. Client predicts only the local player.
- Save format is versioned. Any change to it needs a migration and a test.
- No `unwrap`/`expect` in library code. Use typed errors (thiserror).
- No placeholder code, TODO stubs, speculative abstractions or unused code.
  Build only what the current milestone needs.
- Do not add a dependency without asking. State the reason and alternatives.
- Ask before changing any public type in `sim` or `net`.
- Architecture changes need a new ADR in `docs/adr/`. Do not silently
  contradict an existing one.

## Workflow

- Work in small steps. One concern per commit.
- Before every commit: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`. Do not commit if any of them fails.
- Conventional Commits: feat, fix, test, refactor, perf, docs, chore.
  Imperative mood, subject under 72 characters, body only when the why is
  not obvious.
- A bug gets its own `fix:` commit with a regression test that fails
  without the fix.
- Write the test first when the behavior is clear. Every sim feature needs a
  headless test. Use proptest for stat math, crafting graphs, inventory and
  save/load round trips.
- Replay tests: a scripted scenario must reproduce an identical final state
  hash. A changed hash is a failure until explained.
- Performance-sensitive sim code gets a criterion benchmark. Budgets are in
  `docs/perf.md`.
- At the end of each milestone, stop, summarize what changed and what is
  risky, and wait for approval.

## Code style

- Small functions, clear names, early returns.
- Doc comments only where the why is not obvious. No comments that restate code.
- Prefer plain data structs and free functions over trait hierarchies until a
  second implementation actually exists.
- Errors carry enough context to debug from a log line.

## Commands

- Test: `cargo test --workspace`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Format: `cargo fmt --all`
- Validate content: `cargo run -p tools -- validate`
- Balance sim: `cargo run -p tools -- sim`
- Bench: `cargo bench -p sim`

## Current milestone

M0 Foundations, then M1 Simulation core. Do not start later milestones
until told.
