
pub trait CorticalTypeDefaults {
    fn default_cortical_area_writer(&self) -> ();//CorticalWriterByModelQuant;
}

/*

/// Cortical Types with default generation
pub enum WCorticalType {
    Core(WCoreCorticalType)
}

impl CorticalTypeDefaults for WCorticalType {
    fn default_cortical_area_writer(&self) -> ()//CorticalWriterByModelQuant {
        match self {
            WCorticalType::Core(core_type) => {
                core_type.default_cortical_area_writer()
            }
        }
    }
}



pub enum WCoreCorticalType {
    Power,
    Death
}

impl CorticalTypeDefaults for WCoreCorticalType {
    fn default_cortical_area_writer(&self) -> ()/*CorticalWriterByModelQuant*/ {
        match self {
            WCoreCorticalType::Power => {
                CorticalWriterByModelQuant::FeagiAdvanced(
                    FeagiAdvancedModelWriter::Standard(
                        FeagiAdvancedModelCorticalWriter::DefaultNewDimensional
                        { dimensions: DimensionalCorticalArea4DDimensions::new_from_usizes_unchecked(1,1,1,1),
                            _p: core::marker::PhantomData,
                        }
                    )
                )
            }
            WCoreCorticalType::Death => {
                CorticalWriterByModelQuant::FeagiAdvanced(
                    FeagiAdvancedModelWriter::Standard(
                        FeagiAdvancedModelCorticalWriter::DefaultNewDimensional
                        { dimensions: DimensionalCorticalArea4DDimensions::new_from_usizes_unchecked(1,1,1,1),
                            _p: core::marker::PhantomData,
                        }
                    )
                )
            }
        }
    }
}

 */