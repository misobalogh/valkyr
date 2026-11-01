//! Replay recording: a seed plus the commands of every tick.

use std::fmt;

use crate::command::Command;
use crate::event::Event;
use crate::world::World;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replay {
    seed: u64,
    frames: Vec<Vec<Command>>,
    final_hash: u64,
}

impl Replay {
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Commands per tick, in tick order.
    pub fn frames(&self) -> &[Vec<Command>] {
        &self.frames
    }

    pub fn final_hash(&self) -> u64 {
        self.final_hash
    }

    /// Reruns the recorded commands on a fresh world and checks that the final
    /// state hash is the recorded one.
    pub fn run(&self) -> Result<World, ReplayError> {
        let mut world = World::new(self.seed);
        for commands in &self.frames {
            world.step(commands);
        }
        let actual = world.state_hash();
        if actual != self.final_hash {
            return Err(ReplayError::HashMismatch {
                expected: self.final_hash,
                actual,
            });
        }
        Ok(world)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    HashMismatch { expected: u64, actual: u64 },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplayError::HashMismatch { expected, actual } => write!(
                f,
                "replay final hash mismatch: expected {expected:#018x}, got {actual:#018x}"
            ),
        }
    }
}

impl std::error::Error for ReplayError {}

/// Steps a world while remembering every command it was given.
#[derive(Debug, Clone)]
pub struct Recorder {
    seed: u64,
    world: World,
    frames: Vec<Vec<Command>>,
}

impl Recorder {
    pub fn new(seed: u64) -> Recorder {
        Recorder {
            seed,
            world: World::new(seed),
            frames: Vec::new(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn step(&mut self, commands: &[Command]) -> Vec<Event> {
        self.frames.push(commands.to_vec());
        self.world.step(commands)
    }

    pub fn finish(self) -> Replay {
        Replay {
            seed: self.seed,
            final_hash: self.world.state_hash(),
            frames: self.frames,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Position;

    #[test]
    fn records_one_frame_per_step() {
        let mut recorder = Recorder::new(3);
        let spawn = Command::Spawn {
            position: Position { x: 1, y: 2 },
        };
        recorder.step(std::slice::from_ref(&spawn));
        recorder.step(&[]);
        let replay = recorder.finish();
        assert_eq!(replay.seed(), 3);
        assert_eq!(replay.frames(), &[vec![spawn], vec![]]);
    }

    #[test]
    fn run_reproduces_the_recorded_hash() {
        let mut recorder = Recorder::new(9);
        let id = match recorder
            .step(&[Command::Spawn {
                position: Position { x: 0, y: 0 },
            }])
            .as_slice()
        {
            [Event::Spawned { id, .. }] => *id,
            other => panic!("unexpected events: {other:?}"),
        };
        for _ in 0..20 {
            recorder.step(&[Command::Scatter { id, radius: 30 }]);
        }
        let replay = recorder.finish();
        let world = replay.run().unwrap();
        assert_eq!(world.state_hash(), replay.final_hash());
    }

    #[test]
    fn run_detects_a_tampered_hash() {
        let mut recorder = Recorder::new(9);
        recorder.step(&[]);
        let mut replay = recorder.finish();
        let expected = replay.final_hash ^ 1;
        replay.final_hash = expected;
        assert!(matches!(
            replay.run(),
            Err(ReplayError::HashMismatch { expected: e, .. }) if e == expected
        ));
    }

    #[test]
    fn final_hash_is_the_hash_of_the_recorded_world() {
        let mut recorder = Recorder::new(3);
        recorder.step(&[]);
        let expected = recorder.world().state_hash();
        assert_eq!(recorder.finish().final_hash(), expected);
    }
}
