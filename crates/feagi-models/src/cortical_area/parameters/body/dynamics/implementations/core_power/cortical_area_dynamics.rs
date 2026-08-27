use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayout};
use crate::cortical_area::parameters::body::dynamics::components::data::NullData;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::cortical_area::parameters::body::dynamics::cortical_area_dynamics::CorticalAreaDynamics;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use crate::wrapped_indexes::BurstIndex;
use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, CorticalNeuronPotential};
use std::marker::PhantomData;
use crate::cortical_area::components::neuron_layout::implementations::voxel::NeuronLayoutVoxel;

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

impl<NPUIQ, BEIQ, CAMQ> CorticalAreaDynamics<NPUIQ, BEIQ, NeuronLayoutVoxel<BEIQ>, CAMQ> for CorePowerCorticalAreaDynamics<NPUIQ, BEIQ, NeuronLayoutVoxel<BEIQ>, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    type CorticalDataProperties = NullData<CAMQ>;
    type CorticalDataInternal = NullData<CAMQ>;
    type CorticalDataShared = NullData<CAMQ>;
    type NeuronDataProperties = NullData<CAMQ>;
    type NeuronDataInternal = NullData<CAMQ>;

    fn process_neuron_dynamics(
        _incoming_potential: &CorticalNeuronPotential<CAMQ::MembranePotentialQuant>,
        _burst_index: &BurstIndex<NPUIQ::BurstIndexQuant>,
        _cortical_properties: &Self::CorticalDataProperties,
        _cortical_internal: &Self::CorticalDataInternal,
        _cortical_shared: &Self::CorticalDataShared,
        _neuron_properties: &mut Self::NeuronDataProperties,
        _neuron_internal: &mut Self::NeuronDataInternal,
        _neuron_linear_index: &CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>,
        _layout_context: &NeuronLayoutVoxel<BEIQ>,
    ) -> bool {
        return true;
    }
}
