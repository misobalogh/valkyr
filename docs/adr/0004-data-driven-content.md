# 0004. Gameplay content is data

Status: accepted

## Context

Items, enemies, recipes and stats change constantly during balancing.

## Decision

- All gameplay content lives in `content/` as RON (or TOML) files.
- The `data` crate defines schemas, loads files and validates them at startup.
- Code never hardcodes an item, enemy, recipe or stat.
- Content is referenced by validated IDs.

## Consequences

- Invalid content fails at startup or in CI (`tools validate`), not in the
  middle of a run.
- Balance tooling and modding work on the same files the game loads.
