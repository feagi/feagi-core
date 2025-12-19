# feagi-pns

**Peripheral Nervous System (PNS) - Data processing, caching, and neuron voxel encoding for FEAGI agents**

[![Crates.io](https://img.shields.io/crates/v/feagi-pns.svg)](https://crates.io/crates/feagi-pns)
[![Documentation](https://docs.rs/feagi-pns/badge.svg)](https://docs.rs/feagi-pns)
[![License](https://img.shields.io/crates/l/feagi-pns.svg)](LICENSE)

## Overview

`feagi-pns` provides the foundational components for building FEAGI connector agents. This crate includes data processing pipelines, caching mechanisms, and neuron voxel encoding/decoding for various data types. It serves as the "Peripheral Nervous System" layer, handling sensory input processing and motor output encoding.

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
feagi-pns = "2.0.0"
feagi-data-structures = "2.0.0"
feagi-data-serialization = "2.0.0"
```

## Usage

### Data Processing Pipeline

```rust
use feagi_pns::data_pipeline::PipelineStage;

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
use feagi_pns::neuron_voxel_coding::xyzp::encoders::Percentage1DLinear;
use feagi_pns::data_types::percentages::Percentage;

// Encode a percentage value as neuron voxels
let encoder = Percentage1DLinear::new(100);
let percentage = Percentage::new(75.0)?;
let voxels = encoder.encode(&percentage);
```

### Sensory Device Cache

```rust
use feagi_pns::caching::SensoryChannelStreamCaches;

// Create a cache for sensory data streams
let mut cache = SensoryChannelStreamCaches::new();

// Cache data for a sensor
cache.update("camera_01", sensor_data);

// Retrieve cached data
let data = cache.get("camera_01")?;
```

### Image Segmentation

```rust
use feagi_pns::data_types::SegmentedImageFrame;

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

## Pipeline Stages

Available pipeline stages:

- **Identity**: Pass-through (no transformation)
- **ImageSegmentor**: Divide image into grid segments
- **QuickDiff**: Motion detection via frame differencing
- **ImageTransformer**: Scale, rotate, crop operations (disabled)
- **Ranges**: Value range mapping (disabled)
- **RollingWindows**: Temporal aggregation (disabled)

## Documentation

For detailed API documentation, visit [docs.rs/feagi-pns](https://docs.rs/feagi-pns).

## Examples

See the [examples/](examples/) directory for complete examples:

- `segmented_video_stream.rs`: Video processing with segmentation

## Part of FEAGI Ecosystem

This crate is part of the FEAGI project:

- **Main Project**: [feagi](https://crates.io/crates/feagi)
- **Data Structures**: [feagi-data-structures](https://crates.io/crates/feagi-data-structures)
- **Data Serialization**: [feagi-data-serialization](https://crates.io/crates/feagi-data-serialization)
- **Agent SDK**: [feagi-agent](https://crates.io/crates/feagi-agent)

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/feagi/feagi-core) for contribution guidelines.

## Links

- **Homepage**: https://feagi.org
- **Repository**: https://github.com/feagi/feagi-core
- **Documentation**: https://docs.rs/feagi-pns
- **Issue Tracker**: https://github.com/feagi/feagi-core/issues

