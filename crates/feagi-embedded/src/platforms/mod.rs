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

#[cfg(feature = "arduino-due")]
pub mod arduino;

#[cfg(feature = "stm32f4")]
pub mod stm32;

#[cfg(feature = "rpi-pico")]
pub mod rpi_pico;

#[cfg(feature = "hailo")]
pub mod hailo;

// Re-export platform types
#[cfg(feature = "esp32")]
pub use esp32::Esp32Platform;

#[cfg(feature = "arduino-due")]
pub use arduino::ArduinoDuePlatform;

#[cfg(feature = "stm32f4")]
pub use stm32::Stm32F4Platform;

#[cfg(feature = "rpi-pico")]
pub use rpi_pico::RpiPicoPlatform;

#[cfg(feature = "hailo")]
pub use hailo::{Hailo8Accelerator, HailoError, HybridCpuHailo};

// Platform registry for runtime selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    #[cfg(feature = "esp32")]
    Esp32,
    #[cfg(feature = "esp32-s3")]
    Esp32S3,
    #[cfg(feature = "esp32-c3")]
    Esp32C3,
    #[cfg(feature = "arduino-due")]
    ArduinoDue,
    #[cfg(feature = "stm32f4")]
    Stm32F4,
    #[cfg(feature = "rpi-pico")]
    RaspberryPiPico,
    #[cfg(feature = "hailo")]
    Hailo8,
    // Future platforms
    // #[cfg(feature = "nrf52")]
    // Nrf52,
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
            #[cfg(feature = "arduino-due")]
            PlatformType::ArduinoDue => "Arduino Due",
            #[cfg(feature = "stm32f4")]
            PlatformType::Stm32F4 => "STM32F4",
            #[cfg(feature = "rpi-pico")]
            PlatformType::RaspberryPiPico => "Raspberry Pi Pico",
            #[cfg(feature = "hailo")]
            PlatformType::Hailo8 => "Hailo-8 Neural Accelerator",
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
        
        #[cfg(feature = "arduino-due")]
        return Some(PlatformType::ArduinoDue);
        
        #[cfg(feature = "stm32f4")]
        return Some(PlatformType::Stm32F4);
        
        #[cfg(feature = "rpi-pico")]
        return Some(PlatformType::RaspberryPiPico);
        
        #[cfg(feature = "hailo")]
        return Some(PlatformType::Hailo8);
        
        #[allow(unreachable_code)]
        None
    }
}

