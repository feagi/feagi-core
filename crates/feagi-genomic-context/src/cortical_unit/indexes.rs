/// Denotes the index for grouping cortical_area units of the same type within a genome.
pub type CorticalUnitIndex = generics::CorticalUnitIndexQuant<u8>;

/// Denotes the index for a cortical area within a cortical unit
pub type CorticalSubUnitIndex = generics::CorticalSubUnitIndexQuant<u8>;

pub mod generics {
    use feagi_data::create_wrapped_quantized_unsigned_integer;

    create_wrapped_quantized_unsigned_integer!(
        /// Denotes the index for grouping cortical_area units of the same type within a genome.
        pub CorticalUnitIndexQuant
    );

    create_wrapped_quantized_unsigned_integer!(
        /// Denotes the index for a cortical area within a cortical unit
        pub CorticalSubUnitIndexQuant
    );
}
