use serde::{Deserialize, Serialize};

/// Used to identify a connected client to the server. A random identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionID {
    bytes: [u8; SessionID::NUMBER_BYTES],
}

impl SessionID {
    pub const NUMBER_BYTES: usize = 8;

    pub fn new(bytes: [u8; SessionID::NUMBER_BYTES]) -> Self {
        Self { bytes }
    }

    pub const fn new_null() -> Self {
        Self { bytes: [0; SessionID::NUMBER_BYTES] }
    }

    pub fn new_random() -> Self {
        let mut bytes = [0u8; SessionID::NUMBER_BYTES];
        getrandom::getrandom(&mut bytes).expect("Failed to generate random bytes");
        Self { bytes }
    }

    pub fn is_blank(&self) -> bool {
        self.bytes == [0; SessionID::NUMBER_BYTES]
    }

    pub fn bytes(&self) -> &[u8; SessionID::NUMBER_BYTES] {
        &self.bytes
    }
}