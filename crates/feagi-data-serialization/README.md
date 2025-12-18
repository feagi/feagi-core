# feagi-data-serialization

**Efficient serialization and deserialization for FEAGI data structures**

[![Crates.io](https://img.shields.io/crates/v/feagi-data-serialization.svg)](https://crates.io/crates/feagi-data-serialization)
[![Documentation](https://docs.rs/feagi-data-serialization/badge.svg)](https://docs.rs/feagi-data-serialization)
[![License](https://img.shields.io/crates/l/feagi-data-serialization.svg)](LICENSE)

## Overview

`feagi-data-serialization` provides high-performance serialization and deserialization for FEAGI data structures. This crate implements the FEAGI Byte Container (FBC) format for efficient binary encoding of neural data.

## Features

- **FEAGI Byte Container (FBC)**: Efficient binary container format
- **Multiple Structure Types**: Support for JSON, raw images, and XYZP neuron data
- **Cortical Mapped Data**: Serialization for cortical-area-mapped neuron voxels
- **Zero-Copy Operations**: Where possible, for maximum performance
- **Type-Safe Deserialization**: Strongly typed structure identification

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
feagi-data-serialization = "2.0.0"
feagi-data-structures = "2.0.0"
```

## Usage

### FEAGI Byte Container

```rust
use feagi-data-serialization::{FeagiByteContainer, FeagiByteStructureType};

// Create a container with JSON data
let json_data = br#"{"cortical_id": "ipu000", "data": [1,2,3]}"#;
let container = FeagiByteContainer::new(
    FeagiByteStructureType::Json,
    json_data.to_vec()
);

// Serialize to bytes
let bytes = container.to_bytes();

// Deserialize from bytes
let decoded = FeagiByteContainer::from_bytes(&bytes)?;
```

### Cortical Mapped XYZP Data

```rust
use feagi-data-serialization::implementations::cortical_mapped_xyzp_neuron_data::CorticalMappedXYZPNeuronVoxels;
use feagi-data-structures::genomic::cortical_area::CorticalID;

// Create neuron data mapped to cortical areas
let cortical_id = CorticalID::from_base_64("aXB1MDAw")?;
let mut data = CorticalMappedXYZPNeuronVoxels::new();

// Add voxels for a cortical area
data.insert(cortical_id, vec![(10, 20, 5, 128)]);

// Serialize efficiently
let bytes = data.serialize_to_bytes();
```

## Supported Structure Types

- **JSON** (Type 0x01): Standard JSON encoding
- **Single Raw Image** (Type 0x08): Raw image data
- **XYZP Neuron Data** (Type 0x0B): Neuron voxel potential data

See [documentation](docs/byte_structure_container.md) for the complete format specification.

## Performance

The FBC format is designed for high-throughput neural data transmission:

- Minimal overhead (8-byte header)
- Type identification for fast routing
- Optional compression support
- Efficient for large batches of neuron data

## Documentation

For detailed API documentation, visit [docs.rs/feagi-data-serialization](https://docs.rs/feagi-data-serialization).

For format specifications, see the [docs/](docs/) directory.

## Part of FEAGI Ecosystem

This crate is part of the FEAGI project:

- **Main Project**: [feagi](https://crates.io/crates/feagi)
- **Data Structures**: [feagi-data-structures](https://crates.io/crates/feagi-data-structures)
- **Connector Core**: [feagi-connector-core](https://crates.io/crates/feagi-connector-core)

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/feagi/feagi-core) for contribution guidelines.

## Links

- **Homepage**: https://feagi.org
- **Repository**: https://github.com/feagi/feagi-core
- **Documentation**: https://docs.rs/feagi-data-serialization
- **Issue Tracker**: https://github.com/feagi/feagi-core/issues

