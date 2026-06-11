use core::marker::PhantomData;
use feagi_structures::feagi_data::feagi_pdi::PDICollection;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};
use feagi_structures::feagi_data::SupportsUintOps;
use crate::neural_processing_unit_data_structures::neuron_models::base_traits_all_devices::{NeuronModelCorticalData};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_types::cortical_area_layout::CorticalConfigurationBase;
use crate::neural_processing_unit_data_structures::cpu_wrappers::indexes_burst_engine::NPUCorticalAreaModelQuantizationIndex;

/// Holds all cortical level data
pub trait TypedNeuronModelCorticalDataCollection<FGQ, NMQ, CCC, CMD>:
PDICollection
+ PDITagGenericDevice
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCC: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
    CMD: NeuronModelCorticalData<FGQ, NMQ, CCC>,
{
    fn get_number_of_cortical_areas_of_this_type(&self) -> NPUCorticalAreaModelQuantizationIndex<FGQ::CorticalAreaIndexCountQuant>;

    // NOTE: We assume NeuronModelNeuronData as an implementation doesnt need padding!
}

//region CPU Implementations

pub struct TypedNeuronModelCorticalDataCPUCollection<FGQ, NMQ, CCC, CMD>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCC: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
    CMD: NeuronModelCorticalData<FGQ, NMQ, CCC>,
{
    pub typed_cortical_data: Vec<CMD>,
    _p: PhantomData<(FGQ, NMQ, CCC, CMD)>,
}

impl<FGQ, NMQ, CCC, CMD> PDICollection for TypedNeuronModelCorticalDataCPUCollection<FGQ, NMQ, CCC, CMD>
where
    CCC: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
    CMD: NeuronModelCorticalData<FGQ, NMQ, CCC>,
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
{}

impl<FGQ, NMQ, CCC, CMD> PDITagGenericDevice for TypedNeuronModelCorticalDataCPUCollection<FGQ, NMQ, CCC, CMD>
where
    CCC: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
    CMD: NeuronModelCorticalData<FGQ, NMQ, CCC>,
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
{}

impl<FGQ, NMQ, CCC, CMD> PDITagCPU for TypedNeuronModelCorticalDataCPUCollection<FGQ, NMQ, CCC, CMD> where
    CCC: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
    CMD: NeuronModelCorticalData<FGQ, NMQ, CCC>,
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
{}

impl<FGQ, NMQ, CCC, CMD> TypedNeuronModelCorticalDataCollection<FGQ, NMQ, CCC, CMD> for TypedNeuronModelCorticalDataCPUCollection<FGQ, NMQ, CCC, CMD>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCC: CorticalConfigurationBase<FGQ, NMQ::CorticalPotentialQuant>,
    CMD: NeuronModelCorticalData<FGQ, NMQ, CCC>,
{
    fn get_number_of_cortical_areas_of_this_type(&self) -> NPUCorticalAreaModelQuantizationIndex<FGQ::CorticalAreaIndexCountQuant> {
        NPUCorticalAreaModelQuantizationIndex::wrap(FGQ::CorticalAreaIndexCountQuant::from_usize_unchecked(self.typed_cortical_data.len()))
    }
}




//endregion