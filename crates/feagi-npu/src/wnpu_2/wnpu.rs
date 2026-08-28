//! A temporary wrapper

use feagi_data::bidirectional_channel_queue::{MpscBiDirectionalChannelQueue, MpscBidirectionalChannelSide};
use crate::npu::neuron_processor_unit_composable::NeuronProcessingUnitComposable;
use crate::standard::npu::npu_target_frequency::NPUTargetFrequency;
use crate::wnpu::agents::agent_registration_response::AgentRegistrationResponse;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationGenomic;
use feagi_genomic_context::cortical_area::CorticalID;
use crate::wnpu::agents::data_exchange::force_fire::VoxelForceFire;
use crate::wnpu::agents::data_exchange::visualization::VoxelVisualization;
use crate::wnpu::agents::data_exchange::voxel_potentials::CorticalAreaVoxelPotentials;

/// Compatibility wrapper for the new NPU as it is developed and as we use the old FEAGI architecture.
/// Can be owned directly, handles the threading and quantization shenanigans internally
pub struct WrappedNeuronProcessingUnit {
    npu: NeuronProcessingUnitComposable<FeagiIndexQuantizationGenomic>,
    inputs_sensors: Vec<MpscBidirectionalChannelSide<CorticalAreaVoxelPotentials<u32, f32>>>,
    inputs_force_fire: Vec<MpscBidirectionalChannelSide<VoxelForceFire<u32>>>,
    outputs_motors: Vec<MpscBidirectionalChannelSide<CorticalAreaVoxelPotentials<u32, f32>>>,
    outputs_visualization: Vec<MpscBidirectionalChannelSide<VoxelVisualization<u32>>>,
}

impl WrappedNeuronProcessingUnit {
    pub fn new() -> WrappedNeuronProcessingUnit {
        Self {
            npu: NeuronProcessingUnitComposable::new(),
            inputs_sensors: vec![],
            inputs_force_fire: vec![],
            outputs_motors: vec![],
            outputs_visualization: vec![],
        }
    }

    /// Start engines at a frequency or update the frequency if its already running
    pub fn run_at(&mut self, burst_frequency: NPUTargetFrequency) {
        self.npu.start_engines(
            burst_frequency,
            &mut self.inputs_sensors,
            &mut self.inputs_force_fire,
            &mut self.outputs_motors,
            &mut self.outputs_visualization,
        )
    }

    /// Stop the NPU from running. Best called during shutdown for now
    pub fn stop_npu(&mut self) {
        self.npu.stop_engines()
    }

    /// Submit  change request (adding cortical area, mapping, etc) to the NPU.Change will be implemented next best oppertunity
    pub fn request_change(&mut self) {
        todo!()
    }

    // TODO rename from register (npu subscribe)

    pub fn register_agent(
        &mut self,
        uses_visualization: bool,
        uses_force_fire: bool,
        sensor_ids: Vec<CorticalID>,
        motor_ids: Vec<CorticalID>,
    ) -> AgentRegistrationResponse {
        // TODO wire these ids into routing and filtering.
        let _ = (sensor_ids, motor_ids);

        let queue = MpscBiDirectionalChannelQueue::new(2, 2);
        let (outside_side, npu_side) = queue.split();
        let input_sensor: Option<MpscBidirectionalChannelSide<CorticalAreaVoxelPotentials<u32, f32>>> = Some(outside_side);
        self.inputs_sensors.push(npu_side);

        let mut input_force_fire: Option<MpscBidirectionalChannelSide<VoxelForceFire<u32>>> = None;
        let mut output_visualization: Option<MpscBidirectionalChannelSide<VoxelVisualization<u32>>> = None;

        let queue = MpscBiDirectionalChannelQueue::new(2, 2);
        let (outside_side, npu_side) = queue.split();
        let output_motor: Option<MpscBidirectionalChannelSide<CorticalAreaVoxelPotentials<u32, f32>>> = Some(outside_side);
        self.outputs_motors.push(npu_side);

        if uses_force_fire {
            let queue = MpscBiDirectionalChannelQueue::new(2, 2);
            let (outside_side, npu_side) = queue.split();
            input_force_fire = Some(outside_side);
            self.inputs_force_fire.push(npu_side);
        }

        if uses_visualization {
            let queue = MpscBiDirectionalChannelQueue::new(2, 2);
            let (outside_side, npu_side) = queue.split();
            output_visualization = Some(outside_side);
            self.outputs_visualization.push(npu_side);
        }

        AgentRegistrationResponse {
            input_sensor,
            input_force_fire,
            output_visualization,
            output_motor,
        }
    }
}
