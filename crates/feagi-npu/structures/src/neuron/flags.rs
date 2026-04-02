use feagi_structures::define_bit_packed_u8_flags;

/// Stores various neuron boolean flags under a single byte
define_bit_packed_u8_flags! {
    pub struct NeuronFlag(
        valid,
        reserved_1,
        reserved_2,
        reserved_3,
        reserved_4,
        reserved_5,
        reserved_6,
        reserved_7,
    )
}

// TODO double check this!

impl NeuronFlag {
    pub const fn new_valid() -> NeuronFlag {
        NeuronFlag::from_raw(0x01)
    }
}

/// Stores various interneuron cortical area boolean flags under a single byte
define_bit_packed_u8_flags! {
    pub struct InterneuronCorticalFlag(
        valid,
        mp_charge_accumulation_enabled,
        mp_driven_psp_enabled,
        reserved_3,
        reserved_4,
        reserved_5,
        reserved_6,
        reserved_7,
    )
}

// TODO double check this!

impl InterneuronCorticalFlag {
    pub const fn new_valid() -> InterneuronCorticalFlag {
        InterneuronCorticalFlag::from_raw(0x01)
    }
}
