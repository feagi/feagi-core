//! In-memory analog corpus shared by the WFDB codec and the package reader.

use std::collections::BTreeMap;

/// One point event on an analog episode timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalogEvent {
    /// Sample index in the episode (0-based).
    pub sample_index: u64,
    /// Annotation symbol (WFDB `anntyp` string or package label).
    pub label: String,
}

/// One continuous multi-channel recording.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalogEpisode {
    /// Stable episode id (WFDB record name or package episode id).
    pub episode_id: String,
    /// Sampling rate in Hz; every stream in the episode shares this rate.
    pub sample_rate_hz: f64,
    /// Physical-unit samples keyed by portable stream id.
    pub streams: BTreeMap<String, Vec<f32>>,
    /// Point events aligned to `sample_index`.
    pub events: Vec<AnalogEvent>,
}

impl AnalogEpisode {
    /// Number of samples in each stream; errors if streams disagree or are empty.
    pub fn sample_count(&self) -> Result<u64, String> {
        let mut counts = self.streams.values().map(Vec::len);
        let first = counts
            .next()
            .ok_or_else(|| format!("episode '{}': no streams", self.episode_id))?;
        if first == 0 {
            return Err(format!("episode '{}': empty stream", self.episode_id));
        }
        for len in counts {
            if len != first {
                return Err(format!(
                    "episode '{}': stream lengths differ",
                    self.episode_id
                ));
            }
        }
        Ok(first as u64)
    }
}
