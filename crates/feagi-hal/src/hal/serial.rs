// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/// Serial I/O abstraction for embedded platforms
pub trait SerialIO {
    /// Platform-specific error type
    type Error;

    /// Write bytes to serial port
    ///
    /// # Arguments
    /// * `data` - Bytes to write
    ///
    /// # Returns
    /// Number of bytes written or error
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error>;

    /// Read bytes from serial port (non-blocking)
    ///
    /// # Arguments
    /// * `buffer` - Buffer to read into
    ///
    /// # Returns
    /// Number of bytes read or error
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;

    /// Flush output buffer
    ///
    /// # Returns
    /// Ok(()) or error
    fn flush(&mut self) -> Result<(), Self::Error>;

    /// Check if data is available to read
    ///
    /// # Returns
    /// True if data is available
    fn available(&self) -> Result<bool, Self::Error> {
        // Default implementation - platforms can override
        Ok(false)
    }
}
