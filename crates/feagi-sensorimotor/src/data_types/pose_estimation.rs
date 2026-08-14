use super::descriptors::PoseEstimationProperties;


use feagi_data::feagi_data_error::{FeagiDataError, FeagiFailDataEtc};

fn feagi_data_etc_error(message: String) -> FeagiDataError {
    let context: &'static str = Box::leak(message.into_boxed_str());
    FeagiFailDataEtc::new(context).into()
}

/// A single decoded joint position with confidence.
///
/// Represents one keypoint in a skeleton after centroid extraction from neuron activations.
/// The `x` and `y` are normalized to [0.0, 1.0] representing the position within the
/// cortical_area area's XY plane. `confidence` is derived from the mean PSP magnitude of
/// contributing neurons, also normalized to [0.0, 1.0].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointPosition {
    pub x: f32,
    pub y: f32,
    pub confidence: f32,
}

/// Pose estimation output data containing per-joint positions.
///
/// Each slot in `joints` corresponds to a Z-layer index (joint ID).
/// `None` means no valid detection for that joint in this frame (neurons were either
/// silent or too spatially scattered to form a coherent cluster).
///
/// The pose schema (HumanBody, HumanHand, etc.) is encoded in the cortical_area ID itself
/// and does not need to be stored in this data structure.
#[derive(Clone, Debug, PartialEq)]
pub struct PoseEstimationData {
    joints: Vec<Option<JointPosition>>,
    properties: PoseEstimationProperties,
}

impl PoseEstimationData {
    pub fn new(properties: &PoseEstimationProperties) -> Result<Self, FeagiDataError> {
        if properties.depth == 0 || properties.width == 0 || properties.height == 0 {
            return Err(feagi_data_etc_error(
                "PoseEstimationData dimensions must all be non-zero".into(),
            ));
        }
        Ok(PoseEstimationData {
            joints: vec![None; properties.depth as usize],
            properties: *properties,
        })
    }

    pub fn get_properties(&self) -> &PoseEstimationProperties {
        &self.properties
    }

    pub fn get_joints(&self) -> &[Option<JointPosition>] {
        &self.joints
    }

    pub fn get_joints_mut(&mut self) -> &mut [Option<JointPosition>] {
        &mut self.joints
    }

    pub fn get_joint(&self, joint_index: usize) -> Option<&JointPosition> {
        self.joints.get(joint_index)?.as_ref()
    }

    pub fn set_joint(&mut self, joint_index: usize, position: Option<JointPosition>) {
        if joint_index < self.joints.len() {
            self.joints[joint_index] = position;
        }
    }

    pub fn clear_all_joints(&mut self) {
        for joint in self.joints.iter_mut() {
            *joint = None;
        }
    }

    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    pub fn detected_joint_count(&self) -> usize {
        self.joints.iter().filter(|j| j.is_some()).count()
    }
}

impl std::fmt::Display for PoseEstimationData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "PoseEstimationData(joints={}, detected={})",
            self.joint_count(),
            self.detected_joint_count(),
        )
    }
}
