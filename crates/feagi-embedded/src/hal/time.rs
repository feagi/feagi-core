/// Time and delay abstraction for embedded platforms
pub trait TimeProvider {
    /// Get current time in microseconds since system boot
    /// 
    /// # Returns
    /// Monotonic timestamp in microseconds
    fn get_time_us(&self) -> u64;
    
    /// Block for the specified number of microseconds
    /// 
    /// # Arguments
    /// * `us` - Microseconds to delay
    fn delay_us(&self, us: u32);
    
    /// Block for the specified number of milliseconds
    /// 
    /// # Arguments
    /// * `ms` - Milliseconds to delay
    fn delay_ms(&self, ms: u32) {
        self.delay_us(ms * 1000);
    }
    
    /// Block for the specified number of seconds
    /// 
    /// # Arguments
    /// * `s` - Seconds to delay
    fn delay_s(&self, s: u32) {
        self.delay_ms(s * 1000);
    }
}

