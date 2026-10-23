use feagi_basis::impl_byte_array_id;

//region Feagi Roles

pub enum FeagiRole {
    /// Processes neuron activity
    FeagiBrainNeuronProcessor,
    /// Brain produces visualization data
    FeagiBrainVisualizationGeneration,
    /// Embodiments can connect to the brain, send sensor / receive motor commands
    FeagiBrainEmbodimentEndpoint,
    /// Brain is editable, developers can modify its genome and connectome
    FeagiBrainDesignerMode,

    /// Can connect to a Feagi Brain to view the genome structure within it
    FeagiViewer,
    /// Can visualize neuronal activations in Feagi Brains with Visualization Enabled
    FeagiViewerNeuronVisualizer,
    /// Can send edit requests to a Feagi Brain to edit its genome
    FeagiViewerEditor,
    Reserved2,

    /// Acts as or part of an embodiment, or a representation of an entity (such as a robot or agent)
    FeagiEmbodiment,
    /// Pipeline Stages of this embodiment can be remotely reconfigured
    FeagiEmbodimentDynamicPipeline,
    /// Embodiment will only register for a specific Embodiment ID and a specific role within it.
    FeagiEmbodimentFixedEmbodimentOnly,
    Reserved3,

    Reserved4,
    Reserved5,
    Reserved6,
    Reserved7,


}





//endregion


pub const NUMBER_OF_BYTES_NODE_ID: usize = 8;

impl_byte_array_id!(
    /// Unique session tracker of a node tracked by FEAGI (generally for Zenoh)
    pub FeagiNodeID,
    NUMBER_OF_BYTES_NODE_ID
);


/// The version of a given node
pub type FeagiNodeVersion = u16;
pub const NUMBER_OF_BYTES_NODE_FLAGS: usize = 6;

/// What node version the node with this code compiled will report itself as
pub const CURRENT_FEAGI_NODE_VERSION: FeagiNodeVersion = 0;

/// If nodes below (exclusive) this version are detected in the network, this node will refuse to start
pub const MINIMUM_SUPPORTED_INTERFACE_VERSION: FeagiNodeVersion = 0;

/// Denotes the version, state, and roles this given node supports
pub struct FeagiNodeFlags([u8; NUMBER_OF_BYTES_NODE_FLAGS]);

impl FeagiNodeFlags {

    // Bytes 0, 1 specify version
    // Bytes 2, 3 specify roles
    // Bytes 4 specify general device state
    // Byte 5 Reserved

    /// Creates a new struct with the current version
    pub fn new() -> Self {
        let mut out = [0u8; 6];
        let bytes: [u8; 2] = CURRENT_FEAGI_NODE_VERSION.to_be_bytes();
        out[0] = bytes[0];
        out[1] = bytes[1];
        Self(out)
    }

    pub fn get_version(&self) -> FeagiNodeVersion {
        let slice = [self.0[0], self.0[1]];
        u16::from_be_bytes(slice)
    }

    // cannot "set" version, thats defined at compile time




}

impl Default for FeagiNodeFlags {
    fn default() -> Self {
        Self::new()
    }
}