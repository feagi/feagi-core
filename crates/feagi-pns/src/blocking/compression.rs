// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! LZ4 compression utilities for blocking transports

use crate::core::{PNSError, Result};

/// Compress data using LZ4 (fast mode)
///
/// # Arguments
/// - `data`: Raw bytes to compress
///
/// # Returns
/// - `Ok(Vec<u8>)`: Compressed data
/// - `Err(PNSError)`: Compression failed
///
/// # Performance
/// - Uses LZ4 FAST(1) mode for maximum speed
/// - Typically achieves 2-4x compression for neural data
/// - ~200-500 MB/s compression throughput on modern CPUs
pub fn compress_lz4(data: &[u8]) -> Result<Vec<u8>> {
    lz4::block::compress(data, Some(lz4::block::CompressionMode::FAST(1)), true)
        .map_err(|e| PNSError::Transport(format!("LZ4 compression failed: {}", e)))
}

/// Decompress LZ4 data
///
/// # Arguments
/// - `compressed`: LZ4-compressed bytes
/// - `max_size`: Maximum expected decompressed size (safety limit)
///
/// # Returns
/// - `Ok(Vec<u8>)`: Decompressed data
/// - `Err(PNSError)`: Decompression failed or size exceeded
pub fn decompress_lz4(compressed: &[u8], max_size: i32) -> Result<Vec<u8>> {
    lz4::block::decompress(compressed, Some(max_size))
        .map_err(|e| PNSError::Transport(format!("LZ4 decompression failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let data = vec![42u8; 1000]; // Highly compressible
        let compressed = compress_lz4(&data).unwrap();
        assert!(compressed.len() < data.len());

        // LZ4 prepends the size, so we can decompress without specifying max size
        let decompressed = lz4::block::decompress(&compressed, None).unwrap();
        assert_eq!(decompressed, data);
    }
}
