// TODO ????

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub enum InterconnectCorticalType {
    #[default]
    LeakyIntegrateFire,
}

impl core::fmt::Display for InterconnectCorticalType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LeakyIntegrateFire => write!(f, "Leaky IntegrateFire"),
        }
    }
}
