# feagi_data_structures

**Core data structures and types for the FEAGI ecosystem**

[![Crates.io](https://img.shields.io/crates/v/feagi_data_structures.svg)](https://crates.io/crates/feagi_data_structures)
[![Documentation](https://docs.rs/feagi_data_structures/badge.svg)](https://docs.rs/feagi_data_structures)
[![License](https://img.shields.io/crates/l/feagi_data_structures.svg)](LICENSE)

## Overview

`feagi_data_structures` provides the foundational data types used throughout the FEAGI (Framework for Evolutionary Artificial General Intelligence) ecosystem. This crate defines core structures for genomic data, brain regions, cortical areas, and neuron voxels.

## Features

- **Genomic Data Structures**: Types for representing brain genome definitions
- **Brain Regions**: Hierarchical brain region organization with RegionID and RegionType
- **Cortical Areas**: Cortical area definitions with CorticalID and CorticalType
- **Neuron Voxels**: XYZP (X, Y, Z, Potential) neuron voxel representations
- **Templates**: Predefined templates for sensor and motor cortical units
- **Error Handling**: Comprehensive error types for data validation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
feagi_data_structures = "2.0.0"
```

## Usage

### Cortical Areas

```rust
use feagi_data_structures::genomic::cortical_area::{CorticalID, CorticalType};

// Create a cortical ID
let cortical_id = CorticalID::from_base_64("aXB1MDAw")?;

// Get the cortical type
let cortical_type = cortical_id.cortical_type();
```

### Brain Regions

```rust
use feagi_data_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};

// Create a brain region
let region = BrainRegion::new(
    RegionID::new("vision"),
    "Visual Processing".to_string(),
    RegionType::Sensory
)?;
```

### Neuron Voxels

```rust
use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZP;

// Create neuron voxels
let voxel = NeuronVoxelXYZP::new(10, 20, 5, 128);
```

## Documentation

For detailed API documentation, visit [docs.rs/feagi_data_structures](https://docs.rs/feagi_data_structures).

For conceptual documentation and guides, see the [docs/](docs/) directory.

## Part of FEAGI Ecosystem

This crate is part of the FEAGI project:

- **Main Project**: [feagi](https://crates.io/crates/feagi)
- **Data Serialization**: [feagi_data_serialization](https://crates.io/crates/feagi_data_serialization)
- **Connector Core**: [feagi_connector_core](https://crates.io/crates/feagi_connector_core)

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/feagi/feagi-core) for contribution guidelines.

## Links

- **Homepage**: https://feagi.org
- **Repository**: https://github.com/feagi/feagi-core
- **Documentation**: https://docs.rs/feagi_data_structures
- **Issue Tracker**: https://github.com/feagi/feagi-core/issues

