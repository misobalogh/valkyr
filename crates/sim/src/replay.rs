//! Replay recording: a seed plus the commands of every tick.

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
}

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
    fn final_hash_is_the_hash_of_the_recorded_world() {
        let mut recorder = Recorder::new(3);
        recorder.step(&[]);
        let expected = recorder.world().state_hash();
        assert_eq!(recorder.finish().final_hash(), expected);
    }
}
