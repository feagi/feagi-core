# feagi-sensorimotor

**Peripheral Nervous System (PNS) - Data processing, caching, and neuron voxel encoding for FEAGI agents**

[![Crates.io](https://img.shields.io/crates/v/feagi-sensorimotor.svg)](https://crates.io/crates/feagi-sensorimotor)
[![Documentation](https://docs.rs/feagi-sensorimotor/badge.svg)](https://docs.rs/feagi-sensorimotor)
[![License](https://img.shields.io/crates/l/feagi-sensorimotor.svg)](LICENSE)

## Overview

`feagi-sensorimotor` provides the foundational components for building FEAGI connector agents. This crate includes data processing pipelines, caching mechanisms, and neuron voxel encoding/decoding for various data types. It serves as the "Peripheral Nervous System" layer, handling sensory input processing and motor output encoding.

## Features

- **Data Processing Pipelines**: Composable stages for transforming sensory data
- **Caching Systems**: Efficient per-channel stream caches for sensory and motor data
- **XYZP Encoding/Decoding**: Convert between data types and neuron voxel representations
- **Image Processing**: Segmentation, transformation, and quick-diff algorithms
- **Multiple Encoding Schemes**: Linear and exponential encoding for 1D-4D data
- **Type-Safe Pipeline**: Strongly-typed pipeline stages with validation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
feagi-sensorimotor = "2.0.0"
feagi-structures = "0.0.1-beta.1"
feagi-serialization = "0.0.1-beta.1"
```

## Usage

### Data Processing Pipeline

```rust
use feagi_sensorimotor::data_pipeline::PipelineStage;

// Create a pipeline for image processing
let pipeline = vec![
    PipelineStage::ImageSegmentor { segments: 4 },
    PipelineStage::QuickDiff { threshold: 10 },
];

// Process data through the pipeline
for stage in &pipeline {
    data = stage.process(data)?;
}
```

### XYZP Encoding

```rust
use feagi_sensorimotor::neuron_voxel_coding::xyzp::encoders::Percentage1DLinear;
use feagi_sensorimotor::data_types::percentages::Percentage;

// Encode a percentage value as neuron voxels
let encoder = Percentage1DLinear::new(100);
let percentage = Percentage::new(75.0)?;
let voxels = encoder.encode(&percentage);
```

### Sensory Device Cache

```rust
use feagi_sensorimotor::caching::SensoryChannelStreamCaches;

// Create a cache for sensory data streams
let mut cache = SensoryChannelStreamCaches::new();

// Cache data for a sensor
cache.update("camera_01", sensor_data);

// Retrieve cached data
let data = cache.get("camera_01")?;
```

### Image Segmentation

```rust
use feagi_sensorimotor::data_types::SegmentedImageFrame;

// Segment an image into regions
let segmented = SegmentedImageFrame::from_image_frame(
    &image_frame,
    4,  // number of segments
)?;

// Access individual segments
for segment in segmented.segments() {
    process_segment(segment);
}
```

## Supported Encodings

### Linear Encodings
- **1D**: Single value → voxel line
- **2D**: (x, y) coordinates → voxel plane
- **3D**: (x, y, z) coordinates → voxel cube
- **4D**: (x, y, z, intensity) → voxel hypercube

### Exponential Encodings
- Higher resolution near zero
- Suitable for non-linear sensory data
- Available for 1D through 4D

### Specialized Encodings
- **Boolean**: On/off states
- **Cartesian Plane**: 2D position tracking
- **Gaze Properties**: Eye tracking data
- **Misc Data**: Generic key-value pairs

## Audio Spectrum

`AudioInput` (IPU) and `AudioOutput` (OPU) carry one tick of audio as a spectrum. The unit
reference is `aud`, and the areas use the `Misc` configuration flag.

- X is the frequency column, Y is the quantized phase step, Z is 0.
- Membrane potential is the column magnitude, mapped linearly from
  `magnitude_floor_db` (0) to `magnitude_ceiling_db` (1). Quieter columns emit no neuron.
- Stereo is a second unit index with the same properties.
- On decode, every tick starts silent. When several phase rows fire in one column, the
  row with the highest potential wins.

`AudioSpectrumProperties` travels in the device registration, so FEAGI sizes the area as
`bin_count x phase_steps x 1`:

| Field | Meaning |
|---|---|
| `sample_rate_hz` | PCM rate the analyzer expects |
| `window_size` | FFT length, power of two, 4 to 8192 |
| `hop_size` | Samples per tick, at most half the window |
| `spacing` | `Linear` or `Logarithmic` |
| `min_frequency_hz`, `max_frequency_hz` | Band. Linear is fixed at 0 Hz to Nyquist |
| `bin_count` | Columns. Linear is `window_size / 2 + 1` |
| `phase_steps` | Y height, 1 to 256. 1 carries magnitude only |

`data_types::processing::AudioSpectrumAnalyzer` turns PCM into one frame per hop.
`AudioSpectrumSynthesizer` turns frames back into PCM by overlap-add. On the linear layout the
output is the input delayed by `window_size - hop_size` samples, with error set by the phase
resolution. On the logarithmic layout several FFT bins share a column, so playback is a
resynthesis of the spectrum, not the input waveform.

```rust
use feagi_sensorimotor::data_types::processing::{AudioSpectrumAnalyzer, AudioSpectrumSynthesizer};
use feagi_sensorimotor::data_types::AudioSpectrumProperties;

// 16 kHz, 1024-sample window, 30 Hz ticks, 16 phase steps, -80..0 dB.
let properties = AudioSpectrumProperties::new_linear(16000, 1024, 16000 / 30, 16, -80, 0)?;
let mut analyzer = AudioSpectrumAnalyzer::new(properties)?;
for frame in analyzer.push_samples(&pcm)? {
    sensor_cache.audio_input_write(unit, channel, frame.into())?;
}
```

## Pipeline Stages

Available pipeline stages:

- **Identity**: Pass-through (no transformation)
- **ImageSegmentor**: Divide image into grid segments
- **QuickDiff**: Motion detection via frame differencing
- **ImageTransformer**: Scale, rotate, crop operations (disabled)
- **Ranges**: Value range mapping (disabled)
- **RollingWindows**: Temporal aggregation (disabled)

## Documentation

For detailed API documentation, visit [docs.rs/feagi-sensorimotor](https://docs.rs/feagi-sensorimotor).

## Examples

See the [examples/](examples/) directory for complete examples:

- `segmented_video_stream.rs`: Video processing with segmentation

## Part of FEAGI Ecosystem

This crate is part of the FEAGI project:

- **Main Project**: [feagi](https://crates.io/crates/feagi)
- **Data Structures**: [feagi-data-structures](https://crates.io/crates/feagi-data-structures)
- **Data Serialization**: [feagi-serialization](https://crates.io/crates/feagi-serialization)
- **Agent SDK**: [feagi-agent](https://crates.io/crates/feagi-agent)

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/feagi/feagi-core) for contribution guidelines.

## Links

- **Homepage**: https://feagi.org
- **Repository**: https://github.com/feagi/feagi-core
- **Documentation**: https://docs.rs/feagi-sensorimotor
- **Issue Tracker**: https://github.com/feagi/feagi-core/issues

