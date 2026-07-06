use feagi_data::create_wrapped_index;

create_wrapped_index!(
    /// Denotes the index for grouping cortical_area units of the same type within a genome.
    pub CorticalUnitIndex, u8
);

create_wrapped_index!(
    /// Denotes the index for a cortical area within a cortical unit
    pub CorticalSubUnitIndex, u8
);

