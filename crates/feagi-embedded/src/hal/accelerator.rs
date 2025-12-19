// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/// Neural accelerator abstraction for hardware acceleration (Hailo, TPU, etc.)
pub trait NeuralAccelerator {
    /// Platform-specific error type
    type Error;
    
    /// Check if accelerator is available and ready
    /// 
    /// # Returns
    /// True if accelerator is ready to use
    fn is_available(&self) -> bool;
    
    /// Get accelerator name/identifier
    /// 
    /// # Returns
    /// Human-readable accelerator name
    fn name(&self) -> &'static str;
    
    /// Get accelerator performance metrics (TOPS, GOPS, etc.)
    /// 
    /// # Returns
    /// Performance in operations per second
    fn performance_ops_per_sec(&self) -> u64 {
        0 // Default: unknown
    }
    
    /// Upload neuron state to accelerator
    /// 
    /// # Arguments
    /// * `neurons` - Serialized neuron state data
    /// 
    /// # Returns
    /// Ok(()) or error
    fn upload_neurons(&mut self, neurons: &[u8]) -> Result<(), Self::Error>;
    
    /// Upload synapse connectivity to accelerator
    /// 
    /// # Arguments
    /// * `synapses` - Serialized synapse data
    /// 
    /// # Returns
    /// Ok(()) or error
    fn upload_synapses(&mut self, synapses: &[u8]) -> Result<(), Self::Error>;
    
    /// Process burst on accelerator
    /// 
    /// # Returns
    /// Number of neurons that fired, or error
    fn process_burst(&mut self) -> Result<u32, Self::Error>;
    
    /// Download updated neuron state from accelerator
    /// 
    /// # Arguments
    /// * `buffer` - Buffer to write neuron state into
    /// 
    /// # Returns
    /// Number of bytes written or error
    fn download_neurons(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
    
    /// Reset accelerator to initial state
    /// 
    /// # Returns
    /// Ok(()) or error
    fn reset(&mut self) -> Result<(), Self::Error> {
        // Default implementation: no-op
        Ok(())
    }
}

/// Accelerator capabilities
#[derive(Debug, Clone, Copy)]
pub struct AcceleratorCapabilities {
    /// Maximum neurons supported
    pub max_neurons: usize,
    
    /// Maximum synapses supported
    pub max_synapses: usize,
    
    /// Supported precisions (bitmask: bit 0 = INT8, bit 1 = FP16, bit 2 = FP32)
    pub supported_precisions: u8,
    
    /// Memory bandwidth (bytes/sec)
    pub memory_bandwidth_bytes_per_sec: u64,
    
    /// Power consumption (milliwatts)
    pub power_consumption_mw: u32,
}

impl AcceleratorCapabilities {
    /// Check if INT8 precision is supported
    pub fn supports_int8(&self) -> bool {
        (self.supported_precisions & 0b001) != 0
    }
    
    /// Check if FP16 precision is supported
    pub fn supports_fp16(&self) -> bool {
        (self.supported_precisions & 0b010) != 0
    }
    
    /// Check if FP32 precision is supported
    pub fn supports_fp32(&self) -> bool {
        (self.supported_precisions & 0b100) != 0
    }
}

