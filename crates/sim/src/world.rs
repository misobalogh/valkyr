//! The world and its single entry point, `World::step`.

use std::collections::BTreeMap;

use crate::command::Command;
use crate::entity::{Entity, EntityId, Position};
use crate::event::{Event, Rejection};
use crate::time::Tick;

#[derive(Debug, Clone, Default)]
pub struct World {
    tick: Tick,
    next_id: u64,
    entities: BTreeMap<EntityId, Entity>,
}

impl World {
    pub fn tick(&self) -> Tick {
        self.tick
    }

    pub fn entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
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
        let mut world = World::default();
        world.step(&[]);
        world.step(&[]);
        assert_eq!(world.tick(), Tick(2));
    }

    #[test]
    fn spawn_creates_entity_with_fresh_ids() {
        let mut world = World::default();
        let first = spawn_one(&mut world);
        let second = spawn_one(&mut world);
        assert_ne!(first, second);
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn despawned_ids_are_not_reused() {
        let mut world = World::default();
        let first = spawn_one(&mut world);
        world.step(&[Command::Despawn { id: first }]);
        let second = spawn_one(&mut world);
        assert_ne!(first, second);
    }

    #[test]
    fn move_updates_position() {
        let mut world = World::default();
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
        let mut world = World::default();
        let ghost = EntityId::new(99);
        let events = world.step(&[Command::Despawn { id: ghost }]);
        assert_eq!(
            events,
            vec![Event::Rejected(Rejection::UnknownEntity(ghost))]
        );
    }

    #[test]
    fn overflowing_move_is_rejected_and_keeps_position() {
        let mut world = World::default();
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
    fn commands_apply_in_order_within_one_step() {
        let mut world = World::default();
        let id = spawn_one(&mut world);
        let events = world.step(&[Command::Despawn { id }, Command::Move { id, dx: 1, dy: 1 }]);
        assert_eq!(events[1], Event::Rejected(Rejection::UnknownEntity(id)));
    }
}
