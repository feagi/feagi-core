//! Internal processing utilities for data type transformations.
//!
//! Provides helper functions and processors for manipulating image frames
//! and other data types. Not part of the public API.

mod audio_spectrum_codec;
mod image_frame_processor;
mod image_frame_segmentator;

pub use audio_spectrum_codec::{AudioSpectrumAnalyzer, AudioSpectrumSynthesizer};
pub use image_frame_processor::ImageFrameProcessor;
pub use image_frame_segmentator::ImageFrameSegmentator;
