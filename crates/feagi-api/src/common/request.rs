// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transport-agnostic API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,

    /// Request path (e.g., "/v1/cortical_area/ipu")
    pub path: String,

    /// Query parameters
    #[serde(default)]
    pub query: HashMap<String, String>,

    /// Request headers (optional)
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Request body as JSON string (optional)
    pub body: Option<String>,
}

impl ApiRequest {
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn with_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Parse body as JSON
    pub fn parse_body<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        match &self.body {
            Some(body) => serde_json::from_str(body),
            None => serde_json::from_str("{}"), // Return empty object if no body
        }
    }

    /// Get query parameter
    pub fn get_query(&self, key: &str) -> Option<&String> {
        self.query.get(key)
    }

    /// Get header value
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}
