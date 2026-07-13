/// Implemented for all sub enums of `NeuronModelDescriptor`, to make it easier to organize them.
pub trait NeuronModelQuantizationLevel: Clone + Copy {
    /// The index of the model, range 0-31  (inclusive). Make sure it does not conflict with other models
    const MODEL_INDEX: u8;

    /// The index of the quant level, range 0-7 (inclusive) should encode for this enum. Return it
    /// given the bits are matching. Note that unsafe code is used, so invalid bytes will result
    /// in undefined behavior!
    unsafe fn get_quant_enum_from_quant_bits(quant_bits: u8) -> Self;
}
