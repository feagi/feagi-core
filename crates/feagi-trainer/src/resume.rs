//! Resume contract shared by the run engines ("pause and save, resume later").
//!
//! A run paused at an attempt / sample / batch boundary reports where it stopped
//! ([`ResumeExport`]); a later session passes that back as [`ResumeFrom`] and the engine
//! continues at the same unit index with the same seeds. The unit is whatever the engine
//! iterates (an attempt, a sample, a batch). `engine_state` is opaque JSON owned by the engine
//! that exported it; engines with no state beyond the unit index leave it `Null`.
//!
//! The default [`ResumeFrom`] starts at unit 0 with no state, so callers that do not resume
//! behave exactly as before.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::TrainerError;

/// Where a resumed run starts.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ResumeFrom {
    /// Zero-based index of the first unit to run (units before it were done by earlier segments).
    pub first_index: u64,
    /// State exported by the engine at the pause (`Null` when none).
    #[serde(default)]
    pub engine_state: Value,
    /// Pause-save mode: a skip request is honoured only at the unit boundary (the batch or the
    /// hold), never in the middle of a unit. Engines that already stop at a boundary ignore it.
    #[serde(default)]
    pub boundary_stop: bool,
}

impl ResumeFrom {
    /// Resume from the export of an earlier segment.
    pub fn from_export(export: &ResumeExport) -> Self {
        Self {
            first_index: export.units_done,
            engine_state: export.engine_state.clone(),
            boundary_stop: false,
        }
    }

    /// True for the default (start at 0, no state).
    pub fn is_fresh(&self) -> bool {
        self.first_index == 0 && self.engine_state.is_null() && !self.boundary_stop
    }

    /// Checks `first_index` against the number of units planned.
    pub fn check_within(&self, total_units: u64, what: &str) -> Result<(), TrainerError> {
        if self.first_index >= total_units && total_units > 0 {
            return Err(TrainerError::Config(format!(
                "resume index {} is past the last {what} ({total_units} planned)",
                self.first_index
            )));
        }
        Ok(())
    }

    /// Resume with a stop that is honoured only at the unit boundary (see [`Self::boundary_stop`]).
    pub fn at_boundary(mut self) -> Self {
        self.boundary_stop = true;
        self
    }

    /// Decodes `engine_state` into the engine's own state type (`None` when empty).
    pub fn decode_state<T: serde::de::DeserializeOwned>(&self) -> Result<Option<T>, TrainerError> {
        if self.engine_state.is_null() {
            return Ok(None);
        }
        serde_json::from_value(self.engine_state.clone())
            .map(Some)
            .map_err(|e| TrainerError::Config(format!("resume engine_state is not usable: {e}")))
    }
}

/// What an engine hands back when a segment ends (paused, or finished).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ResumeExport {
    /// Units done in total, counting earlier segments (the next segment's `first_index`).
    pub units_done: u64,
    /// Engine state to pass back (`Null` when the engine has none).
    #[serde(default)]
    pub engine_state: Value,
    /// True when the segment ended before every planned unit ran (a pause-stop or a skip). A
    /// partial segment is not a finished run: hosts keep its scorecard out of history until the
    /// later segments are merged (or the run is ended on purpose).
    #[serde(default)]
    pub partial: bool,
}

impl ResumeExport {
    /// Export with no engine state.
    pub fn at(units_done: u64) -> Self {
        Self {
            units_done,
            engine_state: Value::Null,
            partial: false,
        }
    }
}

/// Engine state of the dataset and stream engines: the predictions of the earlier segments.
///
/// Final metrics are computed over every prediction of the run, so carrying the records (and
/// scoring the merged list at the end) makes a split run equal an unsplit one for any metric
/// pack. The cost is a state that grows with the number of scored samples.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PredictionState {
    /// Prediction records of the segments already run, in visit order.
    pub predictions: Vec<crate::contracts::PredictionRecord>,
}

impl PredictionState {
    /// Restores the earlier segments' records (empty for a fresh run).
    pub fn restore(
        resume: &ResumeFrom,
    ) -> Result<Vec<crate::contracts::PredictionRecord>, TrainerError> {
        Ok(resume
            .decode_state::<PredictionState>()?
            .map(|state| state.predictions)
            .unwrap_or_default())
    }

    /// Scored pairs (those with a target) of `records`, in order.
    pub fn scored(
        records: &[crate::contracts::PredictionRecord],
    ) -> (
        Vec<crate::contracts::TypedPrediction>,
        Vec<crate::contracts::TypedTarget>,
    ) {
        records
            .iter()
            .filter_map(|r| r.target.clone().map(|t| (r.prediction.clone(), t)))
            .unzip()
    }

    /// Export for the next segment.
    pub fn export(
        units_done: u64,
        total_units: u64,
        records: &[crate::contracts::PredictionRecord],
    ) -> ResumeExport {
        ResumeExport {
            units_done,
            engine_state: serde_json::to_value(PredictionState {
                predictions: records.to_vec(),
            })
            .unwrap_or(Value::Null),
            partial: units_done < total_units,
        }
    }
}
