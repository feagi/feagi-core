use feagi_connector_core::data_types::descriptors::ColorSpace;
use feagi_connector_core::data_types::ImageFrame;

fn load_bird_image() -> ImageFrame {
    let bird_bytes = std::fs::read("tests/images/bird.jpg").expect("Bird image should exist");
    ImageFrame::new_from_jpeg_bytes(&bird_bytes, &ColorSpace::Gamma).expect("Bird image should load correctly")
}

