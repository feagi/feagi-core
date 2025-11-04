/// Platform implementations for embedded systems
/// 
/// Each platform module implements the HAL traits defined in `crate::hal`.
/// 
/// Available platforms:
/// - ESP32 family (ESP32, ESP32-S3, ESP32-C3)
/// - Arduino family (Due, Mega, Uno) - future
/// - STM32 family (F4, H7) - future
/// - ARM Cortex-M (Raspberry Pi Pico, nRF52) - future
/// - Neural accelerators (Hailo-8, Google Coral TPU) - future

#[cfg(feature = "esp32")]
pub mod esp32;

// Future platforms
// #[cfg(feature = "arduino-due")]
// pub mod arduino;

// #[cfg(feature = "stm32f4")]
// pub mod stm32;

// #[cfg(feature = "hailo")]
// pub mod hailo;

// Re-export platform types
#[cfg(feature = "esp32")]
pub use esp32::Esp32Platform;

// Platform registry for runtime selection (future feature)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    #[cfg(feature = "esp32")]
    Esp32,
    #[cfg(feature = "esp32-s3")]
    Esp32S3,
    #[cfg(feature = "esp32-c3")]
    Esp32C3,
    // Future platforms
    // #[cfg(feature = "arduino-due")]
    // ArduinoDue,
    // #[cfg(feature = "stm32f4")]
    // Stm32F4,
}

impl PlatformType {
    /// Get platform type name
    pub fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "esp32")]
            PlatformType::Esp32 => "ESP32",
            #[cfg(feature = "esp32-s3")]
            PlatformType::Esp32S3 => "ESP32-S3",
            #[cfg(feature = "esp32-c3")]
            PlatformType::Esp32C3 => "ESP32-C3 (RISC-V)",
            #[allow(unreachable_patterns)]
            _ => "Unknown",
        }
    }
    
    /// Detect platform at runtime
    pub fn detect() -> Option<Self> {
        #[cfg(feature = "esp32")]
        {
            #[cfg(esp32s3)]
            return Some(PlatformType::Esp32S3);
            #[cfg(esp32c3)]
            return Some(PlatformType::Esp32C3);
            #[cfg(esp32)]
            return Some(PlatformType::Esp32);
        }
        None
    }
}

