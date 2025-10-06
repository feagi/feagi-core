//! Tests for the data pipeline module - focusing on end -> end tests

use feagi_connector_core::data_types::descriptors::ColorSpace;
use feagi_connector_core::data_types::ImageFrame;

//region Helpers


fn load_bird_image() -> ImageFrame {
    let bird_bytes = std::fs::read("tests/images/bird.jpg").expect("Bird image should exist");
    ImageFrame::new_from_jpeg_bytes(&bird_bytes, &ColorSpace::Gamma).expect("Bird image should load correctly")
}

//endregion

#[cfg(test)]
mod test_connector_end_to_end {
    
}