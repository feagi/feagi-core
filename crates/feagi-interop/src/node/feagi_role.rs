
/// A single role a node can fulfill. The discriminant of each variant is the bit index it
/// occupies within [FeagiRoles]. Roles are split into 4 groups of 4 bits, where any bit in a
/// group being set implies the node fulfills that group as a whole. Skipped bits are reserved.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeagiRole {

    // Feagi Brain (bits 0-3)
    // Acts as a Feagi Brain to process neurons

    // reserve bit 0
    /// Brain produces visualization data
    FeagiBrainVisualizationGeneration = 1,
    /// Embodiments can connect to the brain, send sensor / receive motor commands
    FeagiBrainEmbodimentEndpoint = 2,
    /// Brain is editable, developers can modify its genome and connectome
    FeagiBrainDesignerMode = 3,

    // Feagi Viewer (bits 4-7)
    // Can connect to a Feagi Brain to view the genome structure within it

    /// Can visualize neuronal activations in Feagi Brains with Visualization Enabled
    FeagiViewerNeuronVisualizer = 4,
    /// Can send edit requests to a Feagi Brain to edit its genome
    FeagiViewerEditor = 5,
    // Bit 6 reserved
    // Bit 7 reserved

    // Feagi Embodiment (bits 8-11)
    // Acts as or part of an embodiment, or a representation of an entity (such as a robot or agent)

    /// Pipeline Stages of this embodiment can be remotely reconfigured
    FeagiEmbodimentDynamicPipeline = 8,
    /// Embodiment will only register for a specific Embodiment ID and a specific role within it.
    FeagiEmbodimentFixedEmbodimentOnly = 9,
    // Bit 10 reserved
    // Bit 11 reserved

    // TBA (bits 12-15)
    // Bit 12 reserved
    // Bit 13 reserved
    // Bit 14 reserved
    // Bit 15 reserved
}

impl FeagiRole {
    /// The bit this role occupies within a [FeagiRoles]
    pub const fn bit_mask(self) -> u16 {
        1 << (self as u8)
    }
}

/// Bitfield of every [FeagiRole] a node fulfills, serializable as a u16 / 2 bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FeagiRoles(u16);

impl FeagiRoles {
    const FEAGI_BRAIN_MASK: u16 = 0x000F;
    const FEAGI_VIEWER_MASK: u16 = 0x00F0;
    const FEAGI_EMBODIMENT_MASK: u16 = 0x0F00;
    const TBA_MASK: u16 = 0xF000;

    /// Creates a new instance with no roles set
    pub const fn new() -> Self {
        Self(0)
    }

    /// Wraps raw bits. Reserved bits are kept as-is so roles from newer versions are not lost
    pub const fn from_u16(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn as_u16(&self) -> u16 {
        self.0
    }

    pub const fn from_be_bytes(bytes: [u8; 2]) -> Self {
        Self(u16::from_be_bytes(bytes))
    }

    pub const fn to_be_bytes(&self) -> [u8; 2] {
        self.0.to_be_bytes()
    }

    pub const fn get_role(&self, role: FeagiRole) -> bool {
        self.0 & role.bit_mask() != 0
    }

    pub fn set_role(&mut self, role: FeagiRole, enabled: bool) {
        if enabled {
            self.0 |= role.bit_mask();
        } else {
            self.0 &= !role.bit_mask();
        }
    }

    /// True if any Feagi Brain role is set
    pub const fn is_feagi_brain(&self) -> bool {
        self.0 & Self::FEAGI_BRAIN_MASK != 0
    }

    /// True if any Feagi Viewer role is set
    pub const fn is_feagi_viewer(&self) -> bool {
        self.0 & Self::FEAGI_VIEWER_MASK != 0
    }

    /// True if any Feagi Embodiment role is set
    pub const fn is_feagi_embodiment(&self) -> bool {
        self.0 & Self::FEAGI_EMBODIMENT_MASK != 0
    }

    /// True if any bit in the TBA group is set
    pub const fn is_tba(&self) -> bool {
        self.0 & Self::TBA_MASK != 0
    }
}

impl From<u16> for FeagiRoles {
    fn from(bits: u16) -> Self {
        Self::from_u16(bits)
    }
}

impl From<FeagiRoles> for u16 {
    fn from(roles: FeagiRoles) -> Self {
        roles.as_u16()
    }
}