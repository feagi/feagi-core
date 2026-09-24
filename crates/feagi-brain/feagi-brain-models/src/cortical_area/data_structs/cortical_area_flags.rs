use feagi_macros::bit_struct_builder;


bit_struct_builder! {
    u8,
    ExampleFlags,
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
    /// stuff visibility and enable flags.
    {
        enabled,
        visible,
    }
}