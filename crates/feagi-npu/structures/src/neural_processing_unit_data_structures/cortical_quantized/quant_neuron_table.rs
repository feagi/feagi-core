use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, CorticalPotentialQuantizationFloat32, FeagiGlobalQuantization};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::tables::cortical_structure_configuration::cortical_configuration::CorticalConfigurationDimensionalCPUQuant;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::tables::typed_by_membrane_potential::input_fcl::quant_typed_collection::QuantTypedFCLInputPotentialCollectionCPU;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::tables::typed_by_membrane_potential::cpq_typed_membrane_potentials::CPQTypedMembranePotentialCollectionCPU;




pub struct NPUQuantTableCPU<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{
    pub quant_float_32: NPUQuantGroupCPU<FGQ, CorticalPotentialQuantizationFloat32>
}


pub struct NPUQuantGroupCPU<FGQ, CPQ>
where
    FGQ: FeagiGlobalQuantization,
    CPQ: CorticalPotentialQuantization
{
    pub fcl_data: QuantTypedFCLInputPotentialCollectionCPU<FGQ, CPQ>,
    pub neuron_potentials: CPQTypedMembranePotentialCollectionCPU<FGQ, CPQ>,
    // TODO nondimensional
    pub cortical_configuration_dimensional: CorticalConfigurationDimensionalCPUQuant<FGQ, CPQ, Self::CORTICAL_CONFIGURATION_PADDING>
}

impl<FGQ, CPQ> NPUQuantGroupCPU<FGQ, CPQ>
where
    FGQ: FeagiGlobalQuantization,
    CPQ: CorticalPotentialQuantization
{
    const CORTICAL_CONFIGURATION_PADDING: usize = CorticalConfigurationDimensionalCPUQuant::calculate_padding();
}



