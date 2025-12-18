// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM-compatible type aliases for endpoint compatibility
//!
//! These types mimic axum types so endpoints can be called from WASM without modification.

use serde::Serialize;

/// WASM-compatible State wrapper (mimics axum::extract::State)
#[derive(Clone)]
pub struct State<T>(pub T);

impl<T> std::ops::Deref for State<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> State<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

/// WASM-compatible Json wrapper (mimics axum::response::Json)
pub struct Json<T>(pub T);

impl<T> std::ops::Deref for Json<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Json<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Serialize> Json<T> {
    pub fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.0).unwrap_or(serde_json::Value::Null)
    }
}

/// WASM-compatible Query extractor (mimics axum::extract::Query)
#[derive(Debug, Clone)]
pub struct Query<T>(pub T);

impl<T> std::ops::Deref for Query<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Query<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

/// WASM-compatible Path extractor (mimics axum::extract::Path)
#[derive(Debug, Clone)]
pub struct Path<T>(pub T);

impl<T> std::ops::Deref for Path<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Path<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}
