//! This library holds burst engine definitions for the family of ESP32 devices. Use crate features
//! to select which one you want to use. Please select only one at a time!


// The original ESP32 chip (Why Expressif, did you give this product the same name as the family???)
#[cfg(feature = "board-esp-32")]
pub mod esp_32;
