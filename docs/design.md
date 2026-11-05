# Design

Skeleton. Sections are filled in as the matching milestone starts.
Early thoughts and open questions live in [ideas.md](ideas.md).

## Pillars

1. Movement: very fast and fluid (sprint, slide, wall-run, glide, mounted flight).
2. Combat: weighty melee, stagger, stealth kills, ranged, shield/weapon combos.
3. Builds: no classes. Identity comes from armor sets, upgrades, race and crafted gear.
4. Survival and base: gathering, crafting, building, tamed mounts, wave defense.
5. Dungeons and caves: handcrafted and procedural, loot-heavy.

## Non-goals

MMO scale, PvP balance, mobile, microtransactions.

## Systems

- Stats, items and loadout
- Crafting and economy
- Movement and combat
- Networking
- Mounts and creatures
- World, building and defense
- Dungeons and caves
- Content pipeline and tooling

## Implemented so far (sim)

- Modifiers evaluate in a fixed order: flat, then summed percent, then each
  multiplicative percent. Collection order never matters.
- Builds are computed from race, equipped items, item upgrade level, rolled
  affixes and set bonuses. There are no classes.
- Crafting costs roll up through the recipe graph to raw materials.
- Combat resolves damage types against per-type resistances. Poise damage that
  empties the bar staggers the target. Damage-over-time effects are one per
  damage type.

