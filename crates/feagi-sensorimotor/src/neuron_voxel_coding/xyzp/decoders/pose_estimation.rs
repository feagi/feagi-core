use crate::configuration::jsonable::JSONDecoderProperties;
use crate::data_pipeline::per_channel_stream_caches::MotorPipelineStageRunner;
use crate::data_types::descriptors::PoseEstimationProperties;
use crate::data_types::{JointPosition, PoseEstimationData};
use crate::neuron_voxel_coding::xyzp::NeuronVoxelXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_genomic_context::cortical_area::descriptors::CorticalChannelCount;
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::time::Instant;

/// Maximum distance (in normalized [0,1] units) between two activated neurons
/// for them to be considered part of the same spatial cluster within a Z layer.
const CLUSTER_RADIUS_NORMALIZED: f32 = 0.15;

#[derive(Debug)]
pub struct PoseEstimationNeuronVoxelXYZPDecoder {
    cortical_read_target: CorticalID,
    properties: PoseEstimationProperties,
}

/// Temporary per-neuron activation record for centroid computation.
struct NeuronActivation {
    x_normalized: f32,
    y_normalized: f32,
    psp: f32,
}

impl NeuronVoxelXYZPDecoder for PoseEstimationNeuronVoxelXYZPDecoder {
    fn get_decodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::PoseEstimationData(Some(self.properties))
    }

    fn get_as_properties(&self) -> JSONDecoderProperties {
        JSONDecoderProperties::PoseEstimation(self.properties)
    }

    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        _time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError> {
        let neuron_array = neurons_to_read.get_neurons_of(&self.cortical_read_target);

        if neuron_array.is_none() {
            return Ok(());
        }

        let neuron_array = neuron_array.unwrap();
        if neuron_array.is_empty() {
            return Ok(());
        }

        let number_of_channels = pipelines_with_data_to_update.len() as u32;
        let per_channel_width = self.properties.width;
        let max_possible_x = per_channel_width * number_of_channels;
        let max_possible_y = self.properties.height;
        let max_possible_z = self.properties.depth;

        let depth = self.properties.depth as usize;

        for (channel_idx, pipeline_runner) in pipelines_with_data_to_update
            .iter_mut()
            .enumerate()
            .take(number_of_channels as usize)
        {
            let x_offset = (channel_idx as u32) * per_channel_width;
            let x_end = x_offset + per_channel_width;

            let mut layer_activations: Vec<Vec<NeuronActivation>> =
                (0..depth).map(|_| Vec::new()).collect();

            for neuron in neuron_array.iter() {
                let nx = neuron.neuron_voxel_coordinate.x;
                let ny = neuron.neuron_voxel_coordinate.y;
                let nz = neuron.neuron_voxel_coordinate.z;

                if nx >= max_possible_x || ny >= max_possible_y || nz >= max_possible_z {
                    continue;
                }
                if nx < x_offset || nx >= x_end {
                    continue;
                }

                let in_channel_x = nx - x_offset;
                let z_layer = nz as usize;

                layer_activations[z_layer].push(NeuronActivation {
                    x_normalized: (in_channel_x as f32) / (per_channel_width as f32 - 1.0).max(1.0),
                    y_normalized: (ny as f32) / (max_possible_y as f32 - 1.0).max(1.0),
                    psp: neuron.potential.abs().clamp(0.0, 1.0),
                });
            }

            let pose_data: &mut PoseEstimationData = pipeline_runner
                .get_preprocessed_cached_value_mut()
                .try_into()?;

            if !channel_changed[channel_idx] {
                pose_data.clear_all_joints();
                channel_changed[channel_idx] = true;
            }

            for (z_layer, activations) in layer_activations.iter().enumerate().take(depth) {
                let joint = extract_centroid_from_layer(activations);
                pose_data.set_joint(z_layer, joint);
            }
        }

        Ok(())
    }
}

/// Extracts a single joint position from a set of neuron activations within one Z layer.
///
/// Algorithm:
/// 1. If no activations exist, returns None.
/// 2. Computes the PSP-weighted centroid of all activations.
/// 3. Checks spatial coherence: all points must be within `CLUSTER_RADIUS_NORMALIZED`
///    of the centroid. If any point is outside this radius, the cluster is considered
///    incoherent and the result is discarded (returns None).
/// 4. Returns the weighted centroid position with mean PSP as confidence.
fn extract_centroid_from_layer(activations: &[NeuronActivation]) -> Option<JointPosition> {
    if activations.is_empty() {
        return None;
    }

    if activations.len() == 1 {
        let a = &activations[0];
        return Some(JointPosition {
            x: a.x_normalized,
            y: a.y_normalized,
            confidence: a.psp,
        });
    }

    let total_weight: f32 = activations.iter().map(|a| a.psp).sum();
    if total_weight <= 0.0 {
        return None;
    }

    let centroid_x: f32 = activations
        .iter()
        .map(|a| a.x_normalized * a.psp)
        .sum::<f32>()
        / total_weight;
    let centroid_y: f32 = activations
        .iter()
        .map(|a| a.y_normalized * a.psp)
        .sum::<f32>()
        / total_weight;

    for a in activations {
        let dx = a.x_normalized - centroid_x;
        let dy = a.y_normalized - centroid_y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq > CLUSTER_RADIUS_NORMALIZED * CLUSTER_RADIUS_NORMALIZED {
            return None;
        }
    }

    let mean_confidence = total_weight / activations.len() as f32;

    Some(JointPosition {
        x: centroid_x,
        y: centroid_y,
        confidence: mean_confidence,
    })
}

impl PoseEstimationNeuronVoxelXYZPDecoder {
    pub fn new_box(
        cortical_read_target: CorticalID,
        properties: PoseEstimationProperties,
        _number_of_channels: CorticalChannelCount,
    ) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        let decoder = PoseEstimationNeuronVoxelXYZPDecoder {
            cortical_read_target,
            properties,
        };
        Ok(Box::new(decoder))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_activations_returns_none() {
        let result = extract_centroid_from_layer(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn single_activation_returns_position() {
        let activations = vec![NeuronActivation {
            x_normalized: 0.5,
            y_normalized: 0.3,
            psp: 0.8,
        }];
        let result = extract_centroid_from_layer(&activations).unwrap();
        assert!((result.x - 0.5).abs() < 1e-6);
        assert!((result.y - 0.3).abs() < 1e-6);
        assert!((result.confidence - 0.8).abs() < 1e-6);
    }

    #[test]
    fn nearby_activations_produce_weighted_centroid() {
        let activations = vec![
            NeuronActivation {
                x_normalized: 0.50,
                y_normalized: 0.50,
                psp: 1.0,
            },
            NeuronActivation {
                x_normalized: 0.52,
                y_normalized: 0.48,
                psp: 0.5,
            },
        ];
        let result = extract_centroid_from_layer(&activations).unwrap();
        let expected_x = (0.50 * 1.0 + 0.52 * 0.5) / 1.5;
        let expected_y = (0.50 * 1.0 + 0.48 * 0.5) / 1.5;
        assert!((result.x - expected_x).abs() < 1e-5);
        assert!((result.y - expected_y).abs() < 1e-5);
    }

    #[test]
    fn scattered_activations_return_none() {
        let activations = vec![
            NeuronActivation {
                x_normalized: 0.1,
                y_normalized: 0.1,
                psp: 1.0,
            },
            NeuronActivation {
                x_normalized: 0.9,
                y_normalized: 0.9,
                psp: 1.0,
            },
        ];
        let result = extract_centroid_from_layer(&activations);
        assert!(result.is_none());
    }

    #[test]
    fn zero_psp_activations_return_none() {
        let activations = vec![
            NeuronActivation {
                x_normalized: 0.5,
                y_normalized: 0.5,
                psp: 0.0,
            },
            NeuronActivation {
                x_normalized: 0.5,
                y_normalized: 0.5,
                psp: 0.0,
            },
        ];
        let result = extract_centroid_from_layer(&activations);
        assert!(result.is_none());
    }
}
