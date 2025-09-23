# FEAGI Data Serialization

This crate contains traits to allow various structures to be serialized to and deserialized from byte vectors via the use of a common FEAGISerializable trait. Byte data itself is managed, and owned by the FeagiByteStructure struct (oftene shortened to FBS).

More information about the specification of the byte structures can be found [here](byte_structure_global.md).