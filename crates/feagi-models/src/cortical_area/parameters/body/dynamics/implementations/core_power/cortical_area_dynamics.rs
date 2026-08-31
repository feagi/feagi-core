use crate::cortical_area::components::neuron_layout::implementations::voxel::NeuronLayoutVoxel;
use crate::cortical_area::components::neuron_layout::neuron_layout_model::NeuronLayout;
use crate::cortical_area::parameters::body::dynamics::components::data::NullData;
use crate::cortical_area::parameters::body::dynamics::components::mp_driven_psp_configurability::MPDrivenPSPForcedOff;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::cortical_area::parameters::body::dynamics::cortical_area_dynamics::{CorticalAreaDynamics, NeuronDynamicsOutput};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::wrapped_indexes::BurstIndex;
use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, CorticalNeuronPotential};
use std::marker::PhantomData;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

/// Simply fires every burst lol
pub struct CorePowerCorticalAreaDynamics<FIQ, NL, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CAMQ: CorticalAreaQuantization,
{
    _p: PhantomData<(FIQ, NL, CAMQ)>,
}

impl<FIQ, CAMQ> CorticalAreaDynamics<FIQ, NeuronLayoutVoxel<FIQ>, CAMQ>
    for CorePowerCorticalAreaDynamics<FIQ, NeuronLayoutVoxel<FIQ>, CAMQ>
where
    FIQ: FeagiIndexQuantization,
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
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        cortical_properties: &mut Self::CorticalDataProperties,
        cortical_internal: &mut Self::CorticalDataInternal,
        cortical_shared: &mut Self::CorticalDataShared,
        layout_context: &NeuronLayoutVoxel<FIQ>,
    ) -> () {
        
    }

    fn process_neuron_dynamics(
        _: &CorticalNeuronPotential<<CAMQ as MembranePotentialQuantization>::MembranePotentialQuant>,
        _: &BurstIndex<<FIQ as FeagiIndexQuantization>::BurstIndexQuant>,
        _: &<Self as CorticalAreaDynamics<FIQ, NeuronLayoutVoxel<FIQ>, CAMQ>>::CorticalDataProperties,
        _: &<Self as CorticalAreaDynamics<FIQ, NeuronLayoutVoxel<FIQ>, CAMQ>>::CorticalDataInternal,
        _: &<Self as CorticalAreaDynamics<FIQ, NeuronLayoutVoxel<FIQ>, CAMQ>>::CorticalDataShared,
        _: &mut <Self as CorticalAreaDynamics<FIQ, NeuronLayoutVoxel<FIQ>, CAMQ>>::NeuronDataProperties,
        _: &mut <Self as CorticalAreaDynamics<FIQ, NeuronLayoutVoxel<FIQ>, CAMQ>>::NeuronDataInternal,
        _: &CorticalNeuronLocalIndex<<FIQ as FeagiIndexQuantization>::NeuronIndexQuant>,
        _: &NeuronLayoutVoxel<FIQ>,
    ) -> NeuronDynamicsOutput<CAMQ> {
        NeuronDynamicsOutput::Firing(CorticalNeuronPotential::QUANT_ONE)
    }
}
