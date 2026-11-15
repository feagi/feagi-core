//! Encoder for the MIDI Input unit (Notes area and Expression area).

use crate::configuration::jsonable::JSONEncoderProperties;
use crate::data_pipeline::per_channel_stream_caches::{
    PipelineStageRunner, SensoryPipelineStageRunner,
};
use crate::data_types::{
    MidiControl, MidiFrame, MidiProperties, Percentage, MIDI_NOTE_FULL_SCALE_POTENTIAL,
};
use crate::neuron_voxel_coding::xyzp::percentage_variant::PercentageCodingVariant;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPEncoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalChannelIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};
use feagi_structures::FeagiDataError;
use std::time::Instant;

/// Writes one [`MidiFrame`] per channel into the two MIDI Input areas, the exact
/// inverse of the MIDI Output decoder.
///
/// Notes: a note with strength above 0 writes one voxel at x = channel *
/// note_columns + note column, y = 0, z = 0, with potential = strength * 255
/// (FEAGI's `0..=255` level scale; strength 0 writes nothing). Expression: a
/// control that is driven (`Some(level)`) writes the z indexes of its level,
/// coded with the unsigned percentage rule the percentage encoder uses, at x =
/// channel * 5 + control column, y = 0, potential 1.0; a control that is not
/// driven writes nothing.
///
/// Stateless: every call writes exactly the frames that were updated since the
/// previous burst, and nothing from earlier bursts. Holding a note across
/// bursts, releasing it, and timing live in the application, not here.
#[derive(Debug)]
pub struct MidiInputNeuronVoxelXYZPEncoder {
    properties: MidiProperties,
    notes_cortical_id: CorticalID,
    expression_cortical_id: CorticalID,
    interpolation: PercentageNeuronPositioning,
    notes_scratch: Vec<NeuronVoxelXYZPArrays>,
    expression_scratch: Vec<NeuronVoxelXYZPArrays>,
    /// Active z indexes of one control, reused between writes.
    z_scratch: Vec<u32>,
}

impl NeuronVoxelXYZPEncoder for MidiInputNeuronVoxelXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::MidiFrame(Some(self.properties))
    }

    fn get_as_properties(&self) -> JSONEncoderProperties {
        JSONEncoderProperties::MidiInput(self.properties, self.interpolation)
    }

    fn write_neuron_data_multi_channel_from_processed_cache(
        &mut self,
        pipelines: &[SensoryPipelineStageRunner],
        time_of_previous_burst: Instant,
        write_target: &mut CorticalMappedXYZPNeuronVoxels,
    ) -> Result<(), FeagiDataError> {
        for (channel_index, pipeline) in pipelines.iter().enumerate() {
            self.notes_scratch[channel_index].clear();
            self.expression_scratch[channel_index].clear();
            if pipeline.get_last_processed_instant() < time_of_previous_burst {
                continue;
            }
            let channel = *pipeline
                .get_channel_index_override()
                .unwrap_or_else(|| CorticalChannelIndex::from(channel_index as u32));
            let data = pipeline.get_postprocessed_sensor_value();
            let frame: &MidiFrame = data.try_into()?;
            self.write_notes(frame, channel, channel_index);
            self.write_expression(frame, channel, channel_index);
        }

        let notes_target = write_target.ensure_clear_and_borrow_mut(&self.notes_cortical_id);
        copy_scratch(notes_target, &self.notes_scratch)?;
        let expression_target =
            write_target.ensure_clear_and_borrow_mut(&self.expression_cortical_id);
        copy_scratch(expression_target, &self.expression_scratch)?;
        Ok(())
    }
}

fn copy_scratch(
    target: &mut NeuronVoxelXYZPArrays,
    scratch: &[NeuronVoxelXYZPArrays],
) -> Result<(), FeagiDataError> {
    let total: usize = scratch.iter().map(|s| s.len()).sum();
    target.ensure_capacity(total);
    target.update_vectors_from_external(|target_x, target_y, target_z, target_p| {
        for s in scratch {
            let (x, y, z, p) = s.borrow_xyzp_vectors();
            target_x.extend_from_slice(x);
            target_y.extend_from_slice(y);
            target_z.extend_from_slice(z);
            target_p.extend_from_slice(p);
        }
        Ok(())
    })
}

impl MidiInputNeuronVoxelXYZPEncoder {
    pub fn new_box(
        notes_cortical_id: CorticalID,
        expression_cortical_id: CorticalID,
        properties: MidiProperties,
        interpolation: PercentageNeuronPositioning,
        number_of_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        properties.validate()?;
        let channels = *number_of_channels as usize;
        Ok(Box::new(MidiInputNeuronVoxelXYZPEncoder {
            properties,
            notes_cortical_id,
            expression_cortical_id,
            interpolation,
            notes_scratch: vec![NeuronVoxelXYZPArrays::new(); channels],
            expression_scratch: vec![NeuronVoxelXYZPArrays::new(); channels],
            z_scratch: Vec::new(),
        }))
    }

    fn write_notes(&mut self, frame: &MidiFrame, channel: u32, slot: usize) {
        let x_offset = channel * self.properties.note_columns;
        let scratch = &mut self.notes_scratch[slot];
        for (column, strength) in frame.notes.strengths().iter().enumerate() {
            if *strength > 0.0 {
                scratch.push_raw(
                    x_offset + column as u32,
                    0,
                    0,
                    strength * MIDI_NOTE_FULL_SCALE_POTENTIAL,
                );
            }
        }
    }

    fn write_expression(&mut self, frame: &MidiFrame, channel: u32, slot: usize) {
        let controls = MidiControl::COUNT as u32;
        let depth = self.properties.expression_depth;
        let variant = PercentageCodingVariant::from_parts(false, true, self.interpolation);
        for control in MidiControl::ALL {
            let Some(level) = frame.expression.level(control) else {
                continue;
            };
            let level = Percentage::new_from_0_1_unchecked(level.clamp(0.0, 1.0));
            variant.encode_unsigned_into(&level, depth, &mut self.z_scratch);
            let x = channel * controls + control.column() as u32;
            for z in &self.z_scratch {
                self.expression_scratch[slot].push_raw(x, 0, *z, 1.0);
            }
        }
    }
}
