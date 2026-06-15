use core::marker::PhantomData;
use feagi_structures::feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::cortical_area_layout::{CorticalLayoutBase, CorticalLayoutDimensional};
use crate::neural_processing_unit_data_structures::wrappers::NPUWrappedCorticalAreaDimensions;


pub struct CorticalLayouts<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{
    pub dimensional: Vec<CorticalLayoutDimensionalCPU<FGQ>>,
}


pub struct CorticalLayoutDimensionalCPU<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{
    pub dimensions: NPUWrappedCorticalAreaDimensions<FGQ::NeuronIndexCountQuant>,
}

impl<FGQ> CorticalLayoutBase<FGQ> for CorticalLayoutDimensionalCPU<FGQ>
where FGQ: FeagiGlobalQuantization, {}

impl<FGQ> CorticalLayoutDimensional<FGQ> for CorticalLayoutDimensionalCPU<FGQ>
where FGQ: FeagiGlobalQuantization, {}

// TODO other types?