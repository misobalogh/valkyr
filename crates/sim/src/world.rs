//! The world and its single entry point, `World::step`.

use std::collections::BTreeMap;
use std::num::NonZeroU64;

use crate::command::Command;
use crate::entity::{Entity, EntityId, Position};
use crate::event::{Event, Rejection};
use crate::hash::StateHasher;
use crate::rng::Rng;
use crate::time::Tick;

#[derive(Debug, Clone)]
pub struct World {
    tick: Tick,
    rng: Rng,
    next_id: u64,
    entities: BTreeMap<EntityId, Entity>,
}

impl World {
    pub fn new(seed: u64) -> World {
        World {
            tick: Tick::ZERO,
            rng: Rng::new(seed),
            next_id: 0,
            entities: BTreeMap::new(),
        }
    }

    pub fn tick(&self) -> Tick {
        self.tick
    }

    pub fn entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Hash over the observable world state. Equal hashes mean equal states.
    pub fn state_hash(&self) -> u64 {
        let mut hasher = StateHasher::new();
        hasher.write_u64(self.tick.0);
        for entity in self.entities.values() {
            hasher.write_u64(entity.id.raw());
            hasher.write_i64(entity.position.x);
            hasher.write_i64(entity.position.y);
        }
        hasher.finish()
    }

    /// Applies the commands in order, advances one tick and returns one event
    /// per command.
    pub fn step(&mut self, commands: &[Command]) -> Vec<Event> {
        let events = commands.iter().map(|command| self.apply(command)).collect();
        self.tick = self.tick.next();
        events
    }

    fn apply(&mut self, command: &Command) -> Event {
        match command {
            Command::Spawn { position } => self.spawn(*position),
            Command::Despawn { id } => self.despawn(*id),
            Command::Move { id, dx, dy } => self.move_entity(*id, *dx, *dy),
            Command::Scatter { id, radius } => self.scatter(*id, *radius),
        }
    }

    fn spawn(&mut self, position: Position) -> Event {
        let Some(next_id) = self.next_id.checked_add(1) else {
            return Event::Rejected(Rejection::IdSpaceExhausted);
        };
        let id = EntityId::new(self.next_id);
        self.next_id = next_id;
        self.entities.insert(id, Entity { id, position });
        Event::Spawned { id, position }
    }

    fn despawn(&mut self, id: EntityId) -> Event {
        match self.entities.remove(&id) {
            Some(_) => Event::Despawned { id },
            None => Event::Rejected(Rejection::UnknownEntity(id)),
        }
    }

    fn scatter(&mut self, id: EntityId, radius: u32) -> Event {
        if !self.entities.contains_key(&id) {
            return Event::Rejected(Rejection::UnknownEntity(id));
        }
        let radius = i64::from(radius);
        let span = NonZeroU64::MIN.saturating_add(radius.unsigned_abs() * 2);
        let dx = self.rng.next_below(span) as i64 - radius;
        let dy = self.rng.next_below(span) as i64 - radius;
        self.move_entity(id, dx, dy)
    }

    fn move_entity(&mut self, id: EntityId, dx: i64, dy: i64) -> Event {
        let Some(entity) = self.entities.get_mut(&id) else {
            return Event::Rejected(Rejection::UnknownEntity(id));
        };
        let Some(position) = entity.position.checked_offset(dx, dy) else {
            return Event::Rejected(Rejection::PositionOverflow(id));
        };
        entity.position = position;
        Event::Moved { id, position }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGIN: Position = Position { x: 0, y: 0 };

    fn spawn_one(world: &mut World) -> EntityId {
        match world
            .step(&[Command::Spawn { position: ORIGIN }])
            .as_slice()
        {
            [Event::Spawned { id, .. }] => *id,
            other => panic!("unexpected events: {other:?}"),
        }
    }

    #[test]
    fn step_advances_tick_even_without_commands() {
        let mut world = World::new(1);
        world.step(&[]);
        world.step(&[]);
        assert_eq!(world.tick(), Tick(2));
    }

    #[test]
    fn spawn_creates_entity_with_fresh_ids() {
        let mut world = World::new(1);
        let first = spawn_one(&mut world);
        let second = spawn_one(&mut world);
        assert_ne!(first, second);
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn despawned_ids_are_not_reused() {
        let mut world = World::new(1);
        let first = spawn_one(&mut world);
        world.step(&[Command::Despawn { id: first }]);
        let second = spawn_one(&mut world);
        assert_ne!(first, second);
    }

    #[test]
    fn move_updates_position() {
        let mut world = World::new(1);
        let id = spawn_one(&mut world);
        let events = world.step(&[Command::Move { id, dx: 5, dy: -3 }]);
        let expected = Position { x: 5, y: -3 };
        assert_eq!(
            events,
            vec![Event::Moved {
                id,
                position: expected
            }]
        );
        assert_eq!(world.entity(id).map(|e| e.position), Some(expected));
    }

    #[test]
    fn unknown_entity_is_rejected() {
        let mut world = World::new(1);
        let ghost = EntityId::new(99);
        let events = world.step(&[Command::Despawn { id: ghost }]);
        assert_eq!(
            events,
            vec![Event::Rejected(Rejection::UnknownEntity(ghost))]
        );
    }

    #[test]
    fn overflowing_move_is_rejected_and_keeps_position() {
        let mut world = World::new(1);
        let id = spawn_one(&mut world);
        world.step(&[Command::Move {
            id,
            dx: i64::MAX,
            dy: 0,
        }]);
        let events = world.step(&[Command::Move { id, dx: 1, dy: 0 }]);
        assert_eq!(
            events,
            vec![Event::Rejected(Rejection::PositionOverflow(id))]
        );
        assert_eq!(world.entity(id).map(|e| e.position.x), Some(i64::MAX));
    }

    #[test]
    fn scatter_stays_within_radius() {
        let mut world = World::new(5);
        let id = spawn_one(&mut world);
        for _ in 0..200 {
            world.step(&[Command::Scatter { id, radius: 0 }]);
        }
        assert_eq!(world.entity(id).map(|e| e.position), Some(ORIGIN));
        let events = world.step(&[Command::Scatter { id, radius: 10 }]);
        let Some(Event::Moved { position, .. }) = events.first() else {
            panic!("unexpected events: {events:?}");
        };
        assert!(position.x.abs() <= 10 && position.y.abs() <= 10);
    }

    #[test]
    fn hash_changes_when_an_entity_moves() {
        let mut world = World::new(1);
        let id = spawn_one(&mut world);
        let before = world.state_hash();
        world.step(&[Command::Move { id, dx: 1, dy: 0 }]);
        assert_ne!(world.state_hash(), before);
    }

    #[test]
    fn rejected_scatter_does_not_consume_randomness() {
        let mut world = World::new(5);
        let before = world.rng.clone();
        let ghost = EntityId::new(3);
        let events = world.step(&[Command::Scatter {
            id: ghost,
            radius: 4,
        }]);
        assert_eq!(
            events,
            vec![Event::Rejected(Rejection::UnknownEntity(ghost))]
        );
        assert_eq!(world.rng, before);
    }

    #[test]
    fn commands_apply_in_order_within_one_step() {
        let mut world = World::new(1);
        let id = spawn_one(&mut world);
        let events = world.step(&[Command::Despawn { id }, Command::Move { id, dx: 1, dy: 1 }]);
        assert_eq!(events[1], Event::Rejected(Rejection::UnknownEntity(id)));
    }
}
