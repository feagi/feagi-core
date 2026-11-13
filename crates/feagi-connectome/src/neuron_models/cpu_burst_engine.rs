//! The neuron models as the CPU burst engine holds them: one store per neuron model and
//! quantization, listed flat so reaching a store is never more than a field access.
//!
//! Written by hand until the neuron model macros generate it. Adding a neuron model or a
//! quantization of one means adding its store as a field of [`CpuNeuronModelStoreList`], and to
//! `new`, `add_cortical_area` and `run_neuron_phase` of its `CpuNeuronModelStores` implementation

use crate::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedQuanted, NMFeagiAdvancedQuantStandard32Bit};
use crate::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleQuanted, NMFeagiSimpleQuantStandard32Bit};
use crate::neuron_models::models::exclusive_death::quantizations::{
    NMExclusiveDeathQuanted, NMExclusiveDeathQuantBFloat16, NMExclusiveDeathQuantFloat16, NMExclusiveDeathQuantFloat32,
    NMExclusiveDeathQuantFloat64,
};
use crate::neuron_models::models::exclusive_power::quantizations::{
    NMExclusivePowerQuanted, NMExclusivePowerQuantBFloat16, NMExclusivePowerQuantFloat16, NMExclusivePowerQuantFloat32,
    NMExclusivePowerQuantFloat64,
};
use crate::neuron_models::models::memory_fire_hashing::quantizations::{
    NMMemoryFireHashingQuanted, NMMemoryFireHashingQuantStandard32Bit,
};
use crate::neuron_models::models::{feagi_advanced, feagi_simple, exclusive_death, exclusive_power, memory_fire_hashing, NMModeled, NMModeledStorageOrGenerator};
use feagi_basis::BurstIndex;
use feagi_connectome_neural::burst_engine_cpu::{
    BurstEngineCPURayon, CpuAddCorticalAreaError, CpuAddedCorticalArea, CpuEngineNeuronData, CpuNeuronModelStore, CpuNeuronModelStores,
};
use feagi_connectome_neural::neural_basis::connectome_editing::ConnectomeVisitorDispatchWrapper;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

/// The CPU burst engine hosting every neuron model of this crate
pub type CpuBurstEngine<FIQ, VisTx> = BurstEngineCPURayon<FIQ, CpuNeuronModelStoreList<FIQ>, VisTx>;

/// The store of every neuron model at each of its quantizations, for the CPU burst engine
pub struct CpuNeuronModelStoreList<FIQ: FeagiIndexQuantization> {
    pub feagi_advanced_standard_32_bit:
        CpuNeuronModelStore<FIQ, NMFeagiAdvancedQuantStandard32Bit, feagi_advanced::BurstEngineDynamics<FIQ, NMFeagiAdvancedQuantStandard32Bit>>,

    pub feagi_simple_standard_32_bit:
        CpuNeuronModelStore<FIQ, NMFeagiSimpleQuantStandard32Bit, feagi_simple::BurstEngineDynamics<FIQ, NMFeagiSimpleQuantStandard32Bit>>,

    pub exclusive_death_float_16:
        CpuNeuronModelStore<FIQ, NMExclusiveDeathQuantFloat16, exclusive_death::BurstEngineDynamics<FIQ, NMExclusiveDeathQuantFloat16>>,
    pub exclusive_death_b_float_16:
        CpuNeuronModelStore<FIQ, NMExclusiveDeathQuantBFloat16, exclusive_death::BurstEngineDynamics<FIQ, NMExclusiveDeathQuantBFloat16>>,
    pub exclusive_death_float_32:
        CpuNeuronModelStore<FIQ, NMExclusiveDeathQuantFloat32, exclusive_death::BurstEngineDynamics<FIQ, NMExclusiveDeathQuantFloat32>>,
    pub exclusive_death_float_64:
        CpuNeuronModelStore<FIQ, NMExclusiveDeathQuantFloat64, exclusive_death::BurstEngineDynamics<FIQ, NMExclusiveDeathQuantFloat64>>,

    pub exclusive_power_float_16:
        CpuNeuronModelStore<FIQ, NMExclusivePowerQuantFloat16, exclusive_power::BurstEngineDynamics<FIQ, NMExclusivePowerQuantFloat16>>,
    pub exclusive_power_b_float_16:
        CpuNeuronModelStore<FIQ, NMExclusivePowerQuantBFloat16, exclusive_power::BurstEngineDynamics<FIQ, NMExclusivePowerQuantBFloat16>>,
    pub exclusive_power_float_32:
        CpuNeuronModelStore<FIQ, NMExclusivePowerQuantFloat32, exclusive_power::BurstEngineDynamics<FIQ, NMExclusivePowerQuantFloat32>>,
    pub exclusive_power_float_64:
        CpuNeuronModelStore<FIQ, NMExclusivePowerQuantFloat64, exclusive_power::BurstEngineDynamics<FIQ, NMExclusivePowerQuantFloat64>>,

    pub memory_fire_hashing_standard_32_bit: CpuNeuronModelStore<
        FIQ,
        NMMemoryFireHashingQuantStandard32Bit,
        memory_fire_hashing::BurstEngineDynamics<FIQ, NMMemoryFireHashingQuantStandard32Bit>,
    >,
}

impl<FIQ: FeagiIndexQuantization> CpuNeuronModelStores<FIQ> for CpuNeuronModelStoreList<FIQ> {
    type CorticalAreaNeurogenesisDispatch = NMModeledStorageOrGenerator<FIQ>;

    fn new() -> Self {
        Self {
            feagi_advanced_standard_32_bit: CpuNeuronModelStore::new(),
            feagi_simple_standard_32_bit: CpuNeuronModelStore::new(),
            exclusive_death_float_16: CpuNeuronModelStore::new(),
            exclusive_death_b_float_16: CpuNeuronModelStore::new(),
            exclusive_death_float_32: CpuNeuronModelStore::new(),
            exclusive_death_float_64: CpuNeuronModelStore::new(),
            exclusive_power_float_16: CpuNeuronModelStore::new(),
            exclusive_power_b_float_16: CpuNeuronModelStore::new(),
            exclusive_power_float_32: CpuNeuronModelStore::new(),
            exclusive_power_float_64: CpuNeuronModelStore::new(),
            memory_fire_hashing_standard_32_bit: CpuNeuronModelStore::new(),
        }
    }

    fn add_cortical_area(
        &mut self,
        neurogenesis: Self::CorticalAreaNeurogenesisDispatch,
        engine: &mut CpuEngineNeuronData<FIQ>,
    ) -> Result<CpuAddedCorticalArea<FIQ>, CpuAddCorticalAreaError> {
        use NMFeagiAdvancedQuanted as FeagiAdvanced;
        use NMFeagiSimpleQuanted as FeagiSimple;
        use NMExclusiveDeathQuanted as ExclusiveDeath;
        use NMExclusivePowerQuanted as ExclusivePower;
        use NMMemoryFireHashingQuanted as MemoryFireHashing;
        use NMModeled as Model;

        match neurogenesis {
            Model::FeagiAdvanced(FeagiAdvanced::Standard32Bit(area)) => self.feagi_advanced_standard_32_bit.add_cortical_area(area, engine),

            Model::FeagiSimple(FeagiSimple::Standard32Bit(area)) => self.feagi_simple_standard_32_bit.add_cortical_area(area, engine),

            Model::ExclusiveDeath(ExclusiveDeath::Float16(area)) => self.exclusive_death_float_16.add_cortical_area(area, engine),
            Model::ExclusiveDeath(ExclusiveDeath::BFloat16(area)) => self.exclusive_death_b_float_16.add_cortical_area(area, engine),
            Model::ExclusiveDeath(ExclusiveDeath::Float32(area)) => self.exclusive_death_float_32.add_cortical_area(area, engine),
            Model::ExclusiveDeath(ExclusiveDeath::Float64(area)) => self.exclusive_death_float_64.add_cortical_area(area, engine),

            Model::ExclusivePower(ExclusivePower::Float16(area)) => self.exclusive_power_float_16.add_cortical_area(area, engine),
            Model::ExclusivePower(ExclusivePower::BFloat16(area)) => self.exclusive_power_b_float_16.add_cortical_area(area, engine),
            Model::ExclusivePower(ExclusivePower::Float32(area)) => self.exclusive_power_float_32.add_cortical_area(area, engine),
            Model::ExclusivePower(ExclusivePower::Float64(area)) => self.exclusive_power_float_64.add_cortical_area(area, engine),

            Model::MemoryFireHashing(MemoryFireHashing::Standard32Bit(area)) => self.memory_fire_hashing_standard_32_bit.add_cortical_area(area, engine),
        }
    }

    fn run_neuron_phase(&mut self, burst_index: &BurstIndex<FIQ::BurstIndexQuant>, engine: &mut CpuEngineNeuronData<FIQ>) {
        self.feagi_advanced_standard_32_bit.run_neuron_phase(burst_index, engine);
        self.feagi_simple_standard_32_bit.run_neuron_phase(burst_index, engine);
        self.exclusive_death_float_16.run_neuron_phase(burst_index, engine);
        self.exclusive_death_b_float_16.run_neuron_phase(burst_index, engine);
        self.exclusive_death_float_32.run_neuron_phase(burst_index, engine);
        self.exclusive_death_float_64.run_neuron_phase(burst_index, engine);
        self.exclusive_power_float_16.run_neuron_phase(burst_index, engine);
        self.exclusive_power_b_float_16.run_neuron_phase(burst_index, engine);
        self.exclusive_power_float_32.run_neuron_phase(burst_index, engine);
        self.exclusive_power_float_64.run_neuron_phase(burst_index, engine);
        self.memory_fire_hashing_standard_32_bit.run_neuron_phase(burst_index, engine);
    }
}

/// The dispatch types of the connectome edit steps the CPU burst engine takes
pub struct CpuConnectomeDispatch;

impl<FIQ: FeagiIndexQuantization> ConnectomeVisitorDispatchWrapper<FIQ> for CpuConnectomeDispatch {
    type CorticalAreaNeurogenesisDispatch = NMModeledStorageOrGenerator<FIQ>;
}
