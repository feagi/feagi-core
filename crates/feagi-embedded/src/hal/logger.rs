// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/// Logging abstraction for embedded platforms
pub trait Logger {
    /// Log a message with specified level
    /// 
    /// # Arguments
    /// * `level` - Log level
    /// * `message` - Message to log
    fn log(&self, level: LogLevel, message: &str);
    
    /// Log an error message
    fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }
    
    /// Log a warning message
    fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }
    
    /// Log an info message
    fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }
    
    /// Log a debug message
    fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }
    
    /// Log a trace message
    fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }
}

/// Log level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Error: critical failures
    Error = 1,
    /// Warning: potential issues
    Warn = 2,
    /// Info: general information
    Info = 3,
    /// Debug: detailed debugging info
    Debug = 4,
    /// Trace: very detailed tracing
    Trace = 5,
}

impl LogLevel {
    /// Get log level as string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

