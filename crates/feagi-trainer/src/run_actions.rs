//! Scheduled actions: timed changes to the running brain that any run can carry.
//!
//! A [`ScheduledEvent`] names an action and the time it is applied. The runner decides what
//! "time" is measured from; the caller supplies an [`ApplyEvent`] callback that performs the
//! action, so this module holds no transport and no task logic.

use serde::{Deserialize, Serialize};

use crate::error::TrainerError;

/// One change to the running brain, applied by the caller (the Trainer names areas as given:
/// cortical id or name, no lookup here).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScheduledAction {
    /// Scale the post-synaptic current of the mapping `src` -> `dst`.
    SetMappingPsc {
        src: String,
        dst: String,
        #[serde(alias = "pscMultiplier")]
        psc_multiplier: f64,
    },
    /// Set one parameter of an area.
    SetAreaParameter {
        area: String,
        key: String,
        value: serde_json::Value,
    },
}

/// An action applied `at_s` seconds after the runner's reference point (for example the end of
/// a settle window), within each attempt or pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduledEvent {
    /// Seconds after the runner's reference point. Finite and >= 0.
    #[serde(alias = "atS")]
    pub at_s: f64,
    /// Name recorded with the event. Non-empty.
    pub label: String,
    /// What to apply.
    pub action: ScheduledAction,
}

impl ScheduledEvent {
    /// Rejects a malformed event.
    pub fn validate(&self) -> Result<(), TrainerError> {
        if !self.at_s.is_finite() || self.at_s < 0.0 {
            return Err(TrainerError::Config(format!(
                "scheduled event {:?}: at_s must be finite and >= 0",
                self.label
            )));
        }
        if self.label.trim().is_empty() {
            return Err(TrainerError::Config(
                "scheduled event label must be non-empty".to_string(),
            ));
        }
        let named = |what: &str, value: &str| {
            if value.trim().is_empty() {
                Err(TrainerError::Config(format!(
                    "scheduled event {:?}: {what} must be non-empty",
                    self.label
                )))
            } else {
                Ok(())
            }
        };
        match &self.action {
            ScheduledAction::SetMappingPsc {
                src,
                dst,
                psc_multiplier,
            } => {
                named("src", src)?;
                named("dst", dst)?;
                if !psc_multiplier.is_finite() || *psc_multiplier < 0.0 {
                    return Err(TrainerError::Config(format!(
                        "scheduled event {:?}: psc_multiplier must be finite and >= 0",
                        self.label
                    )));
                }
            }
            ScheduledAction::SetAreaParameter { area, key, .. } => {
                named("area", area)?;
                named("key", key)?;
            }
        }
        Ok(())
    }
}

/// Callback that applies one scheduled event to the brain.
pub type ApplyEvent<'a> = &'a mut (dyn FnMut(&ScheduledAction) -> Result<(), TrainerError> + 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn event_serializes_with_the_documented_shape() {
        let event = ScheduledEvent {
            at_s: 1.5,
            label: "step".to_string(),
            action: ScheduledAction::SetMappingPsc {
                src: "a".to_string(),
                dst: "b".to_string(),
                psc_multiplier: 0.5,
            },
        };
        assert_eq!(
            serde_json::to_value(&event).unwrap(),
            json!({
                "at_s": 1.5,
                "label": "step",
                "action": {"type": "set_mapping_psc", "src": "a", "dst": "b", "psc_multiplier": 0.5}
            })
        );
        let area = ScheduledAction::SetAreaParameter {
            area: "x".to_string(),
            key: "k".to_string(),
            value: json!(2),
        };
        assert_eq!(
            serde_json::to_value(&area).unwrap(),
            json!({"type": "set_area_parameter", "area": "x", "key": "k", "value": 2})
        );
    }

    #[test]
    fn camel_case_aliases_still_deserialize() {
        let event: ScheduledEvent = serde_json::from_value(json!({
            "atS": 2.0,
            "label": "l",
            "action": {"type": "set_mapping_psc", "src": "a", "dst": "b", "pscMultiplier": 3.0}
        }))
        .unwrap();
        assert_eq!(event.at_s, 2.0);
        assert!(event.validate().is_ok());
    }

    #[test]
    fn malformed_events_are_rejected() {
        let mut event = ScheduledEvent {
            at_s: -1.0,
            label: "l".to_string(),
            action: ScheduledAction::SetAreaParameter {
                area: "a".to_string(),
                key: "k".to_string(),
                value: json!(1),
            },
        };
        assert!(event.validate().is_err());
        event.at_s = 0.0;
        event.label = " ".to_string();
        assert!(event.validate().is_err());
    }
}
