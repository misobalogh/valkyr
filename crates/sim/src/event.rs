//! Events describe what a step did. Every command yields exactly one event.

use crate::entity::{EntityId, Position};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    UnknownEntity(EntityId),
    PositionOverflow(EntityId),
    IdSpaceExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Spawned { id: EntityId, position: Position },
    Despawned { id: EntityId },
    Moved { id: EntityId, position: Position },
    Rejected(Rejection),
}
