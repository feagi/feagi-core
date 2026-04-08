use feagi_structures::define_bit_packed_u8_flags;

/// Stores various synapse boolean flags under a single byte. Meant to be used for all
/// synapse types
define_bit_packed_u8_flags! {
    pub struct SynapseFlag(
        valid,
        inhibitory,
        reserved_2,
        reserved_3,
        reserved_4,
        reserved_5,
        reserved_6,
        reserved_7,
    )
}



