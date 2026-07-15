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


pub trait UIParameterValue: Sized + Copy {
    fn to_parameter_type(self) -> UIFieldParameterType;
    fn try_from_change(change: UIFieldDataChangeType) -> Result<Option<Self>, ()>;
}

impl UIParameterValue for f64 {
    fn to_parameter_type(self) -> UIFieldParameterType {
        UIFieldParameterType::Decimal { min: f64::MIN, max: f64::MAX, value: self }
    }

    fn try_from_change(change: UIFieldDataChangeType) -> Result<Option<Self>, ()> {
        match change {
            UIFieldDataChangeType::NoChange => Ok(None),
            UIFieldDataChangeType::Decimal(d) => Ok(Some(d)),
            UIFieldDataChangeType::Bool(_) => Err(()),
        }
    }
}

impl UIParameterValue for bool {
    fn to_parameter_type(self) -> UIFieldParameterType {
        UIFieldParameterType::Bool { value: self }
    }

    fn try_from_change(change: UIFieldDataChangeType) -> Result<Option<Self>, ()> {
        match change {
            UIFieldDataChangeType::NoChange => Ok(None),
            UIFieldDataChangeType::Bool(b) => Ok(Some(b)),
            UIFieldDataChangeType::Decimal(_) => Err(()),
        }
    }
}



/// Builds a concrete "static" version of the general parameter structs above.
macro_rules! static_parameters_builder {
    (
        $parent:ident, $change:ident {
            $( $field:ident : $value_ty:ty ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $parent {
            $(
                pub $field: ($value_ty, $crate::ui_parameters::TranslationIndex, bool),
            )*
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct $change {
            $(
                pub $field: Option<$value_ty>,
            )*
        }

        impl $parent {
            pub fn update_from_changes(&mut self, changes: $change) {
                $(
                    if let Some(value) = changes.$field {
                        self.$field.0 = value;
                    }
                )*
            }


            pub fn to_general_list(&self) -> $crate::ui_parameters::UIFieldGeneralList {
                let parameters = ::std::vec![
                    $(
                        $crate::ui_parameters::UIFieldGeneralParameter {
                            default_name: ::std::string::String::from(stringify!($field)),
                            translation_index: self.$field.1,
                            parameter_type:
                                $crate::ui_parameters::UIParameterValue::to_parameter_type(self.$field.0),
                            disabled: self.$field.2,
                        },
                    )*
                ];
                $crate::ui_parameters::UIFieldGeneralList::new(
                    ::std::string::String::from(stringify!($parent)),
                    parameters,
                )
            }
        }

        impl $change {
            pub fn try_from_change_list(
                list: $crate::ui_parameters::UIFieldDataChangeList,
            ) -> ::std::result::Result<Self, ()> {
                let mut changes = list.optional_changes.into_iter();
                let result = Self {
                    $(
                        $field: {
                            let change = changes.next().ok_or(  () )?;
                            <$value_ty as $crate::ui_parameters::UIParameterValue>::try_from_change(change)
                                .map_err(|_| () )?
                        },
                    )*
                };
                if changes.next().is_some() {
                    return ::std::result::Result::Err(
                        ()
                    );
                }
                ::std::result::Result::Ok(result)
            }
        }
    };
}
