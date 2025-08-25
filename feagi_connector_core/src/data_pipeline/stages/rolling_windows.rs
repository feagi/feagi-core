//! Rolling window processing for temporal smoothing of data streams.
//!
//! This module provides processing that maintain a sliding window of recent values
//! and compute statistics (like averages) over that window. These processing are
//! useful for smoothing noisy sensor data or computing temporal aggregations.

use std::fmt::{Display, Formatter};
use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::stream_cache_processor_trait::StreamCacheStage;

/// A stream processor that maintains a rolling window of float values and outputs their average.
///
/// This processor keeps track of the last N values (where N is the window length) and
/// continuously outputs the arithmetic mean of these values. When a new value arrives,
/// it replaces the oldest value in the window using a circular buffer approach.
#[derive(Debug, Clone)]
pub struct LinearAverageRollingWindowStage {
    previous_value: WrappedIOData,
    last_index: usize,
    window: Vec<f32>,
    window_length: f32 // While this is treated an int, we do divisions so we keep this as a f32
}

impl Display for LinearAverageRollingWindowStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LinearAverageRollingWindowProcessor(length={:?},prev_val={:?})", self.window.len(),  self.previous_value)
    }
}

impl StreamCacheStage for LinearAverageRollingWindowStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::F32
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::F32
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.previous_value
    }

    fn process_new_input(&mut self, value: &WrappedIOData, _: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        let float_result = f32::try_from(value)?;
        let new_index = (self.last_index + 1) % self.window.len();
        self.window[new_index] = float_result;

        self.last_index += 1;

        self.previous_value = WrappedIOData::F32(self.window.iter().sum::<f32>() / self.window_length); // average
        Ok(&self.previous_value)
    }
}

impl LinearAverageRollingWindowStage {
    /// Creates a new LinearAverageRollingWindowProcessor.
    ///
    /// # Arguments
    /// * `window_length` - The size of the rolling window (must be > 0)
    /// * `initial_value` - The initial value to fill the window with (must be finite)
    ///
    /// # Returns
    /// * `Ok(LinearAverageRollingWindowProcessor)` - A new processor instance with window filled with initial_value
    /// * `Err(FeagiDataError)` - If window_length is 0 or initial_value is invalid (NaN/infinite)
    pub fn new(window_length: usize, initial_value: f32) -> Result<Self, FeagiDataError> {
        if initial_value.is_nan() || initial_value.is_infinite() {
            return Err(FeagiDataError::BadParameters(format!("Given float {} is not valid!", initial_value)).into());
        }
        if window_length == 0 {
            return Err(FeagiDataError::BadParameters(format!("Window length cannot be 0!")).into());
        }

        Ok(LinearAverageRollingWindowStage {
            previous_value: WrappedIOData::F32(initial_value),
            last_index: 0,
            window: vec![initial_value; window_length],
            window_length: window_length as f32
        })
    }
}