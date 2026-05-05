use feagi_structures::base_feagi_types::::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use crate::cortical_area_identifier_flag::NPUCorticalAreaIdentifierFlag;
use crate::quantizables::NPUNeuronIndex;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct CorticalTypedNeuronIndex<T: QuantizableUIntType> {
    pub index: NPUNeuronIndex<T>,
    pub cortical_type: NPUCorticalAreaIdentifierFlag
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct CorticalTypedCorticalIndex<T: QuantizableUIntType> {
    pub index: CorticalAreaIndex<T>,
    pub cortical_type: NPUCorticalAreaIdentifierFlag
}