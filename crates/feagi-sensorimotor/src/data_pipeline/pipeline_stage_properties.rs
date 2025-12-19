//! Configuration properties for pipeline stages.
//!
//! This enum defines all possible pipeline stage configurations.
//! Properties are serializable and can be dynamically updated at runtime.

use crate::data_pipeline::stages::{
    ImageFrameProcessorStage, ImageFrameQuickDiffStage, ImageFrameSegmentatorStage,
    ImagePixelValueCountThresholdStage,
};
use crate::data_types::descriptors::{ImageFrameProperties, SegmentedImageFrameProperties};
use crate::data_types::{GazeProperties, ImageFrameProcessor, ImageFrameSegmentator, Percentage};
use crate::wrapped_io_data::WrappedIOType;
use std::ops::RangeInclusive;

/// Macro to define the PipelineStageProperties enum with all its variants and implementations.
///
/// This macro generates:
/// - The enum definition with all variants
/// - `get_input_data_type()`, `get_output_data_type()`, `create_stage()` methods
/// - `Display` trait implementation
/// - `variant_name()` method
/// - Convenience constructors for each variant
///
/// # Usage
/// ```ignore
/// define_pipeline_stage_properties_enum! {
///     imports: {
///         use crate::data_pipeline::stages::ImageFrameProcessorStage;
///         // ... more imports
///     },
///
///     variants: {
///         /// Documentation for this variant
///         ImageFrameProcessor {
///             transformer_definition: ImageFrameProcessor,
///         } => {
///             input_type: WrappedIOType::ImageFrame(Some(*transformer_definition.get_input_image_properties())),
///             output_type: WrappedIOType::ImageFrame(Some(transformer_definition.get_output_image_properties())),
///             create_stage: ImageFrameProcessorStage::new_box(transformer_definition.clone()).unwrap(),
///             display: ("ImageFrameProcessor(transformer: {:?})", transformer_definition),
///         },
///
///         // ... more variants
///     }
/// }
/// ```
macro_rules! define_pipeline_stage_properties_enum {
    (
        variants: {
            $(
                $(#[$variant_meta:meta])*
                $variant_name:ident {
                    $($field_name:ident : $field_type:ty),* $(,)?
                } => {
                    input_type: $input_expr:expr,
                    output_type: $output_expr:expr,
                    create_stage: $create_stage_expr:expr,
                    display: ($display_format:expr $(, $display_field:ident)*),
                }
            ),* $(,)?
        }
    ) => {

        /// Enum representing all possible pipeline stage configurations.
        ///
        /// Each variant contains the specific configuration data needed to create
        /// and configure its corresponding pipeline stage.
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        pub enum PipelineStageProperties {
            $(
                $(#[$variant_meta])*
                $variant_name {
                    $($field_name: $field_type),*
                },
            )*
        }

        impl PipelineStageProperties {
            /// Returns the data type this stage expects as input.
            pub fn get_input_data_type(&self) -> $crate::wrapped_io_data::WrappedIOType {
                match self {
                    $(
                        Self::$variant_name { $($field_name),* } => {
                            $input_expr
                        }
                    ),*
                }
            }

            /// Returns the data type this stage produces as output.
            pub fn get_output_data_type(&self) -> $crate::wrapped_io_data::WrappedIOType {
                match self {
                    $(
                        Self::$variant_name { $($field_name),* } => {
                            $output_expr
                        }
                    ),*
                }
            }

            /// Creates the corresponding pipeline stage from these properties.
            pub fn create_stage(&self) -> Box<dyn $crate::data_pipeline::pipeline_stage::PipelineStage> {
                match self {
                    $(
                        Self::$variant_name { $($field_name),* } => {
                            $create_stage_expr
                        }
                    ),*
                }
            }

            /// Returns the variant name as a string for display purposes
            pub fn variant_name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant_name { .. } => stringify!($variant_name),
                    )*
                }
            }
        }

        impl std::fmt::Display for PipelineStageProperties {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant_name { $($field_name),* } => {
                            write!(f, $display_format $(, $display_field)*)
                        }
                    ),*
                }
            }
        }

        // Convenience constructors
        impl PipelineStageProperties {
            $(
                paste::paste! {
                    #[doc = "Creates " $variant_name " properties"]
                    pub fn [<new_ $variant_name:snake>]($($field_name: $field_type),*) -> Self {
                        Self::$variant_name { $($field_name),* }
                    }
                }
            )*
        }
    };
}

define_pipeline_stage_properties_enum! {
    variants: {
        /// Properties for ImageFrameProcessorStage that configures various image modification transformations
        ImageFrameProcessor {
            transformer_definition: ImageFrameProcessor,
        } => {
            input_type: WrappedIOType::ImageFrame(Some(*transformer_definition.get_input_image_properties())),
            output_type: WrappedIOType::ImageFrame(Some(transformer_definition.get_output_image_properties())),
            create_stage: ImageFrameProcessorStage::new_box(transformer_definition.clone()).unwrap(),
            display: ("ImageFrameProcessor(transformer: {:?})", transformer_definition),
        },

        /// Properties for ImageFrameSegmentatorStage that store configuration for image segmentation
        ImageFrameSegmentator {
            input_image_properties: ImageFrameProperties,
            output_image_properties: SegmentedImageFrameProperties,
            segmentation_gaze: GazeProperties,
        } => {
            input_type: WrappedIOType::ImageFrame(Some(*input_image_properties)),
            output_type: WrappedIOType::SegmentedImageFrame(Some(*output_image_properties)),
            create_stage: ImageFrameSegmentatorStage::new_box(
                *input_image_properties,
                *output_image_properties,
                ImageFrameSegmentator::new(*input_image_properties, *output_image_properties, *segmentation_gaze).unwrap()
            ).unwrap(),
            display: ("ImageFrameSegmentator(input: {:?}, output: {:?}, gaze: {:?})", input_image_properties, output_image_properties, segmentation_gaze),
        },

        /// Properties for ImageFrameQuickDiffStage that configures quick difference detection
        /// between consecutive image frames.
        ImageQuickDiff {
            per_pixel_allowed_range: RangeInclusive<u8>,
            acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
            image_properties: ImageFrameProperties,
        } => {
            input_type: WrappedIOType::ImageFrame(Some(*image_properties)),
            output_type: WrappedIOType::ImageFrame(Some(*image_properties)),
            create_stage: ImageFrameQuickDiffStage::new_box(
                *image_properties,
                per_pixel_allowed_range.clone(),
                acceptable_amount_of_activity_in_image.clone()
            ).unwrap(),
            display: ("ImageQuickDiff(pixel_range: {:?}, activity: {:?}, image: {:?})", per_pixel_allowed_range, acceptable_amount_of_activity_in_image, image_properties),
        },

        /// Properties for ImagePixelValueCountThresholdStage checks for an image global pixel threshold
        ImagePixelValueCountThreshold {
            input_definition: ImageFrameProperties,
            inclusive_pixel_range: RangeInclusive<u8>,
            acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
        } => {
            input_type: WrappedIOType::ImageFrame(Some(*input_definition)),
            output_type: WrappedIOType::ImageFrame(Some(*input_definition)),
            create_stage: ImagePixelValueCountThresholdStage::new_box(
                *input_definition,
                inclusive_pixel_range.clone(),
                acceptable_amount_of_activity_in_image.clone(),
            ).unwrap(),
            display: ("ImagePixelValueCountThreshold(input: {:?}, pixel_range: {:?}, activity: {:?})", input_definition, inclusive_pixel_range, acceptable_amount_of_activity_in_image),
        },
    }
}
