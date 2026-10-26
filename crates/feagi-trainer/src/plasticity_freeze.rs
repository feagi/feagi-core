//! Plasticity freeze for brain-only evaluation phases.
//!
//! FEAGI has no global plasticity switch: `/v1/neuroplasticity/{enable,disable}/{area}`
//! only log, and `global_plasticity_enabled` in `/v1/neuroplasticity/status` is a constant.
//! The one working freeze is per mapping: rewrite the stored rules of a mapping with
//! `plasticity_flag` off (`PUT /v1/cortical_mapping/mapping_properties`), which keeps the
//! synapses and their weights. [`MappingPlasticityFreeze`] does that for every mapping that
//! has a plastic rule and writes the original rules back on [`PlasticityControl::restore`].
//!
//! Memory-neuron formation has no pause at all; [`PlasticityFreezeOutcome`] reports it as
//! not paused.

use serde_json::Value;

use crate::error::TrainerError;

/// Freeze/restore of learning around a phase. Implemented over the FEAGI REST API by the app
/// and by mocks in tests.
pub trait PlasticityControl {
    /// Stop learning. On `Err` nothing may stay frozen.
    fn freeze(&mut self) -> Result<PlasticityFreezeOutcome, TrainerError>;

    /// Undo [`PlasticityControl::freeze`]. A no-op when nothing is frozen.
    fn restore(&mut self) -> Result<(), TrainerError>;
}

/// What a successful freeze did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlasticityFreezeOutcome {
    /// Mappings whose plastic rules were switched off.
    pub mappings_frozen: u32,
    /// Memory-neuron formation was paused. No FEAGI route does this, so it is `false`.
    pub memory_formation_paused: bool,
}

/// The FEAGI mapping routes the freeze needs.
pub trait FeagiMappingApi {
    /// `(src, dst)` of every mapping that has at least one rule with `plasticity_flag` on.
    fn plastic_mappings(&mut self) -> Result<Vec<(String, String)>, TrainerError>;

    /// Stored rules of `src -> dst`, exactly as FEAGI keeps them.
    fn read_rules(&mut self, src: &str, dst: &str) -> Result<Vec<Value>, TrainerError>;

    /// Replace the rules of `src -> dst`.
    fn write_rules(&mut self, src: &str, dst: &str, rules: &[Value]) -> Result<(), TrainerError>;
}

struct SavedMapping {
    src: String,
    dst: String,
    original: Vec<Value>,
}

/// [`PlasticityControl`] that freezes every plastic mapping through a [`FeagiMappingApi`].
pub struct MappingPlasticityFreeze<A: FeagiMappingApi> {
    api: A,
    saved: Vec<SavedMapping>,
}

impl<A: FeagiMappingApi> MappingPlasticityFreeze<A> {
    /// Control over `api` with nothing frozen.
    pub fn new(api: A) -> Self {
        Self {
            api,
            saved: Vec::new(),
        }
    }

    /// The wrapped API.
    pub fn api(&self) -> &A {
        &self.api
    }

    /// Mappings currently frozen by this control.
    pub fn frozen_count(&self) -> usize {
        self.saved.len()
    }
}

fn rule_is_plastic(rule: &Value) -> bool {
    rule.get("plasticity_flag")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn with_plasticity_off(rules: &[Value]) -> Vec<Value> {
    rules
        .iter()
        .map(|rule| {
            let mut rule = rule.clone();
            if let Some(object) = rule.as_object_mut() {
                object.insert("plasticity_flag".to_string(), Value::Bool(false));
            }
            rule
        })
        .collect()
}

impl<A: FeagiMappingApi> PlasticityControl for MappingPlasticityFreeze<A> {
    fn freeze(&mut self) -> Result<PlasticityFreezeOutcome, TrainerError> {
        if !self.saved.is_empty() {
            return Err(TrainerError::Runtime(
                "plasticity is already frozen by this control".to_string(),
            ));
        }
        let result = (|| -> Result<(), TrainerError> {
            for (src, dst) in self.api.plastic_mappings()? {
                let original = self.api.read_rules(&src, &dst)?;
                if !original.iter().any(rule_is_plastic) {
                    continue;
                }
                let frozen = with_plasticity_off(&original);
                // Remember the mapping first, so a failed write is still restored.
                self.saved.push(SavedMapping {
                    src: src.clone(),
                    dst: dst.clone(),
                    original,
                });
                self.api.write_rules(&src, &dst, &frozen)?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => Ok(PlasticityFreezeOutcome {
                mappings_frozen: self.saved.len() as u32,
                memory_formation_paused: false,
            }),
            Err(error) => {
                let note = match self.restore() {
                    Ok(()) => String::new(),
                    Err(restore) => format!("; rollback also failed: {restore}"),
                };
                Err(TrainerError::Runtime(format!(
                    "plasticity freeze failed: {error}{note}"
                )))
            }
        }
    }

    fn restore(&mut self) -> Result<(), TrainerError> {
        let mut first_error: Option<String> = None;
        for mapping in std::mem::take(&mut self.saved) {
            if let Err(error) = self
                .api
                .write_rules(&mapping.src, &mapping.dst, &mapping.original)
            {
                first_error.get_or_insert(format!("{} -> {}: {error}", mapping.src, mapping.dst));
            }
        }
        match first_error {
            None => Ok(()),
            Some(message) => Err(TrainerError::Runtime(format!(
                "plasticity restore failed for {message}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    /// In-memory FEAGI: `(src, dst) -> rules`, with a write log and an optional failing write.
    #[derive(Default)]
    pub struct MockFeagi {
        pub mappings: BTreeMap<(String, String), Vec<Value>>,
        pub writes: Vec<((String, String), Vec<Value>)>,
        pub fail_write_number: Option<usize>,
        pub write_calls: usize,
    }

    impl MockFeagi {
        fn with(rules: &[(&str, &str, bool)]) -> Self {
            let mut mock = Self::default();
            for (src, dst, plastic) in rules {
                mock.mappings.insert(
                    (src.to_string(), dst.to_string()),
                    vec![json!({"morphology_id": "m", "plasticity_flag": plastic, "plasticity_mode": "stdp"})],
                );
            }
            mock
        }
    }

    impl FeagiMappingApi for MockFeagi {
        fn plastic_mappings(&mut self) -> Result<Vec<(String, String)>, TrainerError> {
            Ok(self.mappings.keys().cloned().collect())
        }

        fn read_rules(&mut self, src: &str, dst: &str) -> Result<Vec<Value>, TrainerError> {
            Ok(self.mappings[&(src.to_string(), dst.to_string())].clone())
        }

        fn write_rules(
            &mut self,
            src: &str,
            dst: &str,
            rules: &[Value],
        ) -> Result<(), TrainerError> {
            self.write_calls += 1;
            if self.fail_write_number == Some(self.write_calls) {
                return Err(TrainerError::Runtime("write refused".to_string()));
            }
            self.writes
                .push(((src.to_string(), dst.to_string()), rules.to_vec()));
            self.mappings
                .insert((src.to_string(), dst.to_string()), rules.to_vec());
            Ok(())
        }
    }

    #[test]
    fn freeze_turns_plastic_rules_off_and_restore_writes_the_originals_back() {
        let mut control = MappingPlasticityFreeze::new(MockFeagi::with(&[
            ("a", "b", true),
            ("b", "c", false),
            ("c", "d", true),
        ]));
        let before = control.api().mappings.clone();
        let outcome = control.freeze().expect("freeze");
        assert_eq!(
            outcome.mappings_frozen, 2,
            "the non-plastic mapping is left alone"
        );
        assert!(!outcome.memory_formation_paused);
        assert!(control
            .api()
            .mappings
            .values()
            .all(|rules| rules.iter().all(|rule| !rule_is_plastic(rule))));
        assert_eq!(control.api().writes.len(), 2);
        control.restore().expect("restore");
        assert_eq!(
            control.api().mappings,
            before,
            "originals restored verbatim"
        );
        assert_eq!(control.frozen_count(), 0);
        control.restore().expect("a second restore is a no-op");
        assert_eq!(control.api().writes.len(), 4);
    }

    #[test]
    fn a_failed_freeze_rolls_back_what_it_already_froze() {
        let mut mock = MockFeagi::with(&[("a", "b", true), ("c", "d", true)]);
        mock.fail_write_number = Some(2);
        let before = mock.mappings.clone();
        let mut control = MappingPlasticityFreeze::new(mock);
        let error = control.freeze().unwrap_err().to_string();
        assert!(error.contains("plasticity freeze failed"), "{error}");
        assert_eq!(control.api().mappings, before, "nothing stays frozen");
        assert_eq!(control.frozen_count(), 0);
    }
}
