// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Genome Storage Abstraction

Provides platform-agnostic storage interface for genome persistence:
- Desktop: File system storage
- WASM: IndexedDB storage (implemented in feagi-wasm)

This trait allows FEAGI to work seamlessly across platforms without
hardcoding platform-specific storage mechanisms.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use core::future::Future;

/// Storage errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// Genome not found
    NotFound,
    /// I/O error (file system, network, etc.)
    IOError(String),
    /// Serialization/deserialization error
    SerializationError(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::NotFound => write!(f, "Genome not found"),
            StorageError::IOError(msg) => write!(f, "I/O error: {}", msg),
            StorageError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

/// Platform-agnostic genome storage trait
///
/// This trait abstracts genome persistence across platforms:
/// - Desktop: File system storage
/// - WASM: IndexedDB storage
///
/// All operations are async to support both blocking (file I/O) and
/// non-blocking (IndexedDB) storage backends.
pub trait GenomeStorage: Send + Sync {
    /// Load a genome by ID
    ///
    /// # Arguments
    ///
    /// * `genome_id` - Unique identifier for the genome
    ///
    /// # Returns
    ///
    /// `Ok(String)` with genome JSON if found,
    /// `Err(StorageError::NotFound)` if genome doesn't exist,
    /// `Err(StorageError::IOError)` for I/O failures
    fn load_genome(
        &self,
        genome_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, StorageError>> + Send + '_>>;

    /// Save a genome by ID
    ///
    /// # Arguments
    ///
    /// * `genome_id` - Unique identifier for the genome
    /// * `genome_json` - Genome JSON string to save
    ///
    /// # Returns
    ///
    /// `Ok(())` on success,
    /// `Err(StorageError::IOError)` for I/O failures,
    /// `Err(StorageError::SerializationError)` for invalid JSON
    fn save_genome(
        &self,
        genome_id: &str,
        genome_json: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send + '_>>;

    /// List all available genome IDs
    ///
    /// # Returns
    ///
    /// `Ok(Vec<String>)` with all genome IDs,
    /// `Err(StorageError::IOError)` for I/O failures
    fn list_genomes(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, StorageError>> + Send + '_>>;

    /// Delete a genome by ID
    ///
    /// # Arguments
    ///
    /// * `genome_id` - Unique identifier for the genome to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` on success,
    /// `Err(StorageError::NotFound)` if genome doesn't exist,
    /// `Err(StorageError::IOError)` for I/O failures
    fn delete_genome(
        &self,
        genome_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send + '_>>;
}

// Re-export Pin for convenience
use core::pin::Pin;

#[cfg(feature = "async-tokio")]
pub mod fs_storage {
    //! File system storage implementation for desktop platforms
    //!
    //! Uses async file I/O via tokio for non-blocking operations.

    use super::{GenomeStorage, StorageError};
    use std::path::{Path, PathBuf};
    use core::pin::Pin;
    use core::future::Future;

    /// File system-based genome storage
    ///
    /// Stores genomes as JSON files in a directory structure:
    /// ```
    /// base_path/
    ///   genome_id_1.json
    ///   genome_id_2.json
    ///   ...
    /// ```
    pub struct FileSystemStorage {
        base_path: PathBuf,
    }

    impl FileSystemStorage {
        /// Create a new file system storage instance
        ///
        /// # Arguments
        ///
        /// * `base_path` - Base directory for storing genomes
        ///
        /// # Errors
        ///
        /// Returns error if base_path doesn't exist or can't be created
        pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self, StorageError> {
            let path = base_path.as_ref().to_path_buf();
            
            // Create directory if it doesn't exist
            std::fs::create_dir_all(&path)
                .map_err(|e| StorageError::IOError(format!("Failed to create directory: {}", e)))?;
            
            Ok(Self { base_path: path })
        }

        /// Get the file path for a genome ID
        fn genome_path(&self, genome_id: &str) -> PathBuf {
            // Sanitize genome_id to prevent path traversal
            let sanitized = genome_id
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect::<String>();
            
            self.base_path.join(format!("{}.json", sanitized))
        }
    }

    #[cfg(feature = "async-tokio")]
    impl GenomeStorage for FileSystemStorage {
        fn load_genome(
            &self,
            genome_id: &str,
        ) -> Pin<Box<dyn Future<Output = Result<String, StorageError>> + Send + '_>> {
            let path = self.genome_path(genome_id);
            Box::pin(async move {
                tokio::fs::read_to_string(&path)
                    .await
                    .map_err(|e| {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            StorageError::NotFound
                        } else {
                            StorageError::IOError(format!("Failed to read file: {}", e))
                        }
                    })
            })
        }

        fn save_genome(
            &self,
            genome_id: &str,
            genome_json: &str,
        ) -> Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send + '_>> {
            let path = self.genome_path(genome_id);
            let json = genome_json.to_string();
            
            Box::pin(async move {
                // Validate JSON before saving
                serde_json::from_str::<serde_json::Value>(&json)
                    .map_err(|e| StorageError::SerializationError(format!("Invalid JSON: {}", e)))?;
                
                tokio::fs::write(&path, json)
                    .await
                    .map_err(|e| StorageError::IOError(format!("Failed to write file: {}", e)))?;
                
                Ok(())
            })
        }

        fn list_genomes(
            &self,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, StorageError>> + Send + '_>> {
            let base_path = self.base_path.clone();
            Box::pin(async move {
                let mut entries = tokio::fs::read_dir(&base_path)
                    .await
                    .map_err(|e| StorageError::IOError(format!("Failed to read directory: {}", e)))?;
                
                let mut genome_ids = Vec::new();
                while let Some(entry) = entries.next_entry().await
                    .map_err(|e| StorageError::IOError(format!("Failed to read directory entry: {}", e)))?
                {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            genome_ids.push(stem.to_string());
                        }
                    }
                }
                
                Ok(genome_ids)
            })
        }

        fn delete_genome(
            &self,
            genome_id: &str,
        ) -> Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send + '_>> {
            let path = self.genome_path(genome_id);
            Box::pin(async move {
                tokio::fs::remove_file(&path)
                    .await
                    .map_err(|e| {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            StorageError::NotFound
                        } else {
                            StorageError::IOError(format!("Failed to delete file: {}", e))
                        }
                    })
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(feature = "async-tokio")]
    #[tokio::test]
    async fn test_filesystem_storage() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemStorage::new(temp_dir.path()).unwrap();
        
        // Test save
        let genome_id = "test_genome";
        let genome_json = r#"{"genome_id": "test_genome", "version": "2.1"}"#;
        storage.save_genome(genome_id, genome_json).await.unwrap();
        
        // Test load
        let loaded = storage.load_genome(genome_id).await.unwrap();
        assert_eq!(loaded, genome_json);
        
        // Test list
        let genomes = storage.list_genomes().await.unwrap();
        assert!(genomes.contains(&genome_id.to_string()));
        
        // Test delete
        storage.delete_genome(genome_id).await.unwrap();
        
        // Verify deleted
        let result = storage.load_genome(genome_id).await;
        assert!(matches!(result, Err(StorageError::NotFound)));
    }
}

