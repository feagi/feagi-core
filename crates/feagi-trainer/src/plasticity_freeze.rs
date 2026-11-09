//! Learning freeze for brain-only evaluation phases.
//!
//! FEAGI keeps learning-freeze switches as brain-instance runtime state (never in the genome):
//! `PUT /v1/cortical_mapping/plasticity_frozen` per mapping, `PUT /v1/cortical_area/learning_frozen`
//! per memory area, and `GET /v1/connectome/learning_frozen` for every switch that is on.
//! [`LearningFreeze`] reads the prior switches, freezes every plastic mapping and every memory
//! area, and on [`PlasticityControl::restore`] (or drop) switches back exactly what it froze, so
//! a switch that was already on before the phase stays on. No rule or genome is rewritten, and
//! learned weights and memory neurons are kept.

use std::collections::BTreeSet;

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
    /// Plastic mappings that are frozen for the phase (including any frozen beforehand).
    pub mappings_frozen: u32,
    /// Memory areas that are frozen for the phase (including any frozen beforehand).
    pub memory_areas_frozen: u32,
    /// Memory-neuron formation is paused (every memory area is frozen).
    pub memory_formation_paused: bool,
}

/// Every learning-freeze switch that is on, as FEAGI reports it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LearningFreezeState {
    /// Memory areas with learning frozen.
    pub memory_areas: BTreeSet<String>,
    /// `(src, dst)` mappings with plasticity frozen.
    pub mappings: BTreeSet<(String, String)>,
}

/// The FEAGI routes the learning freeze needs.
pub trait FeagiLearningApi {
    /// Every switch that is on now (`GET /v1/connectome/learning_frozen`).
    fn frozen_state(&mut self) -> Result<LearningFreezeState, TrainerError>;

    /// `(src, dst)` of every mapping that has at least one plastic rule.
    fn plastic_mappings(&mut self) -> Result<Vec<(String, String)>, TrainerError>;

    /// Every memory area.
    fn memory_areas(&mut self) -> Result<Vec<String>, TrainerError>;

    /// Switch plasticity of `src -> dst` off (`true`) or on (`false`).
    fn set_mapping_frozen(
        &mut self,
        src: &str,
        dst: &str,
        frozen: bool,
    ) -> Result<(), TrainerError>;

    /// Switch learning of a memory area off (`true`) or on (`false`).
    fn set_memory_area_frozen(&mut self, area: &str, frozen: bool) -> Result<(), TrainerError>;
}

/// [`PlasticityControl`] that freezes every plastic mapping and memory area through a
/// [`FeagiLearningApi`] and restores only the switches it turned on.
pub struct LearningFreeze<A: FeagiLearningApi> {
    api: A,
    froze_mappings: Vec<(String, String)>,
    froze_areas: Vec<String>,
}

impl<A: FeagiLearningApi> LearningFreeze<A> {
    /// Control over `api` with nothing frozen.
    pub fn new(api: A) -> Self {
        Self {
            api,
            froze_mappings: Vec::new(),
            froze_areas: Vec::new(),
        }
    }

    /// The wrapped API.
    pub fn api(&self) -> &A {
        &self.api
    }

    /// Switches currently turned on by this control (mappings plus memory areas).
    pub fn frozen_count(&self) -> usize {
        self.froze_mappings.len() + self.froze_areas.len()
    }
}

impl<A: FeagiLearningApi> PlasticityControl for LearningFreeze<A> {
    fn freeze(&mut self) -> Result<PlasticityFreezeOutcome, TrainerError> {
        if self.frozen_count() > 0 {
            return Err(TrainerError::Runtime(
                "learning is already frozen by this control".to_string(),
            ));
        }
        let result = (|| -> Result<PlasticityFreezeOutcome, TrainerError> {
            let prior = self.api.frozen_state()?;
            let mappings = self.api.plastic_mappings()?;
            let areas = self.api.memory_areas()?;
            for (src, dst) in &mappings {
                if prior.mappings.contains(&(src.clone(), dst.clone())) {
                    continue;
                }
                // Remember the switch first, so a failed call is still rolled back.
                self.froze_mappings.push((src.clone(), dst.clone()));
                self.api.set_mapping_frozen(src, dst, true)?;
            }
            for area in &areas {
                if prior.memory_areas.contains(area) {
                    continue;
                }
                self.froze_areas.push(area.clone());
                self.api.set_memory_area_frozen(area, true)?;
            }
            Ok(PlasticityFreezeOutcome {
                mappings_frozen: mappings.len() as u32,
                memory_areas_frozen: areas.len() as u32,
                memory_formation_paused: true,
            })
        })();
        match result {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                let note = match self.restore() {
                    Ok(()) => String::new(),
                    Err(restore) => format!("; rollback also failed: {restore}"),
                };
                Err(TrainerError::Runtime(format!(
                    "learning freeze failed: {error}{note}"
                )))
            }
        }
    }

    fn restore(&mut self) -> Result<(), TrainerError> {
        let mut first_error: Option<String> = None;
        let mut failed_mappings = Vec::new();
        for (src, dst) in std::mem::take(&mut self.froze_mappings) {
            if let Err(error) = self.api.set_mapping_frozen(&src, &dst, false) {
                first_error.get_or_insert(format!("{src} -> {dst}: {error}"));
                failed_mappings.push((src, dst));
            }
        }
        let mut failed_areas = Vec::new();
        for area in std::mem::take(&mut self.froze_areas) {
            if let Err(error) = self.api.set_memory_area_frozen(&area, false) {
                first_error.get_or_insert(format!("memory area {area}: {error}"));
                failed_areas.push(area);
            }
        }
        // Switches that could not be turned back stay listed, so a later restore retries them.
        self.froze_mappings = failed_mappings;
        self.froze_areas = failed_areas;
        match first_error {
            None => Ok(()),
            Some(message) => Err(TrainerError::Runtime(format!(
                "learning restore failed for {message}"
            ))),
        }
    }
}

impl<A: FeagiLearningApi> Drop for LearningFreeze<A> {
    /// Last-resort restore when the control is dropped while still frozen.
    fn drop(&mut self) {
        if self.frozen_count() > 0 {
            let _ = self.restore();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// In-memory FEAGI: the switches, the plastic mappings and memory areas, and a call log.
    #[derive(Default)]
    pub struct MockFeagi {
        pub state: LearningFreezeState,
        pub plastic: Vec<(String, String)>,
        pub areas: Vec<String>,
        /// Number of the `set_*` call (1-based) that fails, if any.
        pub fail_set_number: Option<usize>,
        pub set_calls: usize,
        /// Every request issued: `map src dst bool` or `mem area bool`.
        pub log: Vec<String>,
    }

    fn pair(a: &str, b: &str) -> (String, String) {
        (a.to_string(), b.to_string())
    }

    impl MockFeagi {
        fn standard() -> Self {
            Self {
                plastic: vec![pair("a", "b"), pair("c", "d"), pair("e", "f")],
                areas: vec!["m1".to_string(), "m2".to_string()],
                ..Self::default()
            }
        }
        fn with_prior(mut self) -> Self {
            self.state.mappings.insert(pair("c", "d"));
            self.state.memory_areas.insert("m2".to_string());
            self
        }
        fn tick(&mut self) -> Result<(), TrainerError> {
            self.set_calls += 1;
            if self.fail_set_number == Some(self.set_calls) {
                return Err(TrainerError::Runtime("set refused".to_string()));
            }
            Ok(())
        }
    }

    impl FeagiLearningApi for MockFeagi {
        fn frozen_state(&mut self) -> Result<LearningFreezeState, TrainerError> {
            Ok(self.state.clone())
        }
        fn plastic_mappings(&mut self) -> Result<Vec<(String, String)>, TrainerError> {
            Ok(self.plastic.clone())
        }
        fn memory_areas(&mut self) -> Result<Vec<String>, TrainerError> {
            Ok(self.areas.clone())
        }
        fn set_mapping_frozen(
            &mut self,
            src: &str,
            dst: &str,
            frozen: bool,
        ) -> Result<(), TrainerError> {
            self.log.push(format!("map {src} {dst} {frozen}"));
            self.tick()?;
            let key = pair(src, dst);
            if frozen {
                self.state.mappings.insert(key);
            } else {
                self.state.mappings.remove(&key);
            }
            Ok(())
        }
        fn set_memory_area_frozen(&mut self, area: &str, frozen: bool) -> Result<(), TrainerError> {
            self.log.push(format!("mem {area} {frozen}"));
            self.tick()?;
            if frozen {
                self.state.memory_areas.insert(area.to_string());
            } else {
                self.state.memory_areas.remove(area);
            }
            Ok(())
        }
    }

    fn all_frozen(state: &LearningFreezeState, mock: &MockFeagi) -> bool {
        mock.plastic.iter().all(|m| state.mappings.contains(m))
            && mock.areas.iter().all(|a| state.memory_areas.contains(a))
    }

    #[test]
    fn freeze_covers_every_plastic_mapping_and_memory_area_and_restore_keeps_the_prior_state() {
        let mock = MockFeagi::standard().with_prior();
        let prior = mock.state.clone();
        let mut control = LearningFreeze::new(mock);
        let outcome = control.freeze().expect("freeze");
        assert_eq!(outcome.mappings_frozen, 3);
        assert_eq!(outcome.memory_areas_frozen, 2);
        assert!(outcome.memory_formation_paused);
        assert!(all_frozen(&control.api().state, control.api()));
        assert_eq!(control.frozen_count(), 3, "only the switches it turned on");
        control.restore().expect("restore");
        assert_eq!(control.api().state, prior, "exactly the prior state");
        control.restore().expect("a second restore is a no-op");
        assert!(control
            .api()
            .log
            .iter()
            .all(|l| !l.contains("c d false") && !l.contains("mem m2 false")));
    }

    #[test]
    fn a_failed_freeze_rolls_back_what_it_already_froze() {
        let mut mock = MockFeagi::standard().with_prior();
        mock.fail_set_number = Some(3); // the memory-area switch, after both mappings
        let prior = mock.state.clone();
        let mut control = LearningFreeze::new(mock);
        let error = control.freeze().unwrap_err().to_string();
        assert!(error.contains("learning freeze failed"), "{error}");
        assert_eq!(control.api().state, prior, "back to the prior state");
        assert_eq!(control.frozen_count(), 0);
    }

    /// A mock shared with the test, so its state can be read after the control is dropped.
    struct Shared(std::rc::Rc<std::cell::RefCell<MockFeagi>>);

    impl FeagiLearningApi for Shared {
        fn frozen_state(&mut self) -> Result<LearningFreezeState, TrainerError> {
            self.0.borrow_mut().frozen_state()
        }
        fn plastic_mappings(&mut self) -> Result<Vec<(String, String)>, TrainerError> {
            self.0.borrow_mut().plastic_mappings()
        }
        fn memory_areas(&mut self) -> Result<Vec<String>, TrainerError> {
            self.0.borrow_mut().memory_areas()
        }
        fn set_mapping_frozen(&mut self, s: &str, d: &str, f: bool) -> Result<(), TrainerError> {
            self.0.borrow_mut().set_mapping_frozen(s, d, f)
        }
        fn set_memory_area_frozen(&mut self, a: &str, f: bool) -> Result<(), TrainerError> {
            self.0.borrow_mut().set_memory_area_frozen(a, f)
        }
    }

    #[test]
    fn dropping_a_frozen_control_restores_the_prior_state() {
        let mock = std::rc::Rc::new(std::cell::RefCell::new(MockFeagi::standard().with_prior()));
        let prior = mock.borrow().state.clone();
        let mut control = LearningFreeze::new(Shared(mock.clone()));
        control.freeze().expect("freeze");
        assert_ne!(mock.borrow().state, prior);
        drop(control);
        assert_eq!(mock.borrow().state, prior);
    }

    #[test]
    fn a_failed_restore_keeps_the_switch_for_a_retry() {
        let mut control = LearningFreeze::new(MockFeagi::standard());
        control.freeze().expect("freeze");
        let sets = control.api().set_calls;
        control.api.fail_set_number = Some(sets + 1);
        assert!(control.restore().is_err());
        assert_eq!(control.frozen_count(), 1, "the refused switch is kept");
        control.restore().expect("retry");
        assert!(control.api().state.mappings.is_empty());
        assert!(control.api().state.memory_areas.is_empty());
    }

    #[test]
    fn freezing_twice_is_refused() {
        let mut control = LearningFreeze::new(MockFeagi::standard());
        control.freeze().expect("freeze");
        assert!(control.freeze().is_err());
    }
}
