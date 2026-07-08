use feagi_npu_common::wrapped_indexes::NeuronMPIndexedVector;

// TODO add other quantization levels
/// Creates a set of vectors for each decimal quantized type
macro_rules! decimal_wrapped_vector_quant_group {
    ($(#[$meta:meta])*
    $StructName:ident,
    $WrappedQuantVector:ident,
    $QuantizationIndex:ident) => {
        $(#[$meta])*
        struct $StructName<FGQ: FeagiGlobalQuantization>
        {
            pub float_32: $WrappedQuantVector <FGQ::$QuantizationIndex ,f32>,
        }
    };
}

decimal_wrapped_vector_quant_group!(
    /// MP Grouped Neuron Potential Values
    MPQuantMembranePotentials,
    NeuronMPIndexedVector,
    NeuronIndexCountQuant
);