//! CLI argument parsing for per-crate debug flags
//!
//! Supports flags like `--debug-feagi-api`, `--debug-feagi-burst-engine`, etc.
//! to enable/disable observability per crate.

use std::collections::HashMap;
use std::env;

use crate::KNOWN_CRATES;

/// Parse debug flags from command-line arguments
///
/// # Example
/// ```rust
/// let flags = CrateDebugFlags::from_args(std::env::args().collect());
/// if flags.is_enabled("feagi-api") {
///     // Enable debug logging for feagi-api crate
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct CrateDebugFlags {
    pub enabled_crates: HashMap<String, bool>,
}

impl CrateDebugFlags {
    /// Parse debug flags from command-line arguments
    ///
    /// Looks for arguments matching `--debug-{crate-name}` pattern.
    /// Also supports `--debug-all` to enable all crates.
    pub fn from_args<I>(args: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mut enabled_crates = HashMap::new();
        let mut debug_all = false;

        for arg in args {
            if arg == "--debug-all" {
                debug_all = true;
                continue;
            }

            if let Some(crate_name) = arg.strip_prefix("--debug-") {
                enabled_crates.insert(crate_name.to_string(), true);
            }
        }

        if debug_all {
            // Enable all known crates
            for crate_name in KNOWN_CRATES {
                enabled_crates.insert(crate_name.to_string(), true);
            }
        }

        CrateDebugFlags { enabled_crates }
    }

    /// Check if debug is enabled for a specific crate
    pub fn is_enabled(&self, crate_name: &str) -> bool {
        self.enabled_crates.contains_key(crate_name)
    }

    /// Get all enabled crates
    pub fn enabled_crates(&self) -> Vec<&String> {
        self.enabled_crates.keys().collect()
    }

    /// Check if debug is enabled for any crate
    pub fn any_enabled(&self) -> bool {
        !self.enabled_crates.is_empty()
    }

    /// Get log level filter for a crate
    ///
    /// Returns `tracing::Level::DEBUG` if enabled, `tracing::Level::INFO` otherwise.
    pub fn log_level(&self, crate_name: &str) -> tracing::Level {
        if self.is_enabled(crate_name) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        }
    }

    /// Create a tracing filter from debug flags
    ///
    /// Returns a filter string that can be used with `EnvFilter`.
    /// Format: "feagi-api=debug,feagi-burst-engine=debug" or "info" if none enabled.
    pub fn to_filter_string(&self) -> String {
        if self.enabled_crates.is_empty() {
            return "info".to_string();
        }

        let mut filters = Vec::new();
        for crate_name in self.enabled_crates.keys() {
            filters.push(format!("{}=debug", crate_name));
        }
        // Set default level for other crates
        filters.push("info".to_string());
        filters.join(",")
    }
}

/// Helper function to parse debug flags from environment
///
/// Checks both command-line arguments and `FEAGI_DEBUG` environment variable.
/// Environment variable format: comma-separated crate names, e.g., "feagi-api,feagi-burst-engine"
pub fn parse_debug_flags() -> CrateDebugFlags {
    let mut flags = CrateDebugFlags::from_args(env::args());

    // Also check environment variable
    if let Ok(env_var) = env::var("FEAGI_DEBUG") {
        if env_var == "all" {
            // Enable all crates
            for crate_name in KNOWN_CRATES {
                flags.enabled_crates.insert(crate_name.to_string(), true);
            }
        } else {
            // Parse comma-separated crate names
            for crate_name in env_var.split(',') {
                let crate_name = crate_name.trim();
                if !crate_name.is_empty() {
                    flags.enabled_crates.insert(crate_name.to_string(), true);
                }
            }
        }
    }

    flags
}

/// Generate help text for debug flags
pub fn debug_flags_help() -> String {
    format!(
        r#"Debug Flags:
  --debug-all                    Enable debug logging for all crates
  --debug-{{crate-name}}          Enable debug logging for specific crate

Available crates:
  {}
  
Environment Variable:
  FEAGI_DEBUG={{crate-name}}[,{{crate-name}}]  Enable debug for crates (comma-separated)
  FEAGI_DEBUG=all                               Enable debug for all crates

Examples:
  --debug-feagi-api
  --debug-feagi-api --debug-feagi-burst-engine
  FEAGI_DEBUG=feagi-api,feagi-burst-engine
"#,
        KNOWN_CRATES.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_crate_flag() {
        let flags = CrateDebugFlags::from_args(vec!["--debug-feagi-api".to_string()]);
        assert!(flags.is_enabled("feagi-api"));
        assert!(!flags.is_enabled("feagi-burst-engine"));
    }

    #[test]
    fn test_multiple_crate_flags() {
        let flags = CrateDebugFlags::from_args(vec![
            "--debug-feagi-api".to_string(),
            "--debug-feagi-burst-engine".to_string(),
        ]);
        assert!(flags.is_enabled("feagi-api"));
        assert!(flags.is_enabled("feagi-burst-engine"));
        assert!(!flags.is_enabled("feagi-bdu"));
    }

    #[test]
    fn test_debug_all() {
        let flags = CrateDebugFlags::from_args(vec!["--debug-all".to_string()]);
        for crate_name in KNOWN_CRATES {
            assert!(flags.is_enabled(crate_name), "{} should be enabled", crate_name);
        }
    }

    #[test]
    fn test_filter_string() {
        let flags = CrateDebugFlags::from_args(vec!["--debug-feagi-api".to_string()]);
        let filter = flags.to_filter_string();
        assert!(filter.contains("feagi-api=debug"));
    }

    #[test]
    fn test_log_level() {
        let flags = CrateDebugFlags::from_args(vec!["--debug-feagi-api".to_string()]);
        assert_eq!(flags.log_level("feagi-api"), tracing::Level::DEBUG);
        assert_eq!(flags.log_level("feagi-burst-engine"), tracing::Level::INFO);
    }
}

