use feagi_basis::feagi_basis_proc_common::bit_struct_builder;

bit_struct_builder! {
    u8,
    pub CorticalAreaPropertyFlags,
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
    /// Cortical Area Level Boolean Flags
    {
        is_psp_uniform,
        is_psp_using_membrane_potential,
    }
}