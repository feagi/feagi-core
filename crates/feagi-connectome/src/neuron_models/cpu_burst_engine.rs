//! The neuron models as the CPU burst engine holds them: one store per neuron model and
//! quantization, listed flat so reaching a store is never more than a field access.
//!
//! Written by hand until the neuron model macros generate it. Adding a neuron model or a
//! quantization of one means adding its store as a field of [`CpuNeuronModelStoreList`], and to
//! `new`, `add_cortical_area` and `run_neuron_phase` of its `CpuNeuronModelStores` implementation

use crate::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted, NMFeagiAdvancedQuantStandard32Bit};
use crate::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted, NMFeagiSimpleQuantStandard32Bit};
use crate::neuron_models::models::foundation_death::quantizations::{
    NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted, NMFoundationDeathQuantBFloat16, NMFoundationDeathQuantFloat16, NMFoundationDeathQuantFloat32,
    NMFoundationDeathQuantFloat64,
};
use crate::neuron_models::models::foundation_power::quantizations::{
    NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted, NMFoundationPowerQuantBFloat16, NMFoundationPowerQuantFloat16, NMFoundationPowerQuantFloat32,
    NMFoundationPowerQuantFloat64,
};
use crate::neuron_models::models::memory_fire_hashing::quantizations::{
    NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted, NMMemoryFireHashingQuantStandard32Bit,
};
use crate::neuron_models::models::{feagi_advanced, feagi_simple, foundation_death, foundation_power, memory_fire_hashing, NMNeurogenesisStorageOrGeneratorModeled};
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

    pub foundation_death_float_16:
        CpuNeuronModelStore<FIQ, NMFoundationDeathQuantFloat16, foundation_death::BurstEngineDynamics<FIQ, NMFoundationDeathQuantFloat16>>,
    pub foundation_death_b_float_16:
        CpuNeuronModelStore<FIQ, NMFoundationDeathQuantBFloat16, foundation_death::BurstEngineDynamics<FIQ, NMFoundationDeathQuantBFloat16>>,
    pub foundation_death_float_32:
        CpuNeuronModelStore<FIQ, NMFoundationDeathQuantFloat32, foundation_death::BurstEngineDynamics<FIQ, NMFoundationDeathQuantFloat32>>,
    pub foundation_death_float_64:
        CpuNeuronModelStore<FIQ, NMFoundationDeathQuantFloat64, foundation_death::BurstEngineDynamics<FIQ, NMFoundationDeathQuantFloat64>>,

    pub foundation_power_float_16:
        CpuNeuronModelStore<FIQ, NMFoundationPowerQuantFloat16, foundation_power::BurstEngineDynamics<FIQ, NMFoundationPowerQuantFloat16>>,
    pub foundation_power_b_float_16:
        CpuNeuronModelStore<FIQ, NMFoundationPowerQuantBFloat16, foundation_power::BurstEngineDynamics<FIQ, NMFoundationPowerQuantBFloat16>>,
    pub foundation_power_float_32:
        CpuNeuronModelStore<FIQ, NMFoundationPowerQuantFloat32, foundation_power::BurstEngineDynamics<FIQ, NMFoundationPowerQuantFloat32>>,
    pub foundation_power_float_64:
        CpuNeuronModelStore<FIQ, NMFoundationPowerQuantFloat64, foundation_power::BurstEngineDynamics<FIQ, NMFoundationPowerQuantFloat64>>,

    pub memory_fire_hashing_standard_32_bit: CpuNeuronModelStore<
        FIQ,
        NMMemoryFireHashingQuantStandard32Bit,
        memory_fire_hashing::BurstEngineDynamics<FIQ, NMMemoryFireHashingQuantStandard32Bit>,
    >,
}

impl<FIQ: FeagiIndexQuantization> CpuNeuronModelStores<FIQ> for CpuNeuronModelStoreList<FIQ> {
    type CorticalAreaNeurogenesisDispatch = NMNeurogenesisStorageOrGeneratorModeled<FIQ>;

    fn new() -> Self {
        Self {
            feagi_advanced_standard_32_bit: CpuNeuronModelStore::new(),
            feagi_simple_standard_32_bit: CpuNeuronModelStore::new(),
            foundation_death_float_16: CpuNeuronModelStore::new(),
            foundation_death_b_float_16: CpuNeuronModelStore::new(),
            foundation_death_float_32: CpuNeuronModelStore::new(),
            foundation_death_float_64: CpuNeuronModelStore::new(),
            foundation_power_float_16: CpuNeuronModelStore::new(),
            foundation_power_b_float_16: CpuNeuronModelStore::new(),
            foundation_power_float_32: CpuNeuronModelStore::new(),
            foundation_power_float_64: CpuNeuronModelStore::new(),
            memory_fire_hashing_standard_32_bit: CpuNeuronModelStore::new(),
        }
    }

    fn add_cortical_area(
        &mut self,
        neurogenesis: Self::CorticalAreaNeurogenesisDispatch,
        engine: &mut CpuEngineNeuronData<FIQ>,
    ) -> Result<CpuAddedCorticalArea<FIQ>, CpuAddCorticalAreaError> {
        use NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted as FeagiAdvanced;
        use NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted as FeagiSimple;
        use NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted as FoundationDeath;
        use NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted as FoundationPower;
        use NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted as MemoryFireHashing;
        use NMNeurogenesisStorageOrGeneratorModeled as Model;

        match neurogenesis {
            Model::FeagiAdvanced(FeagiAdvanced::Standard32Bit(area)) => self.feagi_advanced_standard_32_bit.add_cortical_area(area, engine),

            Model::FeagiSimple(FeagiSimple::Standard32Bit(area)) => self.feagi_simple_standard_32_bit.add_cortical_area(area, engine),

            Model::FoundationDeath(FoundationDeath::Float16(area)) => self.foundation_death_float_16.add_cortical_area(area, engine),
            Model::FoundationDeath(FoundationDeath::BFloat16(area)) => self.foundation_death_b_float_16.add_cortical_area(area, engine),
            Model::FoundationDeath(FoundationDeath::Float32(area)) => self.foundation_death_float_32.add_cortical_area(area, engine),
            Model::FoundationDeath(FoundationDeath::Float64(area)) => self.foundation_death_float_64.add_cortical_area(area, engine),

            Model::FoundationPower(FoundationPower::Float16(area)) => self.foundation_power_float_16.add_cortical_area(area, engine),
            Model::FoundationPower(FoundationPower::BFloat16(area)) => self.foundation_power_b_float_16.add_cortical_area(area, engine),
            Model::FoundationPower(FoundationPower::Float32(area)) => self.foundation_power_float_32.add_cortical_area(area, engine),
            Model::FoundationPower(FoundationPower::Float64(area)) => self.foundation_power_float_64.add_cortical_area(area, engine),

            Model::MemoryFireHashing(MemoryFireHashing::Standard32Bit(area)) => self.memory_fire_hashing_standard_32_bit.add_cortical_area(area, engine),
        }
    }

    fn run_neuron_phase(&mut self, burst_index: &BurstIndex<FIQ::BurstIndexQuant>, engine: &mut CpuEngineNeuronData<FIQ>) {
        self.feagi_advanced_standard_32_bit.run_neuron_phase(burst_index, engine);
        self.feagi_simple_standard_32_bit.run_neuron_phase(burst_index, engine);
        self.foundation_death_float_16.run_neuron_phase(burst_index, engine);
        self.foundation_death_b_float_16.run_neuron_phase(burst_index, engine);
        self.foundation_death_float_32.run_neuron_phase(burst_index, engine);
        self.foundation_death_float_64.run_neuron_phase(burst_index, engine);
        self.foundation_power_float_16.run_neuron_phase(burst_index, engine);
        self.foundation_power_b_float_16.run_neuron_phase(burst_index, engine);
        self.foundation_power_float_32.run_neuron_phase(burst_index, engine);
        self.foundation_power_float_64.run_neuron_phase(burst_index, engine);
        self.memory_fire_hashing_standard_32_bit.run_neuron_phase(burst_index, engine);
    }
}

/// The dispatch types of the connectome edit steps the CPU burst engine takes
pub struct CpuConnectomeDispatch;

impl<FIQ: FeagiIndexQuantization> ConnectomeVisitorDispatchWrapper<FIQ> for CpuConnectomeDispatch {
    type CorticalAreaNeurogenesisDispatch = NMNeurogenesisStorageOrGeneratorModeled<FIQ>;
}
