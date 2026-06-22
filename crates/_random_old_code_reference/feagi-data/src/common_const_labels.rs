//! Various structs which are used in const contexts to quickly describe something

pub struct NeuronModelConstLabel {
    pub model_name: &'static str,
    pub description: &'static str,
    pub author: &'static str,
    pub date_created: &'static str,
    pub date_modified: &'static str,
}

