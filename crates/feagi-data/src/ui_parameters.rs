//! Temporarily here for now
//! some common structs and macros for defining interfaces that will have UI control

use std::cmp::PartialEq;

/// Used to lookup the translation of this parameter. All parameters have a unique index which
/// corresponds to a structure with text in various languages. The default name is used instead if
/// this is disabled or the lookup fails
pub type TranslationIndex = Option<u16>;

/// Enum that stores the current value state of a parameter, including specific metadata such as
/// ranges
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UIFieldParameterType {
    None,
    Decimal { min: f64, max: f64, value: f64 },
    Bool { value: bool },
}

/// Other Metadata surrounding a `UIFieldParameterType` that applies for all parameters
#[derive(Debug, Clone, PartialEq)]
pub struct UIFieldGeneralParameter {
    /// The name that is used when there is no translation index
    pub default_name: String,
    pub translation_index: TranslationIndex,
    pub parameter_type: UIFieldParameterType,
    pub disabled: bool,
}

/// When parameters may have changes, this represents the possibility of a change
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UIFieldDataChangeType {
    NoChange,
    Decimal(f64),
    Bool(bool),
}

pub struct UIFieldGeneralList {
    pub list_name: String,
    pub parameters: Vec<UIFieldGeneralParameter>,
}

pub struct UIFieldDataChangeList {
    pub optional_changes: Vec<UIFieldDataChangeType>,
}

impl UIFieldGeneralList {
    pub fn new(list_name: String, parameters: Vec<UIFieldGeneralParameter>) -> UIFieldGeneralList {
        UIFieldGeneralList { list_name, parameters }
    }

    pub fn update_from_changes(&mut self, changes: UIFieldDataChangeList) -> Result<(), ()> {
        if changes.optional_changes.len() != self.parameters.len() {
            return Err(());
        }

        self.parameters.iter_mut().zip(changes.optional_changes).for_each(|(current, change)| {
            match change {
                UIFieldDataChangeType::NoChange => { /* Do Nothing */ }
                UIFieldDataChangeType::Decimal(d) => {
                    match &mut current.parameter_type {
                        UIFieldParameterType::Decimal { min, max, value } => *value = d,
                        _ => { /* continue */ }
                    }
                }
                UIFieldDataChangeType::Bool(b) => {
                    match &mut current.parameter_type {
                        UIFieldParameterType::Bool { value } => *value = b,
                        _ => { /* continue */ }
                    }
                }
            }
        });

        Ok(())
    }
}



macro_rules! static_parameters_builder {
    (

        $(  ),* $(,)?
    ) => {};
}
