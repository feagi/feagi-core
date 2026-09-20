//! Binding profiles — the resolved, pinned mapping between a dataset's I/O and FEAGI areas.
//!
//! A profile names the cortical area(s) a selector targets plus the scheme/shape used. It is
//! produced when a run's binding is resolved and is recorded in provenance so a benchmark is
//! reproducible. The concrete cortical-area identifiers come from the pinned genome/connectome
//! (a design artifact), not from the Trainer.

use serde::{Deserialize, Serialize};

use crate::binding::encoding_scheme::EncodingScheme;
use crate::error::TrainerError;

/// Dual-IPU sample-by-sample presentation (amplitude Misc A + class-teacher Misc B).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamMode {
    /// Hold the class neuron on Misc B for every tick of the beat window.
    Train,
    /// Drive Misc A only. Misc B stays silent. Score at each `hold_end`.
    Infer,
}

/// Operator-set geometry for the streamed Misc A / Misc B pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamBinding {
    /// Parallel beat slots along X. Must be >= 1.
    pub parallel_width: u32,
    /// Sensory unit index for amplitude Misc A.
    pub amplitude_unit: u16,
    /// Sensory unit index for class-teacher Misc B.
    pub teacher_unit: u16,
    /// Provenance id for the teacher area (genome-owned circuit still maps B).
    pub teacher_cortical_area_id: String,
    /// Operator-visible genome title applied when Trainer creates teacher Misc B.
    #[serde(default)]
    pub teacher_cortical_name: Option<String>,
    /// Train holds the class; infer leaves B silent.
    pub mode: StreamMode,
}

/// Snapshot class-teacher Misc B (one beat, class held on Y).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassTeacherBinding {
    /// Sensory unit index for class-teacher Misc B.
    pub unit: u16,
    /// Provenance id for the teacher area.
    pub cortical_area_id: String,
    /// Operator-visible genome title applied when Trainer creates or renames Misc B.
    #[serde(default)]
    pub cortical_name: Option<String>,
    /// Class neurons along Y. Must match decoder_profile.class_count.
    pub class_count: u32,
}

impl ClassTeacherBinding {
    /// Returns blocking configuration errors. Never repairs values.
    pub fn validate(&self) -> Result<(), TrainerError> {
        if self.unit == 0 {
            return Err(TrainerError::Config(
                "snapshot teacher unit must be distinct from magnitude Misc unit 0".to_string(),
            ));
        }
        if self.cortical_area_id.trim().is_empty() {
            return Err(TrainerError::Config(
                "snapshot teacher cortical_area_id must be non-empty".to_string(),
            ));
        }
        if self.class_count == 0 {
            return Err(TrainerError::Config(
                "snapshot teacher class_count must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

impl StreamBinding {
    /// Returns blocking configuration errors. Never repairs values.
    pub fn validate(&self) -> Result<(), TrainerError> {
        if self.parallel_width == 0 {
            return Err(TrainerError::Config(
                "stream parallel_width must be >= 1".to_string(),
            ));
        }
        if self.amplitude_unit == self.teacher_unit {
            return Err(TrainerError::Config(
                "stream amplitude_unit and teacher_unit must be distinct Misc units".to_string(),
            ));
        }
        if self.teacher_cortical_area_id.trim().is_empty() {
            return Err(TrainerError::Config(
                "stream teacher_cortical_area_id must be non-empty".to_string(),
            ));
        }
        if self.mode == StreamMode::Infer && self.parallel_width != 1 {
            return Err(TrainerError::Config(
                "stream infer requires parallel_width = 1".to_string(),
            ));
        }
        Ok(())
    }
}

/// Mask teacher IPU for image segmentation (`iseg`, W×H×C, Z = class).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentationTeacherBinding {
    /// Provenance id for the iseg area (genome-owned circuit still maps it).
    pub cortical_area_id: String,
    /// Operator-visible genome title applied when Trainer creates the iseg IPU.
    #[serde(default)]
    pub cortical_name: Option<String>,
    /// Mask width in pixels. Must match the sample `SegmentationMask`.
    pub mask_width: u32,
    /// Mask height in pixels. Must match the sample `SegmentationMask`.
    pub mask_height: u32,
    /// Surviving class count along Z. Pixel class ids must be `< mask_depth`.
    pub mask_depth: u32,
}

impl SegmentationTeacherBinding {
    /// Returns blocking configuration errors. Never repairs values.
    pub fn validate(&self) -> Result<(), TrainerError> {
        if self.cortical_area_id.trim().is_empty() {
            return Err(TrainerError::Config(
                "segmentation teacher cortical_area_id must be non-empty".to_string(),
            ));
        }
        if self.mask_width == 0 || self.mask_height == 0 || self.mask_depth == 0 {
            return Err(TrainerError::Config(
                "segmentation teacher mask_width, mask_height, and mask_depth must be > 0"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

/// How an encoder selector maps sample features onto a FEAGI sensory (IPU) area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncoderBindingProfile {
    /// Identifier of the target sensory cortical area (from the pinned genome).
    pub cortical_area_id: String,
    /// Operator-visible genome title applied when Trainer creates this IPU.
    #[serde(default)]
    pub cortical_name: Option<String>,
    /// Number of channels (e.g. one per scalar feature, or `1` for a single vision channel).
    pub channels: u32,
    /// The neural encoding scheme to apply.
    pub scheme: EncodingScheme,
    /// Vision feed width in pixels when using an image-frame encoder.
    #[serde(default)]
    pub image_width: Option<u32>,
    /// Vision feed height in pixels when using an image-frame encoder.
    #[serde(default)]
    pub image_height: Option<u32>,
    /// Sample-by-sample dual-Misc presentation. `None` is the snapshot path.
    #[serde(default)]
    pub stream: Option<StreamBinding>,
    /// Snapshot class-teacher Misc B. Required for ECG snapshot; unused for stream.
    #[serde(default)]
    pub teacher: Option<ClassTeacherBinding>,
    /// Object-segmentation mask teacher (`iseg`). Required for image-folder segmentation.
    #[serde(default)]
    pub segmentation_teacher: Option<SegmentationTeacherBinding>,
}

/// How a decoder selector reads a FEAGI motor (OPU) area into a typed prediction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecoderBindingProfile {
    /// Identifier of the source motor cortical area (from the pinned genome).
    pub cortical_area_id: String,
    /// Operator-visible genome title applied when Trainer creates this OPU.
    #[serde(default)]
    pub cortical_name: Option<String>,
    /// Number of distinct classes (= channels, one channel per class for classification).
    pub class_count: u32,
    /// Cortical-column depth (number of bins) of the motor area, used to decode each
    /// channel's activation. Must match the pinned genome's OPU area depth.
    pub bins: u32,
    /// Segmentation mask width when using a misc-data object-segmentation decoder.
    #[serde(default)]
    pub mask_width: Option<u32>,
    /// Segmentation mask height when using a misc-data object-segmentation decoder.
    #[serde(default)]
    pub mask_height: Option<u32>,
    /// Segmentation class depth (Z dimension) when using a misc-data decoder.
    #[serde(default)]
    pub mask_depth: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::encoding_scheme::EncodingScheme;

    #[test]
    fn missing_cortical_names_deserialize_as_none() {
        let encoder: EncoderBindingProfile = serde_json::from_str(
            r#"{
                "cortical_area_id": "misc_input",
                "channels": 4,
                "scheme": { "scheme": "population_single_spike", "bins": 1, "spacing": "linear" }
            }"#,
        )
        .expect("encoder");
        assert_eq!(encoder.cortical_name, None);

        let decoder: DecoderBindingProfile = serde_json::from_str(
            r#"{
                "cortical_area_id": "count_output",
                "class_count": 3,
                "bins": 1
            }"#,
        )
        .expect("decoder");
        assert_eq!(decoder.cortical_name, None);
    }

    #[test]
    fn cortical_names_roundtrip() {
        let encoder = EncoderBindingProfile {
            cortical_area_id: "misc_input".to_string(),
            cortical_name: Some("ECG Amplitude IPU".to_string()),
            channels: 1,
            scheme: EncodingScheme::Value,
            image_width: None,
            image_height: None,
            stream: Some(StreamBinding {
                parallel_width: 1,
                amplitude_unit: 0,
                teacher_unit: 1,
                teacher_cortical_area_id: "misc_input_teacher".to_string(),
                teacher_cortical_name: Some("ECG Class Teacher IPU".to_string()),
                mode: StreamMode::Train,
            }),
            teacher: None,
            segmentation_teacher: None,
        };
        let json = serde_json::to_string(&encoder).expect("json");
        let parsed: EncoderBindingProfile = serde_json::from_str(&json).expect("parse");
        assert_eq!(parsed.cortical_name.as_deref(), Some("ECG Amplitude IPU"));
        assert_eq!(
            parsed
                .stream
                .as_ref()
                .and_then(|stream| stream.teacher_cortical_name.as_deref()),
            Some("ECG Class Teacher IPU")
        );
    }
}
