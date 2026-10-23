use feagi_basis::feagi_basis_proc_common::bit_struct_builder;

bit_struct_builder! {
    u8,
    pub FeagiNodeState,
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// Defines some general state information for a given node not relevant to a specific role
    {
        is_embedded_device
    }
}

// TODO 7 other states