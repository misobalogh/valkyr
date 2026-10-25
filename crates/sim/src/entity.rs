//! Entity identity and the data every entity carries.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(u64);

impl EntityId {
    pub fn new(raw: u64) -> EntityId {
        EntityId(raw)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

/// Position in millimeters. Integers keep results identical on every platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub fn checked_offset(self, dx: i64, dy: i64) -> Option<Position> {
        Some(Position {
            x: self.x.checked_add(dx)?,
            y: self.y.checked_add(dy)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub id: EntityId,
    pub position: Position,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_offset_moves_position() {
        let start = Position { x: 10, y: -5 };
        assert_eq!(start.checked_offset(3, 5), Some(Position { x: 13, y: 0 }));
    }

    #[test]
    fn checked_offset_detects_overflow() {
        let start = Position { x: i64::MAX, y: 0 };
        assert_eq!(start.checked_offset(1, 0), None);
    }

    #[test]
    fn ids_order_by_raw_value() {
        assert!(EntityId::new(1) < EntityId::new(2));
    }
}
