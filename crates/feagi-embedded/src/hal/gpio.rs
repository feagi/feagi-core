// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/// GPIO abstraction for embedded platforms
pub trait GpioProvider {
    /// Platform-specific pin type (must be Copy for ease of use)
    type Pin: Copy;
    
    /// Platform-specific error type
    type Error;
    
    /// Set pin high
    /// 
    /// # Arguments
    /// * `pin` - Pin identifier
    /// 
    /// # Returns
    /// Ok(()) or error
    fn set_high(&mut self, pin: Self::Pin) -> Result<(), Self::Error>;
    
    /// Set pin low
    /// 
    /// # Arguments
    /// * `pin` - Pin identifier
    /// 
    /// # Returns
    /// Ok(()) or error
    fn set_low(&mut self, pin: Self::Pin) -> Result<(), Self::Error>;
    
    /// Read pin state
    /// 
    /// # Arguments
    /// * `pin` - Pin identifier
    /// 
    /// # Returns
    /// True if pin is high, false if low, or error
    fn is_high(&self, pin: Self::Pin) -> Result<bool, Self::Error>;
    
    /// Read pin state (inverted)
    /// 
    /// # Arguments
    /// * `pin` - Pin identifier
    /// 
    /// # Returns
    /// True if pin is low, false if high, or error
    fn is_low(&self, pin: Self::Pin) -> Result<bool, Self::Error> {
        Ok(!self.is_high(pin)?)
    }
    
    /// Toggle pin state
    /// 
    /// # Arguments
    /// * `pin` - Pin identifier
    /// 
    /// # Returns
    /// Ok(()) or error
    fn toggle(&mut self, pin: Self::Pin) -> Result<(), Self::Error> {
        if self.is_high(pin)? {
            self.set_low(pin)
        } else {
            self.set_high(pin)
        }
    }
}

