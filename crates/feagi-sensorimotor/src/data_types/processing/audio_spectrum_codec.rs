//! PCM to audio spectrum frames and back.
//!
//! The analyzer takes a Hann-windowed FFT every `hop_size` samples. The
//! synthesizer inverts each frame and overlap-adds with the same window,
//! normalized by the summed squared window, so on the linear layout the output
//! is the input delayed by `window_size - hop_size` samples. Magnitude is exact
//! up to f32; phase is quantized to `phase_steps`.
//!
//! @cursor:critical-path Runs once per FEAGI tick per audio channel. No heap
//! allocation after construction.

use crate::data_types::{AudioFrequencySpacing, AudioSpectrumFrame, AudioSpectrumProperties};
use feagi_structures::FeagiDataError;
use std::collections::VecDeque;
use std::f64::consts::PI;

/// In-place iterative radix-2 complex FFT.
#[derive(Debug, Clone)]
struct RadixTwoFft {
    size: usize,
    cos_table: Vec<f32>,
    sin_table: Vec<f32>,
    bit_reverse: Vec<usize>,
}

impl RadixTwoFft {
    fn new(size: usize) -> Result<Self, FeagiDataError> {
        if size < 2 || !size.is_power_of_two() {
            return Err(FeagiDataError::BadParameters(format!(
                "FFT size {size} must be a power of two of at least 2"
            )));
        }
        let half = size / 2;
        let mut cos_table = Vec::with_capacity(half);
        let mut sin_table = Vec::with_capacity(half);
        for index in 0..half {
            let angle = 2.0 * PI * index as f64 / size as f64;
            cos_table.push(angle.cos() as f32);
            sin_table.push(angle.sin() as f32);
        }
        let bits = size.trailing_zeros();
        let bit_reverse = (0..size)
            .map(|index| index.reverse_bits() >> (usize::BITS - bits))
            .collect();
        Ok(RadixTwoFft {
            size,
            cos_table,
            sin_table,
            bit_reverse,
        })
    }

    /// X[k] = sum x[n] e^{-2 pi i k n / N}
    fn forward(&self, re: &mut [f32], im: &mut [f32]) {
        let n = self.size;
        for index in 0..n {
            let swapped = self.bit_reverse[index];
            if swapped > index {
                re.swap(index, swapped);
                im.swap(index, swapped);
            }
        }
        let mut length = 2;
        while length <= n {
            let half = length / 2;
            let stride = n / length;
            for start in (0..n).step_by(length) {
                for offset in 0..half {
                    let twiddle = offset * stride;
                    let w_re = self.cos_table[twiddle];
                    let w_im = -self.sin_table[twiddle];
                    let upper = start + offset;
                    let lower = upper + half;
                    let t_re = re[lower] * w_re - im[lower] * w_im;
                    let t_im = re[lower] * w_im + im[lower] * w_re;
                    re[lower] = re[upper] - t_re;
                    im[lower] = im[upper] - t_im;
                    re[upper] += t_re;
                    im[upper] += t_im;
                }
            }
            length *= 2;
        }
    }

    /// x[n] = (1/N) sum X[k] e^{+2 pi i k n / N}
    fn inverse(&self, re: &mut [f32], im: &mut [f32]) {
        for value in im.iter_mut() {
            *value = -*value;
        }
        self.forward(re, im);
        let scale = 1.0 / self.size as f32;
        for (r, i) in re.iter_mut().zip(im.iter_mut()) {
            *r *= scale;
            *i = -*i * scale;
        }
    }
}

/// Periodic Hann window, so overlapping frames tile without a seam.
fn periodic_hann(size: usize) -> Vec<f32> {
    (0..size)
        .map(|n| (0.5 - 0.5 * (2.0 * PI * n as f64 / size as f64).cos()) as f32)
        .collect()
}

/// Column each FFT bin feeds, `None` when the bin is outside the registered band.
fn column_of_each_bin(properties: &AudioSpectrumProperties) -> Vec<Option<usize>> {
    let bins = properties.window_size as usize / 2 + 1;
    match properties.spacing {
        AudioFrequencySpacing::Linear => (0..bins).map(Some).collect(),
        AudioFrequencySpacing::Logarithmic => {
            let min = properties.min_frequency_hz as f64;
            let max = properties.max_frequency_hz as f64;
            let columns = properties.bin_count as usize;
            let bin_width = properties.sample_rate_hz as f64 / properties.window_size as f64;
            let octaves = (max / min).ln();
            (0..bins)
                .map(|bin| {
                    let frequency = bin as f64 * bin_width;
                    if frequency < min || frequency > max {
                        return None;
                    }
                    let position = (frequency / min).ln() / octaves;
                    Some(((position * columns as f64) as usize).min(columns - 1))
                })
                .collect()
        }
    }
}

fn quantize_phase(phase: f32, steps: u32) -> u32 {
    let unit = (phase as f64 + PI) / (2.0 * PI);
    ((unit * steps as f64).floor() as u32) % steps
}

fn dequantize_phase(step: u32, steps: u32) -> f32 {
    ((step as f64 + 0.5) / steps as f64 * 2.0 * PI - PI) as f32
}

fn potential_from_amplitude(amplitude: f32, properties: &AudioSpectrumProperties) -> f32 {
    if amplitude <= 0.0 {
        return 0.0;
    }
    let floor = properties.magnitude_floor_db as f32;
    let ceiling = properties.magnitude_ceiling_db as f32;
    let level_db = 20.0 * amplitude.log10();
    ((level_db - floor) / (ceiling - floor)).clamp(0.0, 1.0)
}

fn amplitude_from_potential(potential: f32, properties: &AudioSpectrumProperties) -> f32 {
    if potential <= 0.0 {
        return 0.0;
    }
    let floor = properties.magnitude_floor_db as f32;
    let ceiling = properties.magnitude_ceiling_db as f32;
    let level_db = floor + potential.clamp(0.0, 1.0) * (ceiling - floor);
    10f32.powf(level_db / 20.0)
}

/// Streaming PCM to spectrum frames.
#[derive(Debug, Clone)]
pub struct AudioSpectrumAnalyzer {
    properties: AudioSpectrumProperties,
    fft: RadixTwoFft,
    window: Vec<f32>,
    /// Most recent `window_size` samples, oldest first. Starts as silence.
    history: VecDeque<f32>,
    samples_since_frame: usize,
    /// Converts |X[k]| to the amplitude of a sinusoid at bin k.
    amplitude_scale: f32,
    column_of_bin: Vec<Option<usize>>,
    column_energy: Vec<f32>,
    column_peak: Vec<(f32, f32)>,
    scratch_re: Vec<f32>,
    scratch_im: Vec<f32>,
}

impl AudioSpectrumAnalyzer {
    pub fn new(properties: AudioSpectrumProperties) -> Result<Self, FeagiDataError> {
        properties.validate()?;
        let size = properties.window_size as usize;
        let window = periodic_hann(size);
        let window_sum: f32 = window.iter().sum();
        let columns = properties.bin_count as usize;
        Ok(AudioSpectrumAnalyzer {
            properties,
            fft: RadixTwoFft::new(size)?,
            amplitude_scale: 2.0 / window_sum,
            window,
            history: std::iter::repeat_n(0.0, size).collect(),
            samples_since_frame: 0,
            column_of_bin: column_of_each_bin(&properties),
            column_energy: vec![0.0; columns],
            column_peak: vec![(0.0, 0.0); columns],
            scratch_re: vec![0.0; size],
            scratch_im: vec![0.0; size],
        })
    }

    pub fn get_properties(&self) -> &AudioSpectrumProperties {
        &self.properties
    }

    /// Feed mono PCM in [-1, 1]. Returns one frame per completed hop.
    pub fn push_samples(
        &mut self,
        samples: &[f32],
    ) -> Result<Vec<AudioSpectrumFrame>, FeagiDataError> {
        if let Some(bad) = samples.iter().position(|s| !s.is_finite()) {
            return Err(FeagiDataError::BadParameters(format!(
                "Audio sample {bad} is not a finite number"
            )));
        }
        let hop = self.properties.hop_size as usize;
        let mut frames = Vec::with_capacity((self.samples_since_frame + samples.len()) / hop);
        for &sample in samples {
            self.history.pop_front();
            self.history.push_back(sample);
            self.samples_since_frame += 1;
            if self.samples_since_frame == hop {
                self.samples_since_frame = 0;
                frames.push(self.analyze_current_window()?);
            }
        }
        Ok(frames)
    }

    fn analyze_current_window(&mut self) -> Result<AudioSpectrumFrame, FeagiDataError> {
        for (index, (sample, weight)) in self.history.iter().zip(&self.window).enumerate() {
            self.scratch_re[index] = sample * weight;
            self.scratch_im[index] = 0.0;
        }
        self.fft.forward(&mut self.scratch_re, &mut self.scratch_im);

        self.column_energy.fill(0.0);
        self.column_peak.fill((0.0, 0.0));
        for (bin, column) in self.column_of_bin.iter().enumerate() {
            let Some(column) = *column else {
                continue;
            };
            let re = self.scratch_re[bin];
            let im = self.scratch_im[bin];
            let amplitude = (re * re + im * im).sqrt() * self.amplitude_scale;
            self.column_energy[column] += amplitude * amplitude;
            if amplitude > self.column_peak[column].0 {
                self.column_peak[column] = (amplitude, im.atan2(re));
            }
        }

        let mut frame = AudioSpectrumFrame::new(&self.properties)?;
        let steps = self.properties.phase_steps;
        for column in 0..self.column_energy.len() {
            let potential =
                potential_from_amplitude(self.column_energy[column].sqrt(), &self.properties);
            if potential <= 0.0 {
                continue;
            }
            let phase_step = quantize_phase(self.column_peak[column].1, steps);
            frame.set_column(column, potential, phase_step)?;
        }
        Ok(frame)
    }
}

/// Streaming spectrum frames to PCM.
#[derive(Debug, Clone)]
pub struct AudioSpectrumSynthesizer {
    properties: AudioSpectrumProperties,
    fft: RadixTwoFft,
    window: Vec<f32>,
    accumulator: Vec<f32>,
    window_power: Vec<f32>,
    /// Converts a sinusoid amplitude back to |X[k]|.
    magnitude_scale: f32,
    bins_of_column: Vec<Vec<usize>>,
    scratch_re: Vec<f32>,
    scratch_im: Vec<f32>,
}

impl AudioSpectrumSynthesizer {
    pub fn new(properties: AudioSpectrumProperties) -> Result<Self, FeagiDataError> {
        properties.validate()?;
        let size = properties.window_size as usize;
        let window = periodic_hann(size);
        let window_sum: f32 = window.iter().sum();
        let mut bins_of_column = vec![Vec::new(); properties.bin_count as usize];
        for (bin, column) in column_of_each_bin(&properties).into_iter().enumerate() {
            if let Some(column) = column {
                bins_of_column[column].push(bin);
            }
        }
        Ok(AudioSpectrumSynthesizer {
            properties,
            fft: RadixTwoFft::new(size)?,
            magnitude_scale: window_sum / 2.0,
            window,
            accumulator: vec![0.0; size],
            window_power: vec![0.0; size],
            bins_of_column,
            scratch_re: vec![0.0; size],
            scratch_im: vec![0.0; size],
        })
    }

    pub fn get_properties(&self) -> &AudioSpectrumProperties {
        &self.properties
    }

    /// Samples between an input sample and the same sample leaving the synthesizer.
    pub fn latency_samples(&self) -> usize {
        (self.properties.window_size - self.properties.hop_size) as usize
    }

    /// Consume one frame and return exactly `hop_size` output samples.
    pub fn push_frame(&mut self, frame: &AudioSpectrumFrame) -> Result<Vec<f32>, FeagiDataError> {
        if frame.get_properties() != &self.properties {
            return Err(FeagiDataError::BadParameters(format!(
                "Audio frame {} does not match synthesizer {}",
                frame.get_properties(),
                self.properties
            )));
        }
        let size = self.properties.window_size as usize;
        let half = size / 2;
        let steps = self.properties.phase_steps;
        self.scratch_re.fill(0.0);
        self.scratch_im.fill(0.0);

        for (column, bins) in self.bins_of_column.iter().enumerate() {
            let potential = frame.get_magnitudes()[column];
            if potential <= 0.0 || bins.is_empty() {
                continue;
            }
            // A shared column splits its energy evenly over the bins it covers.
            let amplitude =
                amplitude_from_potential(potential, &self.properties) / (bins.len() as f32).sqrt();
            let magnitude = amplitude * self.magnitude_scale;
            let phase = dequantize_phase(frame.get_phase_steps()[column], steps);
            for &bin in bins {
                if bin == 0 || bin == half {
                    self.scratch_re[bin] = magnitude * phase.cos();
                } else {
                    self.scratch_re[bin] = magnitude * phase.cos();
                    self.scratch_im[bin] = magnitude * phase.sin();
                    self.scratch_re[size - bin] = self.scratch_re[bin];
                    self.scratch_im[size - bin] = -self.scratch_im[bin];
                }
            }
        }
        self.fft.inverse(&mut self.scratch_re, &mut self.scratch_im);

        for index in 0..size {
            let weight = self.window[index];
            self.accumulator[index] += self.scratch_re[index] * weight;
            self.window_power[index] += weight * weight;
        }

        let hop = self.properties.hop_size as usize;
        // Positions no window has covered yet precede the first real input sample.
        let output = (0..hop)
            .map(|index| {
                let power = self.window_power[index];
                if power > f32::EPSILON {
                    self.accumulator[index] / power
                } else {
                    0.0
                }
            })
            .collect();

        self.accumulator.copy_within(hop.., 0);
        self.window_power.copy_within(hop.., 0);
        self.accumulator[size - hop..].fill(0.0);
        self.window_power[size - hop..].fill(0.0);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frequency: f32, amplitude: f32, sample_rate: u32, count: usize) -> Vec<f32> {
        (0..count)
            .map(|n| {
                amplitude
                    * (2.0 * std::f32::consts::PI * frequency * n as f32 / sample_rate as f32).sin()
            })
            .collect()
    }

    fn round_trip(properties: AudioSpectrumProperties, input: &[f32]) -> Vec<f32> {
        let mut analyzer = AudioSpectrumAnalyzer::new(properties).unwrap();
        let mut synthesizer = AudioSpectrumSynthesizer::new(properties).unwrap();
        let mut output = Vec::new();
        for frame in analyzer.push_samples(input).unwrap() {
            output.extend(synthesizer.push_frame(&frame).unwrap());
        }
        output
    }

    /// Signal-to-error ratio in dB between the delayed input and the output.
    fn reconstruction_snr_db(input: &[f32], output: &[f32], latency: usize) -> f32 {
        let mut signal = 0.0f64;
        let mut error = 0.0f64;
        for (index, out) in output.iter().enumerate().skip(latency) {
            let reference = input[index - latency] as f64;
            signal += reference * reference;
            error += (reference - *out as f64).powi(2);
        }
        (10.0 * (signal / error).log10()) as f32
    }

    #[test]
    fn fft_matches_direct_dft() {
        let fft = RadixTwoFft::new(16).unwrap();
        let input: Vec<f32> = (0..16).map(|n| ((n * 7) % 5) as f32 - 2.0).collect();
        let mut re = input.clone();
        let mut im = vec![0.0; 16];
        fft.forward(&mut re, &mut im);
        for k in 0..16 {
            let (mut exp_re, mut exp_im) = (0.0f64, 0.0f64);
            for (n, x) in input.iter().enumerate() {
                let angle = -2.0 * PI * (k * n) as f64 / 16.0;
                exp_re += *x as f64 * angle.cos();
                exp_im += *x as f64 * angle.sin();
            }
            assert!((re[k] as f64 - exp_re).abs() < 1e-3, "re[{k}]");
            assert!((im[k] as f64 - exp_im).abs() < 1e-3, "im[{k}]");
        }
        fft.inverse(&mut re, &mut im);
        for (n, x) in input.iter().enumerate() {
            assert!((re[n] - x).abs() < 1e-4);
        }
    }

    #[test]
    fn phase_quantization_round_trips_to_step_center() {
        for steps in [1u32, 4, 16, 256] {
            for step in 0..steps {
                assert_eq!(quantize_phase(dequantize_phase(step, steps), steps), step);
            }
        }
        assert_eq!(quantize_phase(std::f32::consts::PI, 8), 0);
    }

    #[test]
    fn decibel_mapping_round_trips_inside_range() {
        let p = AudioSpectrumProperties::new_linear(16000, 1024, 512, 16, -80, 0).unwrap();
        for amplitude in [1e-3f32, 0.01, 0.25, 1.0] {
            let potential = potential_from_amplitude(amplitude, &p);
            assert!((amplitude_from_potential(potential, &p) - amplitude).abs() < amplitude * 1e-3);
        }
        assert_eq!(potential_from_amplitude(1e-5, &p), 0.0);
        assert_eq!(potential_from_amplitude(0.0, &p), 0.0);
    }

    #[test]
    fn sine_lights_its_bin_with_its_amplitude() {
        let p = AudioSpectrumProperties::new_linear(16000, 1024, 512, 16, -80, 0).unwrap();
        let bin = 64usize;
        let frequency = bin as f32 * p.fft_bin_width_hz();
        let mut analyzer = AudioSpectrumAnalyzer::new(p).unwrap();
        let frames = analyzer
            .push_samples(&sine(frequency, 0.5, 16000, 4096))
            .unwrap();
        let last = frames.last().unwrap();
        let peak = last
            .get_magnitudes()
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0;
        assert_eq!(peak, bin);
        let decoded = amplitude_from_potential(last.get_magnitudes()[bin], &p);
        assert!((decoded - 0.5).abs() < 0.01, "decoded amplitude {decoded}");
    }

    #[test]
    fn linear_layout_rebuilds_the_input_waveform() {
        let p = AudioSpectrumProperties::new_linear(16000, 1024, 256, 256, -100, 0).unwrap();
        let input: Vec<f32> = sine(440.0, 0.4, 16000, 16000)
            .iter()
            .zip(sine(1234.5, 0.2, 16000, 16000))
            .map(|(a, b)| a + b)
            .collect();
        let output = round_trip(p, &input);
        let latency = (p.window_size - p.hop_size) as usize;
        let snr = reconstruction_snr_db(&input, &output, latency);
        assert!(
            snr > 25.0,
            "256 phase steps should rebuild the wave, snr {snr} dB"
        );
    }

    /// Music-like test signal: a chord with harmonics and a slow loudness swell.
    fn chord_with_swell(sample_rate: u32, count: usize) -> Vec<f32> {
        let partials = [
            (220.0f32, 0.20f32),
            (440.0, 0.12),
            (660.0, 0.08),
            (277.2, 0.15),
            (554.4, 0.07),
            (329.6, 0.12),
            (1318.5, 0.03),
            (2637.0, 0.015),
        ];
        (0..count)
            .map(|n| {
                let t = n as f32 / sample_rate as f32;
                let swell = 0.6 + 0.4 * (2.0 * std::f32::consts::PI * 1.5 * t).sin();
                swell
                    * partials
                        .iter()
                        .map(|(f, a)| a * (2.0 * std::f32::consts::PI * f * t).sin())
                        .sum::<f32>()
            })
            .collect()
    }

    #[test]
    fn streaming_settings_rebuild_music_within_tolerance() {
        // Sensory Generator stream: 16 kHz, window 512, hop 160 (100 Hz bursts), -80..0 dB.
        for (steps, min_snr_db) in [(64u32, 30.0f32), (256, 30.0)] {
            let p = AudioSpectrumProperties::new_linear(16000, 512, 160, steps, -80, 0).unwrap();
            let input = chord_with_swell(16000, 32000);
            let output = round_trip(p, &input);
            let latency = (p.window_size - p.hop_size) as usize;
            let snr = reconstruction_snr_db(&input, &output, latency);
            assert!(
                snr > min_snr_db,
                "{steps} phase steps rebuild music at {snr:.1} dB, expected above {min_snr_db} dB"
            );
        }
    }

    #[test]
    fn a_decoder_floor_other_than_the_encoder_floor_distorts_level() {
        // Same stream decoded with a -40 dB floor instead of the -80 dB it was encoded with.
        let encode = AudioSpectrumProperties::new_linear(16000, 512, 160, 64, -80, 0).unwrap();
        let decode = AudioSpectrumProperties::new_linear(16000, 512, 160, 64, -40, 0).unwrap();
        let input = chord_with_swell(16000, 32000);
        let mut analyzer = AudioSpectrumAnalyzer::new(encode).unwrap();
        let mut synthesizer = AudioSpectrumSynthesizer::new(decode).unwrap();
        let mut output = Vec::new();
        for frame in analyzer.push_samples(&input).unwrap() {
            let mut relabeled = AudioSpectrumFrame::new(&decode).unwrap();
            for (column, (magnitude, phase)) in frame
                .get_magnitudes()
                .iter()
                .zip(frame.get_phase_steps())
                .enumerate()
            {
                if *magnitude > 0.0 {
                    relabeled.set_column(column, *magnitude, *phase).unwrap();
                }
            }
            output.extend(synthesizer.push_frame(&relabeled).unwrap());
        }
        let latency = (encode.window_size - encode.hop_size) as usize;
        let snr = reconstruction_snr_db(&input, &output, latency);
        assert!(
            snr < 6.0,
            "mismatched floor should wreck the level, got {snr:.1} dB"
        );
    }

    #[test]
    fn coarse_phase_still_tracks_the_waveform() {
        let p = AudioSpectrumProperties::new_linear(16000, 1024, 512, 16, -100, 0).unwrap();
        let input = sine(440.0, 0.5, 16000, 16000);
        let output = round_trip(p, &input);
        let snr = reconstruction_snr_db(&input, &output, (p.window_size - p.hop_size) as usize);
        assert!(snr > 8.0, "16 phase steps snr {snr} dB");
    }

    #[test]
    fn silence_stays_silent_and_output_is_one_hop_per_frame() {
        let p = AudioSpectrumProperties::new_linear(16000, 512, 128, 8, -80, 0).unwrap();
        let output = round_trip(p, &vec![0.0; 2048]);
        assert_eq!(output.len(), 2048);
        assert!(output.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn logarithmic_layout_keeps_the_tone_in_its_column() {
        let p = AudioSpectrumProperties::new_logarithmic(16000, 2048, 512, 32, 8, 50, 8000, -80, 0)
            .unwrap();
        let mut analyzer = AudioSpectrumAnalyzer::new(p).unwrap();
        let frame = analyzer
            .push_samples(&sine(1000.0, 0.5, 16000, 8192))
            .unwrap()
            .pop()
            .unwrap();
        let expected = ((1000.0f64 / 50.0).ln() / (8000.0f64 / 50.0).ln() * 32.0) as usize;
        let peak = frame
            .get_magnitudes()
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0;
        assert!(
            peak.abs_diff(expected) <= 1,
            "peak {peak}, expected {expected}"
        );

        let mut synthesizer = AudioSpectrumSynthesizer::new(p).unwrap();
        let samples = synthesizer.push_frame(&frame).unwrap();
        assert_eq!(samples.len(), 512);
        assert!(samples.iter().any(|s| s.abs() > 1e-3));
    }

    #[test]
    fn rejects_non_finite_samples_and_mismatched_frames() {
        let p = AudioSpectrumProperties::new_linear(16000, 512, 128, 8, -80, 0).unwrap();
        let mut analyzer = AudioSpectrumAnalyzer::new(p).unwrap();
        assert!(analyzer.push_samples(&[0.0, f32::NAN]).is_err());

        let other = AudioSpectrumProperties::new_linear(16000, 1024, 128, 8, -80, 0).unwrap();
        let mut synthesizer = AudioSpectrumSynthesizer::new(p).unwrap();
        let frame = AudioSpectrumFrame::new(&other).unwrap();
        assert!(synthesizer.push_frame(&frame).is_err());
    }
}
