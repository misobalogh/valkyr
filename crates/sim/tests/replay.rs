//! Scripted scenario: record, rerun, compare final state hashes.

use sim::command::Command;
use sim::entity::{EntityId, Position};
use sim::replay::{Recorder, Replay};
use sim::time::Tick;

const SEED: u64 = 0x005E_ED0F_FA11;
const GOLDEN_HASH: u64 = 0x260dcf956763fd07;

fn record_scenario() -> Replay {
    let mut recorder = Recorder::new(SEED);
    let spawns: Vec<Command> = (0..8)
        .map(|i| Command::Spawn {
            position: Position { x: i * 1_000, y: 0 },
        })
        .collect();
    recorder.step(&spawns);

    for tick in 1..600_u64 {
        let target = EntityId::new(tick % 8);
        let mut commands = vec![Command::Scatter {
            id: target,
            radius: 250,
        }];
        if tick % 60 == 0 {
            commands.push(Command::Move {
                id: target,
                dx: 5_000,
                dy: -2_000,
            });
        }
        if tick == 300 {
            commands.push(Command::Despawn {
                id: EntityId::new(3),
            });
        }
        recorder.step(&commands);
    }
    recorder.finish()
}

#[test]
fn rerun_reproduces_the_recorded_final_hash() {
    let replay = record_scenario();
    let world = replay.run().unwrap();
    assert_eq!(world.tick(), Tick(600));
    assert_eq!(world.entity_count(), 7);
}

#[test]
fn scenario_matches_golden_hash() {
    assert_eq!(
        record_scenario().final_hash(),
        GOLDEN_HASH,
        "scenario hash changed: explain the change before updating the golden value"
    );
}
