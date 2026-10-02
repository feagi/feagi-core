use serde::{Deserialize, Serialize};
use feagi_basis::{FeagiBasisError, FeagiFailDataEtc};

pub type CorticalUnitIndex = private::CorticalUnitIndex<u16>;
pub type CorticalSubUnitIndex = private::CorticalSubUnitIndex<u8>;
pub type CorticalChannelIndex = private::CorticalChannelIndex<u32>;

mod private {
    use feagi_basis::prelude::create_wrapped_quantized_unsigned_integer;


    create_wrapped_quantized_unsigned_integer!(
        ///Index for grouping cortical units of the same type within a genome
        pub CorticalUnitIndex
    );

    create_wrapped_quantized_unsigned_integer!(
        ///Index for cortical areas within a cortical unit. This allows easy identification of various
        /// cortical areas (which can be called CorticalSubUnits in this case) within a cortical unit
        pub CorticalSubUnitIndex
    );

    create_wrapped_quantized_unsigned_integer!(
        /// Index for addressing specific channels within an I/O cortical area
        pub CorticalChannelIndex
    );
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FrameChangeHandling {
    #[default]
    Absolute,
    Incremental,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PercentageNeuronPositioning {
    Linear,
    #[default]
    Fractional,
}