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
    /// Force-fire `voxels` of an area on every FEAGI burst for `bursts` bursts, or for `seconds`
    /// of wall-clock time converted with the burst interval. Exactly one of the two is given.
    /// Nothing is stored, so there is nothing to restore; a hold in progress is stopped when the
    /// next attempt starts ([`ScheduledAction::AttemptStart`]) and when the phase ends.
    StimulateArea {
        area: String,
        voxels: Vec<[u32; 3]>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bursts: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        seconds: Option<f64>,
        /// Hold form: fire the voxels on every burst until the attempt or phase ends. Replaces
        /// `bursts` and `seconds`, which must then be absent.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        hold: bool,
    },
    /// A timed push on the body, sized as a fraction of its weight. Only the balance-on-feet
    /// runner applies it (it measures the body); every other applier rejects it. The N or N*m
    /// used is computed at run time from the model and written to the run's event log.
    BodyPush {
        /// Push as a force or as a torque about the body's horizontal axes.
        kind: BodyPushKind,
        /// Direction the TOP of the body tips toward, in the body's heading frame.
        axis: BodyPushAxis,
        /// Force: fraction of the body weight (m g). Torque: fraction of weight times the torso's
        /// reset height (m g h). Finite and > 0.
        #[serde(alias = "magnitudeFraction")]
        magnitude_fraction: f64,
        /// Seconds the push lasts. Finite and > 0.
        #[serde(alias = "durationS")]
        duration_s: f64,
    },
    /// A sinusoidal push on the body for `cycles` whole cycles at `frequency_hz`, with a one-cycle
    /// ramp in and out, sized like [`ScheduledAction::BodyPush`]. Only the balance-on-feet runner
    /// applies it (the controller computes the wave per physics step); every other applier rejects
    /// it. Each segment writes a row to the `vibration` table.
    BodyVibration {
        /// Drive as a force or as a torque about the body's horizontal axes.
        kind: BodyPushKind,
        /// Direction the TOP of the body tips toward on the positive half-wave, in the body's
        /// heading frame.
        axis: BodyPushAxis,
        /// Amplitude. Force: fraction of the body weight (m g). Torque: fraction of weight times
        /// the torso's reset height (m g h). Finite and > 0.
        #[serde(alias = "magnitudeFraction")]
        magnitude_fraction: f64,
        /// Drive frequency in hertz. Finite and > 0, and at most a tenth of the sample rate.
        #[serde(alias = "frequencyHz")]
        frequency_hz: f64,
        /// Whole drive cycles, ramps included. At least 4.
        cycles: u32,
    },
    /// Sent by the runner (never part of a schedule) when an attempt begins, so the applier can
    /// end anything the previous attempt left running. Not serializable.
    #[serde(skip)]
    AttemptStart,
}

/// Whether a [`ScheduledAction::BodyPush`] is a force or a torque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyPushKind {
    Force,
    Torque,
}

/// Direction a [`ScheduledAction::BodyPush`] tips (or pushes) the body, in its heading frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyPushAxis {
    Forward,
    Back,
    Left,
    Right,
}

impl BodyPushAxis {
    /// Unit horizontal vector (x, y) in the world frame for a body heading `yaw_rad` (from +X
    /// toward +Y, z up).
    pub fn world_xy(self, yaw_rad: f64) -> [f64; 2] {
        let (sin, cos) = yaw_rad.sin_cos();
        let forward = [cos, sin];
        let left = [-sin, cos];
        match self {
            Self::Forward => forward,
            Self::Back => [-forward[0], -forward[1]],
            Self::Left => left,
            Self::Right => [-left[0], -left[1]],
        }
    }
}

/// World-frame `(torque, force)` of a [`ScheduledAction::BodyPush`] and the magnitude in N or
/// N*m. `mass_kg` is the whole-body mass, `gravity` the gravity magnitude, `height_m` the
/// torso's reset height. A force pushes toward the axis; a torque tips the top toward it
/// (`up x direction`, z up).
pub fn body_push_wrench(
    kind: BodyPushKind,
    axis: BodyPushAxis,
    magnitude_fraction: f64,
    yaw_rad: f64,
    mass_kg: f64,
    gravity: f64,
    height_m: f64,
) -> ([f64; 3], [f64; 3], f64) {
    let weight = mass_kg * gravity;
    let [dx, dy] = axis.world_xy(yaw_rad);
    match kind {
        BodyPushKind::Force => {
            let n = magnitude_fraction * weight;
            ([0.0; 3], [dx * n, dy * n, 0.0], n)
        }
        BodyPushKind::Torque => {
            let nm = magnitude_fraction * weight * height_m;
            ([-dy * nm, dx * nm, 0.0], [0.0; 3], nm)
        }
    }
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
                if !psc_multiplier.is_finite() {
                    return Err(TrainerError::Config(format!(
                        "scheduled event {:?}: psc_multiplier must be finite (negative values are allowed for inhibitory mappings)",
                        self.label
                    )));
                }
            }
            ScheduledAction::SetAreaParameter { area, key, .. } => {
                named("area", area)?;
                named("key", key)?;
            }
            ScheduledAction::StimulateArea {
                area,
                voxels,
                bursts,
                seconds,
                hold,
            } => {
                named("area", area)?;
                if voxels.is_empty() {
                    return Err(TrainerError::Config(format!(
                        "scheduled event {:?}: voxels must list at least one [x, y, z]",
                        self.label
                    )));
                }
                match (bursts, seconds, hold) {
                    (None, None, true) => {}
                    (Some(b), None, false) if *b >= 1 => {}
                    (None, Some(s), false) if s.is_finite() && *s > 0.0 => {}
                    _ => {
                        return Err(TrainerError::Config(format!(
                            "scheduled event {:?}: give exactly one of bursts (>= 1), seconds (finite, > 0) or hold: true",
                            self.label
                        )));
                    }
                }
            }
            ScheduledAction::BodyPush {
                magnitude_fraction,
                duration_s,
                ..
            } => {
                if !magnitude_fraction.is_finite() || *magnitude_fraction <= 0.0 {
                    return Err(TrainerError::Config(format!(
                        "scheduled event {:?}: magnitude_fraction must be finite and > 0",
                        self.label
                    )));
                }
                if !duration_s.is_finite() || *duration_s <= 0.0 {
                    return Err(TrainerError::Config(format!(
                        "scheduled event {:?}: duration_s must be finite and > 0",
                        self.label
                    )));
                }
            }
            ScheduledAction::BodyVibration {
                magnitude_fraction,
                frequency_hz,
                cycles,
                ..
            } => {
                if !magnitude_fraction.is_finite() || *magnitude_fraction <= 0.0 {
                    return Err(TrainerError::Config(format!(
                        "scheduled event {:?}: magnitude_fraction must be finite and > 0",
                        self.label
                    )));
                }
                // The sample rate is known when the phase starts, which checks the upper limit.
                crate::vibration_tracker::check_segment(&self.label, *frequency_hz, *cycles, 0.0)
                    .map_err(TrainerError::Config)?;
            }
            ScheduledAction::AttemptStart => {
                return Err(TrainerError::Config(format!(
                    "scheduled event {:?}: attempt_start is sent by the runner, not scheduled",
                    self.label
                )));
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
    fn body_vibration_serializes_and_validates() {
        let action: ScheduledAction = serde_json::from_value(json!({
            "type": "body_vibration", "kind": "torque", "axis": "forward",
            "magnitude_fraction": 0.01, "frequency_hz": 1.0, "cycles": 10
        }))
        .unwrap();
        let mut event = ScheduledEvent {
            at_s: 5.0,
            label: "shake".to_string(),
            action,
        };
        assert!(event.validate().is_ok());
        assert_eq!(
            serde_json::to_value(&event.action).unwrap(),
            json!({"type": "body_vibration", "kind": "torque", "axis": "forward", "magnitude_fraction": 0.01, "frequency_hz": 1.0, "cycles": 10})
        );
        if let ScheduledAction::BodyVibration { cycles, .. } = &mut event.action {
            *cycles = 3;
        }
        assert!(event.validate().is_err());
    }

    #[test]
    fn body_push_serializes_validates_and_sizes_from_weight() {
        let push = ScheduledAction::BodyPush {
            kind: BodyPushKind::Torque,
            axis: BodyPushAxis::Forward,
            magnitude_fraction: 0.05,
            duration_s: 0.1,
        };
        assert_eq!(
            serde_json::to_value(&push).unwrap(),
            json!({"type": "body_push", "kind": "torque", "axis": "forward", "magnitude_fraction": 0.05, "duration_s": 0.1})
        );
        let mut event = ScheduledEvent {
            at_s: 1.0,
            label: "p".to_string(),
            action: push,
        };
        assert!(event.validate().is_ok());
        if let ScheduledAction::BodyPush { magnitude_fraction, .. } = &mut event.action {
            *magnitude_fraction = 0.0;
        }
        assert!(event.validate().is_err());
        // m = 2 kg, g = 10, h = 0.5 m, heading along +X.
        let (torque, force, nm) =
            body_push_wrench(BodyPushKind::Torque, BodyPushAxis::Forward, 0.1, 0.0, 2.0, 10.0, 0.5);
        assert!((nm - 0.1 * 2.0 * 10.0 * 0.5).abs() < 1e-12);
        assert_eq!(force, [0.0; 3]);
        // Forward = +X: the top tips toward +X, so the torque is about +Y (up x forward).
        assert!(torque[0].abs() < 1e-12 && (torque[1] - nm).abs() < 1e-12 && torque[2] == 0.0);
        // Heading +Y: forward is +Y, torque about -X.
        let (torque, _, _) = body_push_wrench(
            BodyPushKind::Torque,
            BodyPushAxis::Forward,
            0.1,
            std::f64::consts::FRAC_PI_2,
            2.0,
            10.0,
            0.5,
        );
        assert!((torque[0] + nm).abs() < 1e-12 && torque[1].abs() < 1e-12);
        let (_, force, n) =
            body_push_wrench(BodyPushKind::Force, BodyPushAxis::Left, 0.25, 0.0, 2.0, 10.0, 0.5);
        assert!((n - 5.0).abs() < 1e-12 && (force[1] - 5.0).abs() < 1e-12);
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

    #[test]
    fn negative_psc_multiplier_is_accepted_and_non_finite_rejected() {
        let psc = |value: f64| ScheduledEvent {
            at_s: 0.0,
            label: "l".to_string(),
            action: ScheduledAction::SetMappingPsc {
                src: "a".to_string(),
                dst: "b".to_string(),
                psc_multiplier: value,
            },
        };
        for ok in [-2.5, -0.0, 0.0, 3.0] {
            assert!(psc(ok).validate().is_ok(), "{ok}");
        }
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(psc(bad).validate().is_err(), "{bad}");
        }
    }

    #[test]
    fn stimulate_area_has_the_documented_shape_and_is_validated() {
        let event: ScheduledEvent = serde_json::from_value(json!({
            "at_s": 0.5,
            "label": "drive",
            "action": {"type": "stimulate_area", "area": "src", "voxels": [[0, 0, 0], [1, 0, 2]], "bursts": 30}
        }))
        .unwrap();
        assert!(event.validate().is_ok());
        assert_eq!(
            serde_json::to_value(&event.action).unwrap(),
            json!({"type": "stimulate_area", "area": "src", "voxels": [[0, 0, 0], [1, 0, 2]], "bursts": 30})
        );
        let with = |action: serde_json::Value| -> ScheduledEvent {
            serde_json::from_value(json!({"at_s": 0.0, "label": "l", "action": action})).unwrap()
        };
        let stim = |extra: serde_json::Value| {
            let mut action = json!({"type": "stimulate_area", "area": "a", "voxels": [[0, 0, 0]]});
            action
                .as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            with(action)
        };
        assert!(stim(json!({"seconds": 0.2})).validate().is_ok());
        assert!(stim(json!({"hold": true})).validate().is_ok());
        for bad in [
            json!({}),
            json!({"bursts": 0}),
            json!({"bursts": 3, "seconds": 1.0}),
            json!({"seconds": 0.0}),
            json!({"seconds": -1.0}),
            json!({"hold": true, "bursts": 3}),
            json!({"hold": false}),
        ] {
            assert!(stim(bad.clone()).validate().is_err(), "{bad}");
        }
        let mut empty = stim(json!({"bursts": 3}));
        if let ScheduledAction::StimulateArea { voxels, .. } = &mut empty.action {
            voxels.clear();
        }
        assert!(empty.validate().is_err());
        assert!(
            serde_json::from_value::<ScheduledAction>(json!({"type": "attempt_start"})).is_err()
        );
    }
}
