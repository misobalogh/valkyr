//! Commands are the only way to change the world.

use crate::entity::{EntityId, Position};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Spawn { position: Position },
    Despawn { id: EntityId },
    Move { id: EntityId, dx: i64, dy: i64 },
}
