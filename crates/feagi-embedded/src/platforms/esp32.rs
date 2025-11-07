/// ESP32 platform implementation
/// 
/// Supports ESP32, ESP32-S3, ESP32-C3 (RISC-V) variants

use crate::hal::*;

#[cfg(feature = "esp32")]
use esp_idf_svc::{hal::uart::UartDriver, sys as esp_idf_sys};

/// ESP32 platform structure
#[cfg(feature = "esp32")]
pub struct Esp32Platform {
    uart: Option<UartDriver<'static>>,
}

#[cfg(feature = "esp32")]
impl Esp32Platform {
    /// Initialize ESP32 platform with default configuration
    /// 
    /// # Returns
    /// Initialized Esp32Platform or error
    ///
    /// # Example
    /// ```no_run
    /// let platform = Esp32Platform::init().expect("Failed to initialize ESP32");
    /// ```
    pub fn init() -> anyhow::Result<Self> {
        // Initialize ESP-IDF system
        esp_idf_sys::link_patches();
        
        log::info!("ESP32 platform initialized");
        
        Ok(Self {
            uart: None,
        })
    }
    
    /// Initialize with UART
    /// 
    /// # Arguments
    /// * `tx_pin` - TX pin number
    /// * `rx_pin` - RX pin number
    /// * `baudrate` - Baud rate (default: 115200)
    /// 
    /// # Returns
    /// Initialized Esp32Platform with UART or error
    pub fn init_with_uart(
        tx_pin: u32,
        rx_pin: u32,
        baudrate: u32,
    ) -> anyhow::Result<Self> {
        // Initialize ESP-IDF
        esp_idf_sys::link_patches();
        
        log::info!("ESP32 platform initializing with UART...");
        log::info!("  TX: GPIO{}, RX: GPIO{}", tx_pin, rx_pin);
        log::info!("  Baudrate: {}", baudrate);
        
        // TODO: Implement UART initialization
        // This requires accessing Peripherals which needs refactoring
        
        Ok(Self {
            uart: None,
        })
    }
    
    /// Get ESP32 chip model
    pub fn chip_model(&self) -> &'static str {
        #[cfg(feature = "esp32-s3")]
        {
            return "ESP32-S3";
        }
        #[cfg(feature = "esp32-c3")]
        {
            return "ESP32-C3";
        }
        "ESP32"
    }
}

#[cfg(feature = "esp32")]
impl TimeProvider for Esp32Platform {
    fn get_time_us(&self) -> u64 {
        unsafe { esp_idf_sys::esp_timer_get_time() as u64 }
    }
    
    fn delay_us(&self, us: u32) {
        unsafe { esp_idf_sys::esp_rom_delay_us(us) }
    }
}

#[cfg(feature = "esp32")]
impl SerialIO for Esp32Platform {
    type Error = esp_idf_sys::EspError;
    
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
        if let Some(uart) = &mut self.uart {
            uart.write(data)
        } else {
            // Fallback: log to console (no UART configured)
            if let Ok(s) = core::str::from_utf8(data) {
                log::info!("{}", s);
            }
            Ok(data.len())
        }
    }
    
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        if let Some(uart) = &mut self.uart {
            uart.read(buffer, 0)
        } else {
            Ok(0) // No UART configured
        }
    }
    
    fn flush(&mut self) -> Result<(), Self::Error> {
        if let Some(uart) = &mut self.uart {
            // Use wait_tx_done instead of flush for ESP-IDF
            uart.wait_tx_done(1000)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "esp32")]
impl Logger for Esp32Platform {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Error => log::error!("{}", message),
            LogLevel::Warn => log::warn!("{}", message),
            LogLevel::Info => log::info!("{}", message),
            LogLevel::Debug => log::debug!("{}", message),
            LogLevel::Trace => log::trace!("{}", message),
        }
    }
}

#[cfg(feature = "esp32")]
impl Platform for Esp32Platform {
    fn name(&self) -> &'static str {
        self.chip_model()
    }
    
    fn cpu_frequency_hz(&self) -> u32 {
        unsafe {
            (esp_idf_sys::esp_rom_get_cpu_ticks_per_us() as u64 * 1_000_000) as u32
        }
    }
    
    fn available_memory_bytes(&self) -> usize {
        unsafe { esp_idf_sys::esp_get_free_heap_size() as usize }
    }
}

// Placeholder for when esp32 feature is not enabled
#[cfg(not(feature = "esp32"))]
pub struct Esp32Platform;

#[cfg(not(feature = "esp32"))]
impl Esp32Platform {
    pub fn init() -> Result<Self, &'static str> {
        Err("ESP32 feature not enabled. Rebuild with --features esp32")
    }
}

