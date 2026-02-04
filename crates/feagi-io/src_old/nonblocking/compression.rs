// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Async LZ4 compression utilities for nonblocking transports

use crate::core::{IOError, Result};
use tokio::task;

/// Compress data using LZ4 asynchronously (offloaded to blocking threadpool)
///
/// # Arguments
/// - `data`: Raw bytes to compress
///
/// # Returns
/// - `Ok(Vec<u8>)`: Compressed data
/// - `Err(IOError)`: Compression failed
///
/// # Performance
/// - Runs compression in tokio's blocking threadpool to avoid blocking async tasks
/// - Uses LZ4 FAST(1) mode for maximum speed
/// - Suitable for async/await contexts
///
/// # Example
/// ```no_run
/// use feagi_io::nonblocking::compression;
///
/// async fn example() {
///     let data = vec![42u8; 1000];
///     let compressed = compression::compress_lz4_async(&data).await.unwrap();
///     println!("Compressed {} bytes to {}", data.len(), compressed.len());
/// }
/// ```
pub async fn compress_lz4_async(data: &[u8]) -> Result<Vec<u8>> {
    let data = data.to_vec(); // Clone for 'static lifetime
    task::spawn_blocking(move || {
        lz4::block::compress(&data, Some(lz4::block::CompressionMode::FAST(1)), true)
            .map_err(|e| IOError::Transport(format!("LZ4 compression failed: {}", e)))
    })
    .await
    .map_err(|e| IOError::Transport(format!("Task join error: {}", e)))?
}

/// Decompress LZ4 data asynchronously (offloaded to blocking threadpool)
///
/// # Arguments
/// - `compressed`: LZ4-compressed bytes
///
/// # Returns
/// - `Ok(Vec<u8>)`: Decompressed data
/// - `Err(IOError)`: Decompression failed
pub async fn decompress_lz4_async(compressed: &[u8]) -> Result<Vec<u8>> {
    let compressed = compressed.to_vec(); // Clone for 'static lifetime
    task::spawn_blocking(move || {
        lz4::block::decompress(&compressed, None)
            .map_err(|e| IOError::Transport(format!("LZ4 decompression failed: {}", e)))
    })
    .await
    .map_err(|e| IOError::Transport(format!("Task join error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_compress_decompress() {
        let data = vec![42u8; 1000]; // Highly compressible
        let compressed = compress_lz4_async(&data).await.unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = decompress_lz4_async(&compressed).await.unwrap();
        assert_eq!(decompressed, data);
    }
}
