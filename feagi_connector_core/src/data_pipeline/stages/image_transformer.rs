//! Image transformation stream processor for FEAGI vision processing pipelines.
//!
//! This module provides the `ImageFrameTransformerProcessor` which applies a configured
//! set of image transformations to incoming frames in a stream processing pipeline.
//! It wraps an `ImageFrameTransformerDefinition` to provide streaming functionality
//! with caching and efficient processing.

use std::fmt::Display;
use std::time::Instant;
use feagi_data_structures::data::ImageFrame;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::processing::ImageFrameProcessor;
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::stream_cache_processor_trait::StreamCacheStage;

/// A stream processor that applies image transformations to incoming frames.
///
/// This processor wraps an `ImageFrameTransformerDefinition` to provide streaming
/// functionality for image transformation pipelines. It maintains an internal cache
/// of the most recent output and applies the configured transformations (cropping,
/// resizing, color adjustments, etc.) to each incoming frame.
///
/// # Transformation Pipeline
///
/// The processor applies transformations in the optimal order defined by the
/// underlying `ImageFrameTransformerDefinition`:
/// 1. **Cropping** - Extract regions of interest
/// 2. **Resizing** - Scale to target dimensions  
/// 3. **Color space conversion** - Convert between Linear/Gamma
/// 4. **Brightness adjustment** - Modify pixel intensity
/// 5. **Contrast adjustment** - Adjust image contrast
/// 6. **Grayscale conversion** - Convert to single-channel
///
/// # Performance
///
/// The processor includes optimized fast paths for common transformation combinations
/// and maintains a pre-allocated output buffer to minimize memory allocations during
/// stream processing.
#[derive(Debug, Clone)]
pub struct ImageFrameProcessorStage {
    /// The transformation configuration defining which operations to apply and their parameters
    transformer_definition: ImageFrameProcessor,
    /// Cached output buffer containing the most recent transformed image
    cached: WrappedIOData, // Image Frame
}

impl Display for ImageFrameProcessorStage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ImageFrameTransformerProcessor({:?})", self.transformer_definition)
    }
}

impl StreamCacheStage for ImageFrameProcessorStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(*self.transformer_definition.get_input_image_properties()))
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.transformer_definition.get_output_image_properties()))
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.cached
    }

    fn process_new_input(&mut self, value: &WrappedIOData, _time_of_input: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        let read_from: &ImageFrame = value.try_into()?;
        let write_target: &mut ImageFrame = (&mut self.cached).try_into()?;
        self.transformer_definition.process_image(read_from, write_target)?;
        Ok(&self.cached)
    }
}

impl ImageFrameProcessorStage {
    /// Creates a new ImageFrameTransformerProcessor with the specified transformation definition.
    ///
    /// Initializes the processor with a pre-allocated output buffer sized according to the
    /// expected output properties of the transformation pipeline. The buffer is created based
    /// on the final resolution, color space, and channel layout after all transformations.
    ///
    /// # Arguments
    ///
    /// * `transformer_definition` - Configuration defining the transformation pipeline to apply
    ///
    /// # Returns
    ///
    /// * `Ok(ImageFrameProcessor)` - Successfully created processor
    /// * `Err(FeagiDataError)` - If output buffer allocation fails
    pub fn new(transformer_definition: ImageFrameProcessor) -> Result<Self, FeagiDataError> {
        Ok(ImageFrameProcessorStage{
            cached: WrappedIOData::ImageFrame(ImageFrame::from_image_frame_properties(&transformer_definition.get_output_image_properties())?),
            transformer_definition,
        })
    }
}