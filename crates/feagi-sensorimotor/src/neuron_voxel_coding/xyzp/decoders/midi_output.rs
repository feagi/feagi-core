//! Decoder for the MIDI Output unit (Notes area and Expression area).

use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::{MidiControl, MidiOutputFrame, MidiOutputProperties, Percentage};
use crate::neuron_voxel_coding::xyzp::percentage_variant::PercentageCodingVariant;
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalChannelCount;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;

/// Reads both MIDI Output areas into one [`MidiOutputFrame`] per channel.
///
/// Notes: x = channel * note_columns + note column, y = 0, z = 0;
/// a firing voxel is a sounding note and its potential (0..=255 scale, divided by 255 and
/// clamped to 0..1) is its strength. Expression: x = channel * 5 + control column, y = 0,
/// z = level, decoded with the same unsigned percentage rule the percentage
/// decoder uses.
///
/// Like the audio spectrum decoder, every decode replaces the whole state and
/// marks every channel changed, including on a silent frame, so no note or
/// control value is left stale.
#[derive(Debug)]
pub struct MidiOutputNeuronVoxelXYZPDecoder {
    notes_cortical_id: CorticalID,
    expression_cortical_id: CorticalID,
    properties: MidiOutputProperties,
    interpolation: PercentageNeuronPositioning,
    /// Active z indexes per (channel, control), reused between decodes.
    z_scratch: Vec<Vec<u32>>,
}

impl NeuronVoxelXYZPDecoder for MidiOutputNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::MidiOutputFrame(Some(self.properties))
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        JSONDecoderProperties::MidiOutput(self.properties, self.interpolation)
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        _time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError> {
        let mut frames: Vec<&mut MidiOutputFrame> =
            Vec::with_capacity(pipelines_with_data_to_update.len());
        for pipeline in pipelines_with_data_to_update.iter_mut() {
            frames.push(pipeline.get_preprocessed_cached_value_mut().try_into()?);
        }
        self.decode_into_frames(neurons_to_read, &mut frames);
        for changed in channel_changed.iter_mut() {
            *changed = true;
        }
        Ok(())
    }
}

impl MidiOutputNeuronVoxelXYZPDecoder {
    pub fn new_box(
        notes_cortical_id: CorticalID,
        expression_cortical_id: CorticalID,
        properties: MidiOutputProperties,
        interpolation: PercentageNeuronPositioning,
        number_of_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        properties.validate()?;
        let scratch_size = *number_of_channels as usize * MidiControl::COUNT;
        Ok(Box::new(MidiOutputNeuronVoxelXYZPDecoder {
            notes_cortical_id,
            expression_cortical_id,
            properties,
            interpolation,
            z_scratch: vec![Vec::new(); scratch_size],
        }))
    }

    /// Replace the whole state of every frame from this frame's firing voxels.
    /// Voxels outside the area's dims are ignored.
    fn decode_into_frames(
        &mut self,
        neurons: &CorticalMappedXYZPNeuronVoxels,
        frames: &mut [&mut MidiOutputFrame],
    ) {
        for frame in frames.iter_mut() {
            frame.silence();
        }
        let channel_count = frames.len() as u32;

        // Notes: strength is the potential / 255, clamped to 0..1; the largest one wins per cell.
        if let Some(notes) = neurons.get_neurons_of(&self.notes_cortical_id) {
            let columns = self.properties.note_columns;
            for neuron in notes.iter() {
                let c = neuron.neuron_voxel_coordinate;
                if c.x >= columns * channel_count || c.y != 0 || c.z != 0 {
                    continue;
                }
                if neuron.potential <= 0.0 {
                    continue;
                }
                let channel = (c.x / columns) as usize;
                let column = (c.x % columns) as usize;
                frames[channel]
                    .notes
                    .raise_strength(column, neuron.potential);
            }
        }

        // Expression: collect active z per (channel, control), then decode each
        // with the unsigned percentage rule shared with the percentage decoder.
        for scratch in self.z_scratch.iter_mut() {
            scratch.clear();
        }
        let controls = MidiControl::COUNT as u32;
        let depth = self.properties.expression_depth;
        let mut any_expression = false;
        if let Some(expression) = neurons.get_neurons_of(&self.expression_cortical_id) {
            for neuron in expression.iter() {
                let c = neuron.neuron_voxel_coordinate;
                if c.x >= controls * channel_count || c.y != 0 || c.z >= depth {
                    continue;
                }
                if neuron.potential == 0.0 {
                    continue;
                }
                let channel = c.x / controls;
                let control = c.x % controls;
                let index = (channel * controls + control) as usize;
                if let Some(z_indexes) = self.z_scratch.get_mut(index) {
                    z_indexes.push(c.z);
                    any_expression = true;
                }
            }
        }
        if !any_expression {
            return;
        }
        let variant = PercentageCodingVariant::from_parts(false, true, self.interpolation);
        for (channel, frame) in frames.iter_mut().enumerate() {
            for control in MidiControl::ALL {
                let index = channel * MidiControl::COUNT + control.column();
                let Some(z_indexes) = self.z_scratch.get(index) else {
                    continue;
                };
                if z_indexes.is_empty() {
                    continue;
                }
                let mut level = Percentage::new_zero();
                variant.decode_unsigned_into(z_indexes, depth, &mut level);
                frame.expression.set_level(control, level.get_as_0_1());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wrapped_io_data::WrappedIOData;
    use feagi_structures::genomic::cortical_area::descriptors::{
        CorticalSubUnitIndex, CorticalUnitIndex,
    };
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
        FrameChangeHandling, IOCorticalAreaConfigurationFlag,
    };
    use feagi_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;

    const DEPTH: u32 = 10;
    const COLUMNS: u32 = 128;

    fn notes_id() -> CorticalID {
        IOCorticalAreaConfigurationFlag::Misc(FrameChangeHandling::Absolute).as_io_cortical_id(
            false,
            *b"mid",
            CorticalUnitIndex::from(0u16),
            CorticalSubUnitIndex::from(0u8),
        )
    }

    fn expression_id() -> CorticalID {
        IOCorticalAreaConfigurationFlag::Percentage(
            FrameChangeHandling::Absolute,
            PercentageNeuronPositioning::Linear,
        )
        .as_io_cortical_id(
            false,
            *b"mid",
            CorticalUnitIndex::from(0u16),
            CorticalSubUnitIndex::from(1u8),
        )
    }

    fn properties() -> MidiOutputProperties {
        MidiOutputProperties::new(COLUMNS, DEPTH).unwrap()
    }

    fn decoder() -> Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> {
        MidiOutputNeuronVoxelXYZPDecoder::new_box(
            notes_id(),
            expression_id(),
            properties(),
            PercentageNeuronPositioning::Linear,
            CorticalChannelCount::new(1).unwrap(),
        )
        .unwrap()
    }

    fn pipelines() -> Vec<MotorPipelineStageRunner> {
        vec![
            MotorPipelineStageRunner::new(WrappedIOData::MidiOutputFrame(
                MidiOutputFrame::new(&properties()).unwrap(),
            ))
            .unwrap(),
        ]
    }

    /// `notes` and `expression` are (x, y, z, potential) voxels.
    fn frame_of(
        notes: &[(u32, u32, u32, f32)],
        expression: &[(u32, u32, u32, f32)],
    ) -> CorticalMappedXYZPNeuronVoxels {
        let mut map = CorticalMappedXYZPNeuronVoxels::new();
        for (id, voxels) in [(notes_id(), notes), (expression_id(), expression)] {
            if voxels.is_empty() {
                continue;
            }
            let mut arrays = NeuronVoxelXYZPArrays::new();
            for &(x, y, z, p) in voxels {
                arrays.push_raw(x, y, z, p);
            }
            map.insert(id, arrays);
        }
        map
    }

    fn decode(
        decoder: &mut Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>,
        neurons: &CorticalMappedXYZPNeuronVoxels,
        pipelines: &mut Vec<MotorPipelineStageRunner>,
    ) -> MidiOutputFrame {
        let mut changed = vec![false; pipelines.len()];
        decoder
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                neurons,
                Instant::now(),
                pipelines,
                &mut changed,
            )
            .unwrap();
        assert!(
            changed.iter().all(|c| *c),
            "every decode marks channels changed"
        );
        match pipelines[0].get_postprocessed_motor_value() {
            WrappedIOData::MidiOutputFrame(frame) => frame.clone(),
            other => panic!("expected MidiOutputFrame, got {:?}", other),
        }
    }

    #[test]
    fn midi_notes_silent_frame_after_non_silent_decodes_to_all_zeros() {
        let mut d = decoder();
        let mut p = pipelines();
        let loud = decode(&mut d, &frame_of(&[(60, 0, 0, 204.0)], &[]), &mut p);
        assert_eq!(loud.notes.strength(60), 204.0 / 255.0);
        let silent = decode(&mut d, &CorticalMappedXYZPNeuronVoxels::new(), &mut p);
        assert!(silent.notes.strengths().iter().all(|s| *s == 0.0));
        assert_eq!(silent.notes.sounding_count(), 0);
    }

    #[test]
    fn midi_notes_three_columns_decode_to_exactly_those_strengths() {
        let mut d = decoder();
        let mut p = pipelines();
        let frame = decode(
            &mut d,
            &frame_of(&[(48, 0, 0, 64.0), (60, 0, 0, 128.0), (72, 0, 0, 192.0)], &[]),
            &mut p,
        );
        assert_eq!(frame.notes.sounding_count(), 3);
        assert_eq!(frame.notes.strength(48), 64.0 / 255.0);
        assert_eq!(frame.notes.strength(60), 128.0 / 255.0);
        assert_eq!(frame.notes.strength(72), 192.0 / 255.0);
    }

    #[test]
    fn midi_notes_potential_scale_is_zero_to_255() {
        let mut d = decoder();
        let mut p = pipelines();
        for (potential, expected) in [
            (255.0_f32, 1.0_f32),
            (128.0, 128.0 / 255.0),
            (1.0, 1.0 / 255.0),
            (300.0, 1.0),
            (100.0, 100.0 / 255.0),
        ] {
            let frame = decode(&mut d, &frame_of(&[(5, 0, 0, potential)], &[]), &mut p);
            assert_eq!(frame.notes.strength(5), expected, "potential {potential}");
        }
    }

    #[test]
    fn midi_notes_non_positive_or_non_finite_potential_is_silent() {
        let mut d = decoder();
        let mut p = pipelines();
        let frame = decode(
            &mut d,
            &frame_of(&[(1, 0, 0, 0.0), (2, 0, 0, -5.0), (3, 0, 0, f32::NAN)], &[]),
            &mut p,
        );
        assert_eq!(frame.notes.sounding_count(), 0);
    }

    #[test]
    fn midi_notes_y_one_voxel_and_other_out_of_range_voxels_are_ignored() {
        let mut d = decoder();
        let mut p = pipelines();
        let frame = decode(
            &mut d,
            &frame_of(&[(COLUMNS, 0, 0, 230.0), (3, 1, 0, 230.0), (4, 0, 1, 230.0)], &[]),
            &mut p,
        );
        assert_eq!(frame.notes.sounding_count(), 0);
    }

    #[test]
    fn midi_notes_duplicate_cell_keeps_largest_potential() {
        let mut d = decoder();
        let mut p = pipelines();
        let frame = decode(
            &mut d,
            &frame_of(&[(7, 0, 0, 77.0), (7, 0, 0, 230.0), (7, 0, 0, 128.0)], &[]),
            &mut p,
        );
        assert_eq!(frame.notes.strength(7), 230.0 / 255.0);
    }

    /// The level the existing unsigned percentage decoder returns for the same Z pattern at y = 0.
    fn percentage_decoder_level(z: u32) -> f32 {
        use crate::data_types::descriptors::PercentageChannelDimensionality;
        use crate::neuron_voxel_coding::xyzp::decoders::PercentageNeuronVoxelXYZPDecoder;
        let mut reference = PercentageNeuronVoxelXYZPDecoder::new_box(
            expression_id(),
            feagi_structures::genomic::cortical_area::descriptors::NeuronDepth::new(DEPTH).unwrap(),
            CorticalChannelCount::new(1).unwrap(),
            PercentageNeuronPositioning::Linear,
            false,
            PercentageChannelDimensionality::D1,
        )
        .unwrap();
        let mut p =
            vec![
                MotorPipelineStageRunner::new(WrappedIOData::Percentage(Percentage::new_zero()))
                    .unwrap(),
            ];
        let mut changed = vec![false];
        reference
            .read_neuron_data_multi_channel_into_pipeline_input_cache(
                &frame_of(&[], &[(0, 0, z, 1.0)]),
                Instant::now(),
                &mut p,
                &mut changed,
            )
            .unwrap();
        match p[0].get_postprocessed_motor_value() {
            WrappedIOData::Percentage(v) => v.get_as_0_1(),
            other => panic!("expected Percentage, got {:?}", other),
        }
    }

    #[test]
    fn midi_expression_level_on_modulation_matches_percentage_decoder() {
        for z in 0..DEPTH {
            let mut d = decoder();
            let mut p = pipelines();
            let modulation = MidiControl::Modulation.column() as u32;
            let frame = decode(&mut d, &frame_of(&[], &[(modulation, 0, z, 1.0)]), &mut p);
            let expected = percentage_decoder_level(z);
            assert_eq!(
                frame.expression.level(MidiControl::Modulation),
                Some(expected),
                "z={z}"
            );
            assert_eq!(frame.expression.driven_count(), 1, "z={z}");
            for control in MidiControl::ALL {
                if control != MidiControl::Modulation {
                    assert_eq!(frame.expression.level(control), None);
                }
            }
        }
    }

    #[test]
    fn midi_expression_silent_frame_after_driven_decodes_to_all_none() {
        let mut d = decoder();
        let mut p = pipelines();
        let driven = decode(
            &mut d,
            &frame_of(&[], &[(MidiControl::Volume.column() as u32, 0, 5, 1.0)]),
            &mut p,
        );
        assert!(driven.expression.level(MidiControl::Volume).is_some());
        let silent = decode(&mut d, &CorticalMappedXYZPNeuronVoxels::new(), &mut p);
        assert_eq!(silent.expression.driven_count(), 0);
        assert!(silent.expression.levels().iter().all(|l| l.is_none()));
    }

    #[test]
    fn midi_expression_out_of_range_voxel_is_ignored() {
        let mut d = decoder();
        let mut p = pipelines();
        let frame = decode(
            &mut d,
            &frame_of(&[], &[(5, 0, 0, 1.0), (0, 1, 0, 1.0), (0, 0, DEPTH, 1.0)]),
            &mut p,
        );
        assert_eq!(frame.expression.driven_count(), 0);
    }
}
