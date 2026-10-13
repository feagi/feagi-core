use base64::Engine;


/// Unique session tracker of an agent registered to FEAGI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentID([u8; Self::NUMBER_OF_BYTES]);

impl AgentID {

    /// Number of bytes allocated for this ID
    pub const NUMBER_OF_BYTES: usize = 8;

}




pub struct AgentIDB64