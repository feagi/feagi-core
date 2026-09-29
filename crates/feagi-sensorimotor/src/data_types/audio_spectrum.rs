//! Audio spectrum frame and its registration properties.
//!
//! One frame is one FEAGI tick of audio. X is the frequency column, Y is the
//! quantized phase step, and membrane potential is the column magnitude on the
//! registered decibel scale. A column with potential 0 is silent and emits no
//! neuron.

use feagi_structures::FeagiDataError;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Largest FFT window accepted. Keeps linear columns within the template X maximum.
pub const AUDIO_SPECTRUM_MAX_WINDOW_SIZE: u32 = 8192;
/// Smallest FFT window accepted (two linear columns).
pub const AUDIO_SPECTRUM_MIN_WINDOW_SIZE: u32 = 4;
/// Largest phase resolution accepted (template Y maximum).
pub const AUDIO_SPECTRUM_MAX_PHASE_STEPS: u32 = 256;
/// Largest column count accepted (template X maximum).
pub const AUDIO_SPECTRUM_MAX_BIN_COUNT: u32 = AUDIO_SPECTRUM_MAX_WINDOW_SIZE / 2 + 1;

/// How FFT bins are laid out along X.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioFrequencySpacing {
    /// One column per FFT bin from 0 Hz to Nyquist. The only layout that can
    /// rebuild the input waveform.
    Linear,
    /// Columns spaced evenly in pitch between the registered minimum and
    /// maximum frequency. Several FFT bins share a column, so playback is a
    /// resynthesis, not the input waveform.
    Logarithmic,
}

impl Display for AudioFrequencySpacing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFrequencySpacing::Linear => write!(f, "Linear"),
            AudioFrequencySpacing::Logarithmic => write!(f, "Logarithmic"),
        }
    }
}

/// Registration properties for an audio spectrum cortical area.
///
/// Carried in the agent's device registration so FEAGI sizes the area as
/// `bin_count x phase_steps x 1` and the receiving side can decode with the
/// same scale. Frequencies are whole hertz and levels are whole decibels so the
/// type stays `Eq + Hash` like every other wrapped IO descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioSpectrumProperties {
    /// Columns per channel (area X width).
    pub bin_count: u32,
    /// Phase resolution (area Y height). 1 carries magnitude only.
    pub phase_steps: u32,
    pub sample_rate_hz: u32,
    /// FFT length in samples. Must be a power of two.
    pub window_size: u32,
    /// Samples consumed per frame (one FEAGI tick). At most half the window.
    pub hop_size: u32,
    pub spacing: AudioFrequencySpacing,
    pub min_frequency_hz: u32,
    pub max_frequency_hz: u32,
    /// Level mapped to potential 0. Quieter columns emit no neuron.
    pub magnitude_floor_db: i32,
    /// Level mapped to potential 1. Louder columns saturate.
    pub magnitude_ceiling_db: i32,
}

impl AudioSpectrumProperties {
    /// Linear layout: one column per FFT bin, 0 Hz through Nyquist.
    pub fn new_linear(
        sample_rate_hz: u32,
        window_size: u32,
        hop_size: u32,
        phase_steps: u32,
        magnitude_floor_db: i32,
        magnitude_ceiling_db: i32,
    ) -> Result<Self, FeagiDataError> {
        let properties = AudioSpectrumProperties {
            bin_count: window_size / 2 + 1,
            phase_steps,
            sample_rate_hz,
            window_size,
            hop_size,
            spacing: AudioFrequencySpacing::Linear,
            min_frequency_hz: 0,
            max_frequency_hz: sample_rate_hz / 2,
            magnitude_floor_db,
            magnitude_ceiling_db,
        };
        properties.validate()?;
        Ok(properties)
    }

    /// Logarithmic layout: `bin_count` columns spread evenly in pitch.
    #[allow(clippy::too_many_arguments)]
    pub fn new_logarithmic(
        sample_rate_hz: u32,
        window_size: u32,
        hop_size: u32,
        bin_count: u32,
        phase_steps: u32,
        min_frequency_hz: u32,
        max_frequency_hz: u32,
        magnitude_floor_db: i32,
        magnitude_ceiling_db: i32,
    ) -> Result<Self, FeagiDataError> {
        let properties = AudioSpectrumProperties {
            bin_count,
            phase_steps,
            sample_rate_hz,
            window_size,
            hop_size,
            spacing: AudioFrequencySpacing::Logarithmic,
            min_frequency_hz,
            max_frequency_hz,
            magnitude_floor_db,
            magnitude_ceiling_db,
        };
        properties.validate()?;
        Ok(properties)
    }

    /// Reject any combination that cannot be encoded, or on the linear layout
    /// cannot be rebuilt into a waveform.
    pub fn validate(&self) -> Result<(), FeagiDataError> {
        let bad = |message: String| Err(FeagiDataError::BadParameters(message));

        if self.sample_rate_hz == 0 {
            return bad("Audio sample rate must be greater than 0 Hz".into());
        }
        if !self.window_size.is_power_of_two()
            || self.window_size < AUDIO_SPECTRUM_MIN_WINDOW_SIZE
            || self.window_size > AUDIO_SPECTRUM_MAX_WINDOW_SIZE
        {
            return bad(format!(
                "Audio window size {} must be a power of two between {} and {}",
                self.window_size, AUDIO_SPECTRUM_MIN_WINDOW_SIZE, AUDIO_SPECTRUM_MAX_WINDOW_SIZE
            ));
        }
        if self.hop_size == 0 || self.hop_size > self.window_size / 2 {
            return bad(format!(
                "Audio hop of {} samples must be between 1 and half the {}-sample window. \
                 Raise the window size or the burst rate.",
                self.hop_size, self.window_size
            ));
        }
        if self.phase_steps == 0 || self.phase_steps > AUDIO_SPECTRUM_MAX_PHASE_STEPS {
            return bad(format!(
                "Audio phase steps {} must be between 1 and {}",
                self.phase_steps, AUDIO_SPECTRUM_MAX_PHASE_STEPS
            ));
        }
        if self.magnitude_floor_db >= self.magnitude_ceiling_db {
            return bad(format!(
                "Audio magnitude floor {} dB must be below the ceiling {} dB",
                self.magnitude_floor_db, self.magnitude_ceiling_db
            ));
        }
        let nyquist = self.sample_rate_hz / 2;
        match self.spacing {
            AudioFrequencySpacing::Linear => {
                if self.bin_count != self.window_size / 2 + 1 {
                    return bad(format!(
                        "Linear audio needs {} columns for a {}-sample window, got {}",
                        self.window_size / 2 + 1,
                        self.window_size,
                        self.bin_count
                    ));
                }
                if self.min_frequency_hz != 0 || self.max_frequency_hz != nyquist {
                    return bad(format!(
                        "Linear audio covers 0 Hz through {} Hz (Nyquist); got {}-{} Hz",
                        nyquist, self.min_frequency_hz, self.max_frequency_hz
                    ));
                }
            }
            AudioFrequencySpacing::Logarithmic => {
                if self.bin_count == 0 || self.bin_count > AUDIO_SPECTRUM_MAX_BIN_COUNT {
                    return bad(format!(
                        "Audio column count {} must be between 1 and {}",
                        self.bin_count, AUDIO_SPECTRUM_MAX_BIN_COUNT
                    ));
                }
                if self.min_frequency_hz == 0 {
                    return bad("Logarithmic audio minimum frequency must be above 0 Hz".into());
                }
                if self.min_frequency_hz >= self.max_frequency_hz {
                    return bad(format!(
                        "Audio minimum frequency {} Hz must be below the maximum {} Hz",
                        self.min_frequency_hz, self.max_frequency_hz
                    ));
                }
                if self.max_frequency_hz > nyquist {
                    return bad(format!(
                        "Audio maximum frequency {} Hz is above Nyquist ({} Hz) for a {} Hz sample rate",
                        self.max_frequency_hz, nyquist, self.sample_rate_hz
                    ));
                }
            }
        }
        Ok(())
    }

    /// True when decoding can rebuild the input waveform (linear layout only).
    pub fn is_waveform_invertible(&self) -> bool {
        self.spacing == AudioFrequencySpacing::Linear
    }

    /// Hertz spanned by one FFT bin.
    pub fn fft_bin_width_hz(&self) -> f32 {
        self.sample_rate_hz as f32 / self.window_size as f32
    }
}

impl Display for AudioSpectrumProperties {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AudioSpectrum({}x{} {} {}-{} Hz, {} Hz, window {}, hop {}, {}..{} dB)",
            self.bin_count,
            self.phase_steps,
            self.spacing,
            self.min_frequency_hz,
            self.max_frequency_hz,
            self.sample_rate_hz,
            self.window_size,
            self.hop_size,
            self.magnitude_floor_db,
            self.magnitude_ceiling_db
        )
    }
}

/// One tick of audio as columns of (magnitude potential, phase step).
#[derive(Clone, Debug, PartialEq)]
pub struct AudioSpectrumFrame {
    properties: AudioSpectrumProperties,
    /// Per column potential in [0, 1]. 0 is silent.
    magnitudes: Vec<f32>,
    /// Per column phase step in [0, phase_steps).
    phase_steps: Vec<u32>,
}

impl AudioSpectrumFrame {
    /// A silent frame.
    pub fn new(properties: &AudioSpectrumProperties) -> Result<Self, FeagiDataError> {
        properties.validate()?;
        let columns = properties.bin_count as usize;
        Ok(AudioSpectrumFrame {
            properties: *properties,
            magnitudes: vec![0.0; columns],
            phase_steps: vec![0; columns],
        })
    }

    pub fn get_properties(&self) -> &AudioSpectrumProperties {
        &self.properties
    }

    pub fn get_magnitudes(&self) -> &[f32] {
        &self.magnitudes
    }

    pub fn get_phase_steps(&self) -> &[u32] {
        &self.phase_steps
    }

    /// Set one column. Potential is clamped to [0, 1].
    pub fn set_column(
        &mut self,
        column: usize,
        magnitude_potential: f32,
        phase_step: u32,
    ) -> Result<(), FeagiDataError> {
        if column >= self.magnitudes.len() {
            return Err(FeagiDataError::BadParameters(format!(
                "Audio column {} is outside {} columns",
                column,
                self.magnitudes.len()
            )));
        }
        if phase_step >= self.properties.phase_steps {
            return Err(FeagiDataError::BadParameters(format!(
                "Audio phase step {} is outside {} steps",
                phase_step, self.properties.phase_steps
            )));
        }
        self.magnitudes[column] = magnitude_potential.clamp(0.0, 1.0);
        self.phase_steps[column] = phase_step;
        Ok(())
    }

    pub fn silence(&mut self) {
        self.magnitudes.fill(0.0);
        self.phase_steps.fill(0);
    }

    /// Columns that will emit a neuron.
    pub fn active_column_count(&self) -> usize {
        self.magnitudes.iter().filter(|m| **m > 0.0).count()
    }
}

impl Display for AudioSpectrumFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AudioSpectrumFrame({} active of {})",
            self.active_column_count(),
            self.magnitudes.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linear() -> AudioSpectrumProperties {
        AudioSpectrumProperties::new_linear(16000, 1024, 512, 16, -80, 0).unwrap()
    }

    #[test]
    fn linear_derives_columns_and_full_band() {
        let p = linear();
        assert_eq!(p.bin_count, 513);
        assert_eq!(p.min_frequency_hz, 0);
        assert_eq!(p.max_frequency_hz, 8000);
        assert!(p.is_waveform_invertible());
    }

    #[test]
    fn rejects_hop_above_half_window() {
        let err = AudioSpectrumProperties::new_linear(48000, 2048, 1600, 16, -80, 0);
        assert!(err.is_err());
    }

    #[test]
    fn rejects_non_power_of_two_window() {
        assert!(AudioSpectrumProperties::new_linear(16000, 1000, 400, 16, -80, 0).is_err());
    }

    #[test]
    fn rejects_inverted_decibel_range() {
        assert!(AudioSpectrumProperties::new_linear(16000, 1024, 512, 16, 0, -80).is_err());
    }

    #[test]
    fn rejects_linear_with_trimmed_band() {
        let mut p = linear();
        p.min_frequency_hz = 20;
        assert!(p.validate().is_err());
    }

    #[test]
    fn logarithmic_validates_band_against_nyquist() {
        assert!(AudioSpectrumProperties::new_logarithmic(
            48000, 4096, 1600, 128, 16, 20, 20000, -80, 0
        )
        .is_ok());
        assert!(AudioSpectrumProperties::new_logarithmic(
            32000, 4096, 1600, 128, 16, 20, 20000, -80, 0
        )
        .is_err());
        assert!(AudioSpectrumProperties::new_logarithmic(
            48000, 4096, 1600, 128, 16, 0, 20000, -80, 0
        )
        .is_err());
    }

    #[test]
    fn frame_rejects_out_of_range_column_and_phase() {
        let mut frame = AudioSpectrumFrame::new(&linear()).unwrap();
        assert!(frame.set_column(513, 0.5, 0).is_err());
        assert!(frame.set_column(0, 0.5, 16).is_err());
        frame.set_column(3, 2.0, 15).unwrap();
        assert_eq!(frame.get_magnitudes()[3], 1.0);
        assert_eq!(frame.active_column_count(), 1);
        frame.silence();
        assert_eq!(frame.active_column_count(), 0);
    }
}
