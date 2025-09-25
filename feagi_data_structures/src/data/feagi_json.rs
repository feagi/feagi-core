use serde_json;
use crate::FeagiDataError;

/// A wrapper around serde_json::Value for handling JSON data in FEAGI.
/// 
/// Provides methods to create, parse, and manipulate JSON data with error handling.
/// 
/// # Example
/// ```
/// use feagi_data_structures::data::FeagiJSON;
/// 
/// let mut json = FeagiJSON::new_empty();
/// let json_str = r#"{"name": "test", "value": 42}"#;
/// let parsed = FeagiJSON::from_json_string(json_str.to_string()).unwrap();
/// println!("{}", parsed);
/// ```
#[derive(Clone)]
pub struct FeagiJSON {
    json : serde_json::Value,
}

impl FeagiJSON {
    /// Creates a new empty JSON object.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::FeagiJSON;
    /// 
    /// let json = FeagiJSON::new_empty();
    /// assert_eq!(json.to_string(), "{}");
    /// ```
    pub fn new_empty() -> FeagiJSON {
        FeagiJSON {
            json: serde_json::json!({}),
        }
    }

    /// Parses a JSON string into a FeagiJSON instance.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::FeagiJSON;
    /// 
    /// let json_str = r#"{"test": 42}"#;
    /// let json = FeagiJSON::from_json_string(json_str.to_string()).unwrap();
    /// assert!(json.borrow_json_value().is_object());
    /// ```
    pub fn from_json_string(string: String) -> Result<FeagiJSON, FeagiDataError> {
        match serde_json::from_str(&string) {
            Ok(json_value) => Ok(FeagiJSON { json: json_value }),
            Err(e) => Err(FeagiDataError::BadParameters(
                format!("Failed to parse JSON string: {}", e)
            ).into()),
        }
    }

    /// Creates a FeagiJSON from an existing serde_json::Value.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::FeagiJSON;
    /// use serde_json::json;
    /// 
    /// let value = json!({"key": "value"});
    /// let feagi_json = FeagiJSON::from_json_value(value);
    /// assert!(feagi_json.borrow_json_value().is_object());
    /// ```
    pub fn from_json_value(value: serde_json::Value) -> FeagiJSON {
        FeagiJSON { json: value }
    }

    /// Returns a reference to the internal JSON value.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::FeagiJSON;
    /// 
    /// let json = FeagiJSON::new_empty();
    /// let value = json.borrow_json_value();
    /// assert!(value.is_object());
    /// ```
    pub fn borrow_json_value(&self) -> &serde_json::Value {
        &self.json
    }

    /// Updates the internal JSON value.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::FeagiJSON;
    /// use serde_json::json;
    /// 
    /// let mut feagi_json = FeagiJSON::new_empty();
    /// feagi_json.update_json_value(json!({"updated": true}));
    /// assert_eq!(feagi_json.borrow_json_value()["updated"], true);
    /// ```
    pub fn update_json_value(&mut self, new_value: serde_json::Value) {
        self.json = new_value;
    }
}

impl std::fmt::Display for FeagiJSON {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.json)
    }
}

