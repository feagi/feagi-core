# FEAGI Data Serialization

This crate contains traits to allow various structures to be serialized to and deserialized from byte vectors via the use of a common [FeagiSerializable](../src/feagi_serializable.rs) trait. Byte data itself is managed, and owned by the [FeagiByteContainer](../src/feagi_byte_container.rs) struct (often shortened to FBC).

More information about the FeagiByteContainer can be found [here](feagi_byte_container.md).

For detailed specifications of the binary format, see [byte_structure_container.md](byte_structure_container.md).