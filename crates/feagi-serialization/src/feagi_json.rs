//! Arbitrary JSON data as a FEAGI serializable structure.

// TODO `from_json_string` surfaces `serde_json::Error` directly. Revisit once this crate has its
// own error type; the rest of the crate currently returns `Result<_, ()>` as a placeholder.

/// A wrapper around [`serde_json::Value`] for handling JSON data in FEAGI.
///
/// Corresponds to [`crate::FeagiByteStructureType::JSON`], and is serialized as UTF-8 text.
#[derive(Clone, Debug, Hash)]
pub struct FeagiJSON {
    json: serde_json::Value,
}

impl FeagiJSON {
    /// Creates an empty JSON object.
    ///
    /// # Example
    /// ```
    /// use feagi_serialization::FeagiJSON;
    ///
    /// let json = FeagiJSON::new_empty();
    /// assert_eq!(json.to_string(), "{}");
    /// ```
    pub fn new_empty() -> FeagiJSON {
        FeagiJSON {
            json: serde_json::json!({}),
        }
    }

    /// Parses a JSON string into a `FeagiJSON` wrapper.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`serde_json::Error`] if the string is not valid JSON.
    ///
    /// # Example
    /// ```
    /// use feagi_serialization::FeagiJSON;
    ///
    /// let json = FeagiJSON::from_json_string(r#"{"key": "value"}"#.to_string()).unwrap();
    /// assert_eq!(json.borrow_json_value()["key"], "value");
    ///
    /// assert!(FeagiJSON::from_json_string("not json".to_string()).is_err());
    /// ```
    pub fn from_json_string(string: String) -> Result<FeagiJSON, serde_json::Error> {
        serde_json::from_str(&string).map(|json| FeagiJSON { json })
    }

    /// Creates a `FeagiJSON` from an existing [`serde_json::Value`].
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
