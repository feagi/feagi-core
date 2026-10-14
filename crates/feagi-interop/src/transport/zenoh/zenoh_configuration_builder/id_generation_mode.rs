/// How a Zenoh ID is picked on start. The id must be unique on a network.
#[derive(Clone, Copy, Debug, Hash, Default)]
pub enum ZenohIDGenerationMode {
    /// The `id` key is omitted and Zenoh draws a random 128-bit id.
    #[default]
    RandomOnStart,
    /// Encoded as the lowercase hexadecimal `id` string, without leading zeros.
    Specific(u16),
}

impl ZenohIDGenerationMode {
    /// Hexadecimal `id` text, or `None` when Zenoh should draw one.
    pub fn json_id(self) -> Option<String> {
        match self {
            Self::RandomOnStart => None,
            Self::Specific(id) => Some(format!("{id:x}")),
        }
    }
}
