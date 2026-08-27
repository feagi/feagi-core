use crate::cortical_area::components::neuron_layout::implementations::voxel::NeuronLayoutVoxel;
use crate::cortical_area::components::neuron_layout::neuron_layout_model::NeuronLayout;
use crate::cortical_area::parameters::body::dynamics::components::data::NullData;
use crate::cortical_area::parameters::body::dynamics::components::mp_driven_psp_configurability::MPDrivenPSPForcedOff;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::cortical_area::parameters::body::dynamics::cortical_area_dynamics::{CorticalAreaDynamics, NeuronDynamicsOutput};
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use crate::wrapped_indexes::BurstIndex;
use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, CorticalNeuronPotential};
use std::marker::PhantomData;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

/// Simply fires every burst lol
pub struct CorePowerCorticalAreaDynamics<NPUIQ, BEIQ, NL, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    NL: NeuronLayout<BEIQ>,
    CAMQ: CorticalAreaQuantization,
{
    _p: PhantomData<(NPUIQ, BEIQ, NL, CAMQ)>,
}

impl<NPUIQ, BEIQ, CAMQ> CorticalAreaDynamics<NPUIQ, BEIQ, NeuronLayoutVoxel<BEIQ>, CAMQ>
    for CorePowerCorticalAreaDynamics<NPUIQ, BEIQ, NeuronLayoutVoxel<BEIQ>, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    type MPDrivenPSPConfigurability = MPDrivenPSPForcedOff<CAMQ>;
    type CorticalDataProperties = NullData<CAMQ>;
    type CorticalDataInternal = NullData<CAMQ>;
    type CorticalDataShared = NullData<CAMQ>;
    type NeuronDataProperties = NullData<CAMQ>;
    type NeuronDataInternal = NullData<CAMQ>;
    
    const HAS_CORTICAL_DYNAMICS_PROCESSING: bool = false;
    const HAS_NEURON_DYNAMICS_PROCESSING: bool = true;

    fn process_cortical_dynamics(
        burst_index: &BurstIndex<NPUIQ::BurstIndexQuant>,
        cortical_properties: &mut Self::CorticalDataProperties,
        cortical_internal: &mut Self::CorticalDataInternal,
        cortical_shared: &mut Self::CorticalDataShared,
        layout_context: &NeuronLayoutVoxel<BEIQ>,
    ) -> () {
        
    }

    fn process_neuron_dynamics(
        _: &CorticalNeuronPotential<<CAMQ as MembranePotentialQuantization>::MembranePotentialQuant>,
        _: &BurstIndex<<NPUIQ as NeuronProcessingUnitIndexQuantization>::BurstIndexQuant>,
        _: &<Self as CorticalAreaDynamics<NPUIQ, BEIQ, NeuronLayoutVoxel<BEIQ>, CAMQ>>::CorticalDataProperties,
        _: &<Self as CorticalAreaDynamics<NPUIQ, BEIQ, NeuronLayoutVoxel<BEIQ>, CAMQ>>::CorticalDataInternal,
        _: &<Self as CorticalAreaDynamics<NPUIQ, BEIQ, NeuronLayoutVoxel<BEIQ>, CAMQ>>::CorticalDataShared,
        _: &mut <Self as CorticalAreaDynamics<NPUIQ, BEIQ, NeuronLayoutVoxel<BEIQ>, CAMQ>>::NeuronDataProperties,
        _: &mut <Self as CorticalAreaDynamics<NPUIQ, BEIQ, NeuronLayoutVoxel<BEIQ>, CAMQ>>::NeuronDataInternal,
        _: &CorticalNeuronLocalIndex<<BEIQ as BurstEngineIndexQuantization>::NeuronIndexQuant>,
        _: &NeuronLayoutVoxel<BEIQ>,
    ) -> NeuronDynamicsOutput<CAMQ> {
        NeuronDynamicsOutput::Firing(CorticalNeuronPotential::QUANT_ONE)
    }
}
