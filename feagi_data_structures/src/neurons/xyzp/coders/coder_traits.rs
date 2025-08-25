//! Core traits for neural encoding and decoding operations.
//!
//! This module defines the fundamental traits that all neural encoders and decoders
//! must implement. These traits provide a unified interface for converting between
//! external I/O data types and internal neural representations using the XYZP
//! coordinate system.

use std::collections::HashMap;
use crate::FeagiDataError;
use crate::genomic::descriptors::CorticalChannelIndex;
use crate::neurons::xyzp::CorticalMappedXYZPNeuronData;
use crate::wrapped_io_data::{WrappedIOType, WrappedIOData};

/// Trait for encoding external data into neural XYZP representations.
///
/// `NeuronXYZPEncoder` defines the interface for converting I/O data types into
/// neural activation patterns within cortical areas. Encoders transform external
/// sensor data into spatial patterns of neural activation that can be processed
/// by the FEAGI neural network.
///
/// # Encoding Process
///
/// 1. **Data Reception**: Accept typed I/O data from external sources
/// 2. **Spatial Mapping**: Convert data values to neural coordinates (X, Y, Z)
/// 3. **Activation Encoding**: Generate appropriate potential (P) values
/// 4. **Neural Writing**: Store activations in cortical mapped neuron data
///
/// # Implementation Requirements
///
/// Implementors must provide:
/// - **Type Declaration**: Specify which I/O data type this encoder handles
/// - **Single Channel Encoding**: Convert data for one cortical channel
/// - **Multi Channel Support**: Efficiently handle multiple channels (optional override)
///
/// # Thread Safety
/// All encoder implementations must be `Sync + Send` for use in multi-threaded
/// neural simulation environments.
pub trait NeuronXYZPEncoder {
    /// Returns the I/O data type this encoder can process.
    ///
    /// This method specifies which [`IOTypeVariant`] this encoder is designed
    /// to handle. The type system uses this for validation and automatic
    /// encoder selection based on data type.
    ///
    /// # Returns
    /// The [`IOTypeVariant`] that this encoder can process
    fn get_encodable_data_type(&self) -> WrappedIOType;

    /// Encodes I/O data into neural activations for a single cortical channel.
    ///
    /// This is the core encoding method that converts external data into neural
    /// activation patterns within a specific cortical channel. The implementation
    /// should handle the data type specified by `get_encodable_data_type()`.
    ///
    /// # Arguments
    /// * `wrapped_value` - The I/O data to encode (must match encoder's data type)
    /// * `cortical_channel` - Target channel index within the cortical area
    /// * `write_target` - Cortical neuron data collection to write activations to
    ///
    /// # Returns
    /// * `Ok(())` - Encoding completed successfully
    /// * `Err(FeagiDataProcessingError)` - Encoding failed due to data type mismatch or other error
    ///
    /// # Implementation Notes
    /// - Validate that the input data matches the expected type
    /// - Convert data values to appropriate neural coordinates (X, Y, Z)
    /// - Generate suitable potential (P) values for neural activation
    /// - Write results to the target cortical area using the specified channel
    fn write_neuron_data_single_channel(&self, wrapped_value: &WrappedIOData, cortical_channel: CorticalChannelIndex, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError>;
    
    /// Encodes multiple channels of data in a single operation.
    ///
    /// This method processes multiple cortical channels simultaneously, which can
    /// be more efficient than individual single-channel calls. The default
    /// implementation calls `write_neuron_data_single_channel` for each channel,
    /// but can be overridden for better performance.
    ///
    /// # Arguments
    /// * `channels_and_values` - Map of channel indices to their corresponding data
    /// * `write_target` - Cortical neuron data collection to write activations to
    ///
    /// # Returns
    /// * `Ok(())` - All channels encoded successfully
    /// * `Err(FeagiDataProcessingError)` - Encoding failed for at least one channel
    ///
    /// # Performance Notes
    /// Consider overriding this method for:
    /// - Vectorized operations when processing many channels
    /// - Batch optimizations specific to the encoding algorithm
    /// - Shared computation that can be amortized across channels
    fn write_neuron_data_multi_channel(&self, channels_and_values: HashMap<CorticalChannelIndex, &WrappedIOData>, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        for (channel, values) in channels_and_values {
            self.write_neuron_data_single_channel(values, channel, write_target)?;
        };
        Ok(())
    }
}

