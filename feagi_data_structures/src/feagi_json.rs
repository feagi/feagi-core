use serde_json;
use crate::FeagiDataError;

/// A wrapper around serde_json::Value for handling JSON data in FEAGI.
/// 
/// Provides methods to create, parse, and manipulate JSON data with error handling.
#[derive(Clone, Debug)]
pub struct FeagiJSON {
    json : serde_json::Value,
}

impl FeagiJSON {
    pub fn new_empty() -> FeagiJSON {
        FeagiJSON {
            json: serde_json::json!({}),
        }
    }

    pub fn from_json_string(string: String) -> Result<FeagiJSON, FeagiDataError> {
        match serde_json::from_str(&string) {
            Ok(json_value) => Ok(FeagiJSON { json: json_value }),
            Err(e) => Err(FeagiDataError::BadParameters(
                format!("Failed to parse JSON string: {}", e)
            ).into()),
        }
    }

    /// Creates a FeagiJSON from an existing serde_json::Value.
    pub fn from_json_value(value: serde_json::Value) -> FeagiJSON {
        FeagiJSON { json: value }
    }

    /// Returns a reference to the internal JSON value.
    pub fn borrow_json_value(&self) -> &serde_json::Value {
        &self.json
    }

    /// Updates the internal JSON value.
    pub fn update_json_value(&mut self, new_value: serde_json::Value) {
        self.json = new_value;
    }
}

impl std::fmt::Display for FeagiJSON {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.json)
    }
}

