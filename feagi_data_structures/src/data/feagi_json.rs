use serde_json;
use crate::FeagiDataError;


#[derive(Clone)]
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

    pub fn from_json_value(value: serde_json::Value) -> FeagiJSON {
        FeagiJSON { json: value }
    }

    pub fn borrow_json_value(&self) -> &serde_json::Value {
        &self.json
    }

    pub fn update_json_value(&mut self, new_value: serde_json::Value) {
        self.json = new_value;
    }
}

impl std::fmt::Display for FeagiJSON {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.json)
    }
}

