//! Incremental FNV-1a 64-bit hasher used for world state hashes.

const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Debug, Clone)]
pub struct StateHasher {
    state: u64,
}

impl StateHasher {
    pub fn new() -> StateHasher {
        StateHasher {
            state: OFFSET_BASIS,
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state ^= u64::from(*byte);
            self.state = self.state.wrapping_mul(PRIME);
        }
    }

    pub fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    pub fn write_i64(&mut self, value: i64) {
        self.write_bytes(&value.to_le_bytes());
    }

    pub fn finish(&self) -> u64 {
        self.state
    }
}

impl Default for StateHasher {
    fn default() -> StateHasher {
        StateHasher::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_offset_basis() {
        assert_eq!(StateHasher::new().finish(), 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    fn matches_reference_vector_for_a() {
        let mut hasher = StateHasher::new();
        hasher.write_bytes(b"a");
        assert_eq!(hasher.finish(), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn write_order_matters() {
        let mut ab = StateHasher::new();
        ab.write_u64(1);
        ab.write_u64(2);
        let mut ba = StateHasher::new();
        ba.write_u64(2);
        ba.write_u64(1);
        assert_ne!(ab.finish(), ba.finish());
    }

    #[test]
    fn signed_values_hash_differently_from_zero() {
        let mut hasher = StateHasher::new();
        hasher.write_i64(-1);
        assert_ne!(hasher.finish(), StateHasher::new().finish());
    }
}
