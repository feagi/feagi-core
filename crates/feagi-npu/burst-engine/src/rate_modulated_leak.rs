// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! @cursor:critical-path — per-cortical, opt-in homeostatic `leak_coefficient` (see `neural/docs/rate_modulated_leak.md`).

use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MEMORY_NEURON_ID_START: u32 = 50_000_000;

/// @cursor:critical-path — not invoked in hot LIF; runs only when registered areas are non-empty.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateModulatedLeakConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Target in [0,1]: expected fraction of bursts in which a neuron fires.
    #[serde(default = "default_target")]
    pub target_firing_per_burst: f32,
    #[serde(default = "default_tau_r")]
    pub rate_ema_tau_bursts: f32,
    #[serde(default = "default_gain")]
    pub gain: f32,
    #[serde(default = "default_leak_min")]
    pub leak_min: f32,
    #[serde(default = "default_leak_max")]
    pub leak_max: f32,
    /// If 0, treated as 1.
    #[serde(default = "one_u32")]
    pub update_every_n_bursts: u32,
}

fn default_target() -> f32 {
    0.1
}
fn default_tau_r() -> f32 {
    50.0
}
fn default_gain() -> f32 {
    0.2
}
fn default_leak_min() -> f32 {
    0.02
}
fn default_leak_max() -> f32 {
    0.5
}
fn one_u32() -> u32 {
    1
}

#[derive(Debug)]
struct AreaState {
    config: RateModulatedLeakConfig,
    neuron_idx: Vec<usize>,
    r_ema: Vec<f32>,
    g_leak: Vec<f32>,
}

/// Registry of areas that receive the homeostatic post-pass. Empty: zero overhead in [`crate::npu::RustNPU`].
#[derive(Debug, Default)]
pub struct RateModulatedLeakRegistry {
    areas: AHashMap<u32, AreaState>,
}

impl RateModulatedLeakConfig {
    /// Validate an enabled config. `Ok(None)` when disabled; `Err` names the offending field.
    pub fn validate(value: &Value) -> Result<Option<Self>, String> {
        let obj = value
            .as_object()
            .ok_or_else(|| "rate_modulated_leak must be a JSON object".to_string())?;
        match obj.get("enabled") {
            None => return Ok(None),
            Some(Value::Bool(false)) => return Ok(None),
            Some(Value::Bool(true)) => {}
            Some(_) => return Err("rate_modulated_leak.enabled must be a boolean".to_string()),
        }
        let cfg: Self = serde_json::from_value(value.clone())
            .map_err(|e| format!("rate_modulated_leak has invalid field types: {e}"))?;
        if !cfg.rate_ema_tau_bursts.is_finite() || cfg.rate_ema_tau_bursts <= 0.0 {
            return Err(format!(
                "rate_modulated_leak.rate_ema_tau_bursts must be finite and > 0 (got {})",
                cfg.rate_ema_tau_bursts
            ));
        }
        if !cfg.gain.is_finite() || cfg.gain < 0.0 {
            return Err(format!(
                "rate_modulated_leak.gain must be finite and >= 0 (got {})",
                cfg.gain
            ));
        }
        if !(0.0..=1.0).contains(&cfg.target_firing_per_burst) {
            return Err(format!(
                "rate_modulated_leak.target_firing_per_burst must be in [0, 1] (got {})",
                cfg.target_firing_per_burst
            ));
        }
        if !(0.0..=1.0).contains(&cfg.leak_min) {
            return Err(format!(
                "rate_modulated_leak.leak_min must be finite and in [0, 1] (got {})",
                cfg.leak_min
            ));
        }
        if !(0.0..=1.0).contains(&cfg.leak_max) {
            return Err(format!(
                "rate_modulated_leak.leak_max must be finite and in [0, 1] (got {})",
                cfg.leak_max
            ));
        }
        if cfg.leak_min > cfg.leak_max {
            return Err(format!(
                "rate_modulated_leak.leak_min ({}) must be <= leak_max ({})",
                cfg.leak_min, cfg.leak_max
            ));
        }
        Ok(Some(cfg))
    }

    /// Parse or return `None` for invalid or disabled configs.
    pub fn parse_from_cortical_property(value: &Value) -> Option<Self> {
        Self::validate(value).ok().flatten()
    }
}

impl RateModulatedLeakRegistry {
    pub fn is_empty(&self) -> bool {
        self.areas.is_empty()
    }

    pub fn contains(&self, cortical_idx: u32) -> bool {
        self.areas.contains_key(&cortical_idx)
    }

    pub fn remove(&mut self, cortical_idx: u32) {
        self.areas.remove(&cortical_idx);
    }

    /// Firing-rate EMA per global neuron index for one area, or `None` when not registered.
    pub fn rate_ema_by_neuron(&self, cortical_idx: u32) -> Option<AHashMap<usize, f32>> {
        self.areas.get(&cortical_idx).map(|st| {
            st.neuron_idx
                .iter()
                .copied()
                .zip(st.r_ema.iter().copied())
                .collect()
        })
    }

    /// Drop every registration.
    pub fn clear(&mut self) {
        self.areas.clear();
    }

    /// Restart the homeostat for an area from `leak` after an explicit leak edit.
    /// Keeps the rate EMA; the next update step moves from this value.
    pub fn reseed_leak(&mut self, cortical_idx: u32, leak: f32) {
        if !leak.is_finite() {
            return;
        }
        if let Some(st) = self.areas.get_mut(&cortical_idx) {
            let g = leak.clamp(0.0, 1.0);
            st.g_leak.iter_mut().for_each(|x| *x = g);
        }
    }

    /// Replace registration for a cortical area, or remove if `value` is not an enabled, valid object.
    pub fn sync_cortical(
        &mut self,
        cortical_idx: u32,
        value: &Value,
        base_leak: f32,
        neuron_global_indices: Vec<usize>,
    ) {
        let Some(cfg) = RateModulatedLeakConfig::parse_from_cortical_property(value) else {
            self.areas.remove(&cortical_idx);
            return;
        };
        if neuron_global_indices.is_empty() {
            self.areas.remove(&cortical_idx);
            return;
        }
        let n = neuron_global_indices.len();
        let g0 = base_leak.clamp(0.0, 1.0);
        self.areas.insert(
            cortical_idx,
            AreaState {
                config: cfg,
                neuron_idx: neuron_global_indices,
                r_ema: vec![0.0; n],
                g_leak: vec![g0; n],
            },
        );
    }

    /// @cursor:critical-path — call after the fire ledger is archived for `burst_count`.
    pub fn apply_burstal(
        &mut self,
        burst_count: u64,
        main_fire_ledger: &crate::fire_ledger::FireLedger,
        storage_leaks: &mut [f32],
    ) {
        if self.areas.is_empty() {
            return;
        }
        for (cortical_idx, st) in self.areas.iter_mut() {
            let n_period = st.config.update_every_n_bursts.max(1) as u64;
            // EMA samples every burst; only the leak step is gated by the cadence.
            let leak_step_due = (burst_count % n_period) == 0;
            let tau_r = st.config.rate_ema_tau_bursts;
            if !tau_r.is_finite() || tau_r <= 0.0 {
                continue;
            }
            let alpha_r: f32 = 1.0 - (-1.0_f32 / tau_r).exp();
            let t = st.config.target_firing_per_burst.clamp(0.0, 1.0);
            let g_min = st.config.leak_min.clamp(0.0, 1.0);
            let g_max = st.config.leak_max.clamp(0.0, 1.0);
            if g_max < g_min {
                continue;
            }
            let gain = st.config.gain;
            if !gain.is_finite() {
                continue;
            }
            let window =
                match main_fire_ledger.get_dense_window_bitmaps(*cortical_idx, burst_count, 1) {
                    Ok(w) => w,
                    Err(_) => continue,
                };
            let first_bm = match window.into_iter().next() {
                Some((_, b)) => b,
                None => continue,
            };
            for (j, &idx) in st.neuron_idx.iter().enumerate() {
                if idx >= storage_leaks.len() {
                    continue;
                }
                let nid: u32 = idx as u32;
                if nid >= MEMORY_NEURON_ID_START {
                    continue;
                }
                let fired: f32 = if first_bm.contains(nid) { 1.0 } else { 0.0 };
                st.r_ema[j] = (1.0 - alpha_r) * st.r_ema[j] + alpha_r * fired;
                if !leak_step_due {
                    continue;
                }
                let e: f32 = st.r_ema[j] - t;
                st.g_leak[j] = (st.g_leak[j] + gain * e).clamp(g_min, g_max);
                let g: f32 = st.g_leak[j].clamp(0.0, 1.0);
                if let Some(slot) = storage_leaks.get_mut(idx) {
                    *slot = g;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fire_ledger::FireLedger;
    use crate::fire_structures::{FireQueue, FiringNeuron, FIRE_KIND_STDP_ELIGIBLE};
    use feagi_npu_neural::types::NeuronId;
    use serde_json::json;

    /// Archive one burst: neuron ids in the bitmap are global storage indices (matches `apply_burstal`).
    fn archive_burst_fires(
        ledger: &mut FireLedger,
        timestep: u64,
        cortical_idx: u32,
        fired_storage_indices: &[u32],
    ) {
        let mut fq = FireQueue::new();
        for &nid in fired_storage_indices {
            fq.add_neuron(FiringNeuron {
                neuron_id: NeuronId(nid),
                membrane_potential: 1.0,
                cortical_idx,
                x: 0,
                y: 0,
                z: 0,
                fire_kind: FIRE_KIND_STDP_ELIGIBLE,
            });
        }
        ledger.archive_burst(timestep, &fq).unwrap();
    }

    fn archive_burst_silent(ledger: &mut FireLedger, timestep: u64) {
        let fq = FireQueue::new();
        ledger.archive_burst(timestep, &fq).unwrap();
    }

    #[test]
    fn parse_from_property_disabled_is_none() {
        let v = json!({ "enabled": false, "target_firing_per_burst": 0.1 });
        assert!(RateModulatedLeakConfig::parse_from_cortical_property(&v).is_none());
    }

    #[test]
    fn parse_from_property_enabled_ok() {
        let v = json!({
            "enabled": true,
            "target_firing_per_burst": 0.2,
            "rate_ema_tau_bursts": 10.0,
            "gain": 0.1,
        });
        let c = RateModulatedLeakConfig::parse_from_cortical_property(&v);
        assert!(c.is_some());
        let c = c.unwrap();
        assert!((c.target_firing_per_burst - 0.2).abs() < 1e-4);
    }

    #[test]
    fn parse_rejects_non_positive_tau() {
        for tau in [0.0_f32, -1.0_f32] {
            let v = json!({
                "enabled": true,
                "rate_ema_tau_bursts": tau,
                "gain": 0.1,
            });
            assert!(
                RateModulatedLeakConfig::parse_from_cortical_property(&v).is_none(),
                "tau={tau} should be rejected"
            );
        }
    }

    #[test]
    fn parse_accepts_defaults_when_enabled() {
        let v = json!({ "enabled": true });
        let c = RateModulatedLeakConfig::parse_from_cortical_property(&v).unwrap();
        assert!(c.enabled);
        assert!((c.target_firing_per_burst - 0.1).abs() < 1e-5);
        assert!((c.rate_ema_tau_bursts - 50.0).abs() < 1e-5);
        assert!((c.gain - 0.2).abs() < 1e-5);
        assert_eq!(c.update_every_n_bursts, 1);
    }

    #[test]
    fn sync_disabled_removes_area_and_registry_empty() {
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            1,
            &json!({ "enabled": true, "rate_ema_tau_bursts": 1.0, "gain": 0.1 }),
            0.1,
            vec![0usize],
        );
        assert!(!reg.is_empty());
        reg.sync_cortical(1, &json!({ "enabled": false }), 0.1, vec![0usize]);
        assert!(reg.is_empty());
    }

    #[test]
    fn sync_empty_neuron_indices_removes_area() {
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            2,
            &json!({ "enabled": true, "rate_ema_tau_bursts": 1.0, "gain": 0.1 }),
            0.1,
            vec![0usize],
        );
        assert!(!reg.is_empty());
        reg.sync_cortical(
            2,
            &json!({ "enabled": true, "rate_ema_tau_bursts": 1.0, "gain": 0.1 }),
            0.1,
            vec![],
        );
        assert!(reg.is_empty());
    }

    #[test]
    fn remove_clears_area() {
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            3,
            &json!({ "enabled": true, "rate_ema_tau_bursts": 1.0, "gain": 0.1 }),
            0.2,
            vec![5usize],
        );
        reg.remove(3);
        assert!(reg.is_empty());
    }

    #[test]
    fn apply_burstal_noop_when_registry_empty() {
        let mut reg = RateModulatedLeakRegistry::default();
        let mut ledger = FireLedger::new(8);
        ledger.track_area(1, 4).unwrap();
        archive_burst_silent(&mut ledger, 1);
        let mut leaks = vec![0.99_f32; 16];
        reg.apply_burstal(1, &ledger, &mut leaks);
        assert!((leaks[0] - 0.99).abs() < 1e-6);
    }

    #[test]
    fn apply_burstal_firing_above_target_increases_leak() {
        let neuron_idx: usize = 7;
        let cortical = 10u32;
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            cortical,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.0,
                "rate_ema_tau_bursts": 1.0,
                "gain": 1.0,
                "leak_min": 0.0,
                "leak_max": 1.0,
            }),
            0.1,
            vec![neuron_idx],
        );

        let mut ledger = FireLedger::new(8);
        ledger.track_area(cortical, 8).unwrap();
        archive_burst_fires(&mut ledger, 1, cortical, &[neuron_idx as u32]);

        let mut leaks = vec![0.0_f32; 32];
        leaks[neuron_idx] = 0.1;
        reg.apply_burstal(1, &ledger, &mut leaks);

        let tau_r = 1.0_f32;
        let alpha_r = 1.0 - (-1.0_f32 / tau_r).exp();
        let expected_r = alpha_r;
        let expected_g = (0.1_f32 + 1.0 * (expected_r - 0.0)).clamp(0.0, 1.0);
        assert!(
            (leaks[neuron_idx] - expected_g).abs() < 1e-5,
            "leak={} expected {}",
            leaks[neuron_idx],
            expected_g
        );
    }

    #[test]
    fn apply_burstal_silence_below_target_pushes_leak_toward_leak_min() {
        let neuron_idx: usize = 4;
        let cortical = 11u32;
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            cortical,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.9,
                "rate_ema_tau_bursts": 50.0,
                "gain": 0.5,
                "leak_min": 0.05,
                "leak_max": 0.9,
            }),
            0.7,
            vec![neuron_idx],
        );

        let mut ledger = FireLedger::new(16);
        ledger.track_area(cortical, 32).unwrap();
        for t in 1..=40u64 {
            archive_burst_silent(&mut ledger, t);
        }

        let mut leaks = vec![0.0_f32; 24];
        leaks[neuron_idx] = 0.7;
        for t in 1..=40u64 {
            reg.apply_burstal(t, &ledger, &mut leaks);
        }
        assert!(
            leaks[neuron_idx] <= 0.7 + 1e-4,
            "silence should not increase leak: {}",
            leaks[neuron_idx]
        );
        assert!(
            (leaks[neuron_idx] - 0.05).abs() < 0.06,
            "expected near leak_min 0.05, got {}",
            leaks[neuron_idx]
        );
    }

    /// Cadence aliasing: a neuron that fires on bursts 1,2,4,5 (never on update bursts 3,6)
    /// has true rate ~0.67 > target 0.1 (tau 3 keeps the EMA from decaying to ~0 by each silent update burst), so a correct homeostat raises leak.
    #[test]
    fn apply_burstal_cadence_aliasing_does_not_hide_firing() {
        let neuron_idx: usize = 2;
        let cortical = 13u32;
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            cortical,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.1,
                "rate_ema_tau_bursts": 3.0,
                "gain": 0.05,
                "leak_min": 0.0,
                "leak_max": 1.0,
                "update_every_n_bursts": 3,
            }),
            0.2,
            vec![neuron_idx],
        );
        let mut ledger = FireLedger::new(16);
        ledger.track_area(cortical, 16).unwrap();
        let mut leaks = vec![0.0_f32; 8];
        leaks[neuron_idx] = 0.2;
        for t in 1..=6u64 {
            if t % 3 == 0 {
                archive_burst_silent(&mut ledger, t);
            } else {
                archive_burst_fires(&mut ledger, t, cortical, &[neuron_idx as u32]);
            }
            reg.apply_burstal(t, &ledger, &mut leaks);
        }
        println!("aliasing leak after 6 bursts: {}", leaks[neuron_idx]);
        assert!(
            leaks[neuron_idx] > 0.2,
            "firing rate 0.67 > target 0.1 should raise leak above 0.2; got {}",
            leaks[neuron_idx]
        );
    }

    #[test]
    fn apply_burstal_update_every_n_bursts_cadence() {
        let neuron_idx: usize = 2;
        let cortical = 12u32;
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            cortical,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.0,
                "rate_ema_tau_bursts": 1.0,
                "gain": 0.05,
                "leak_min": 0.0,
                "leak_max": 1.0,
                "update_every_n_bursts": 3,
            }),
            0.2,
            vec![neuron_idx],
        );

        let mut ledger = FireLedger::new(16);
        ledger.track_area(cortical, 16).unwrap();

        let mut leaks = vec![0.0_f32; 8];
        leaks[neuron_idx] = 0.2;

        for t in 1..=6u64 {
            archive_burst_fires(&mut ledger, t, cortical, &[neuron_idx as u32]);
            reg.apply_burstal(t, &ledger, &mut leaks);
        }

        assert!(
            (leaks[neuron_idx] - 0.2).abs() > 1e-4,
            "leak should move after bursts with cadence-3 updates; got {}",
            leaks[neuron_idx]
        );
        // The EMA is sampled every burst; the leak step applies only on bursts 3 and 6 (t % 3 == 0).
        // Assert a non-trivial rise vs 0.2.
        assert!(leaks[neuron_idx] > 0.2, "{}", leaks[neuron_idx]);
    }

    #[test]
    fn apply_burstal_update_every_n_bursts_zero_treated_as_one() {
        let neuron_idx: usize = 1;
        let cortical = 13u32;
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            cortical,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.0,
                "rate_ema_tau_bursts": 1.0,
                "gain": 0.1,
                "leak_min": 0.0,
                "leak_max": 1.0,
                "update_every_n_bursts": 0,
            }),
            0.15,
            vec![neuron_idx],
        );

        let mut ledger = FireLedger::new(8);
        ledger.track_area(cortical, 8).unwrap();
        archive_burst_fires(&mut ledger, 1, cortical, &[neuron_idx as u32]);

        let mut leaks = vec![0.0_f32; 8];
        leaks[neuron_idx] = 0.15;
        reg.apply_burstal(1, &ledger, &mut leaks);
        assert!(
            leaks[neuron_idx] > 0.15,
            "zero n should mean update every burst"
        );
    }

    #[test]
    fn apply_burstal_two_neurons_independent() {
        let cortical = 14u32;
        let a: usize = 20;
        let b: usize = 21;
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            cortical,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.0,
                "rate_ema_tau_bursts": 1.0,
                "gain": 0.5,
                "leak_min": 0.0,
                "leak_max": 1.0,
            }),
            0.1,
            vec![a, b],
        );

        let mut ledger = FireLedger::new(16);
        ledger.track_area(cortical, 16).unwrap();
        let mut fq = FireQueue::new();
        fq.add_neuron(FiringNeuron {
            neuron_id: NeuronId(a as u32),
            membrane_potential: 1.0,
            cortical_idx: cortical,
            x: 0,
            y: 0,
            z: 0,
            fire_kind: FIRE_KIND_STDP_ELIGIBLE,
        });
        ledger.archive_burst(1, &fq).unwrap();

        let mut leaks = vec![0.0_f32; 64];
        leaks[a] = 0.1;
        leaks[b] = 0.1;
        reg.apply_burstal(1, &ledger, &mut leaks);

        let alpha = 1.0 - (-1.0_f32).exp();
        let r_a = alpha;
        let r_b = 0.0;
        let g_a = (0.1_f32 + 0.5 * (r_a - 0.0)).clamp(0.0_f32, 1.0_f32);
        let g_b = (0.1_f32 + 0.5 * (r_b - 0.0)).clamp(0.0_f32, 1.0_f32);
        assert!((leaks[a] - g_a).abs() < 1e-4);
        assert!((leaks[b] - g_b).abs() < 1e-4);
        assert!(leaks[a] > leaks[b]);
    }

    #[test]
    fn apply_burstal_index_out_of_storage_skips_without_panic() {
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            15,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.0,
                "rate_ema_tau_bursts": 1.0,
                "gain": 1.0,
            }),
            0.2,
            vec![100usize],
        );
        let mut ledger = FireLedger::new(8);
        ledger.track_area(15, 8).unwrap();
        archive_burst_fires(&mut ledger, 1, 15, &[100]);
        let mut leaks = vec![0.33_f32; 8];
        reg.apply_burstal(1, &ledger, &mut leaks);
        assert!((leaks[0] - 0.33).abs() < 1e-5);
    }

    #[test]
    fn apply_burstal_untracked_area_skips_without_panic() {
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            99,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.0,
                "rate_ema_tau_bursts": 1.0,
                "gain": 1.0,
            }),
            0.5,
            vec![3usize],
        );
        let mut ledger = FireLedger::new(8);
        ledger.track_area(1, 8).unwrap();
        archive_burst_fires(&mut ledger, 1, 1, &[3]);
        let mut leaks = vec![0.0_f32; 8];
        leaks[3] = 0.5;
        reg.apply_burstal(1, &ledger, &mut leaks);
        assert!((leaks[3] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn apply_burstal_g_max_less_than_g_min_skips_area() {
        let neuron_idx: usize = 6;
        let cortical = 16u32;
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            cortical,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.0,
                "rate_ema_tau_bursts": 1.0,
                "gain": 1.0,
                "leak_min": 0.8,
                "leak_max": 0.1,
            }),
            0.3,
            vec![neuron_idx],
        );

        let mut ledger = FireLedger::new(8);
        ledger.track_area(cortical, 8).unwrap();
        archive_burst_fires(&mut ledger, 1, cortical, &[neuron_idx as u32]);

        let mut leaks = vec![0.0_f32; 16];
        leaks[neuron_idx] = 0.3;
        reg.apply_burstal(1, &ledger, &mut leaks);
        assert!((leaks[neuron_idx] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn apply_burstal_clamps_to_leak_max() {
        let neuron_idx: usize = 8;
        let cortical = 17u32;
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            cortical,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.0,
                "rate_ema_tau_bursts": 1.0,
                "gain": 10.0,
                "leak_min": 0.0,
                "leak_max": 0.25,
            }),
            0.2,
            vec![neuron_idx],
        );

        let mut ledger = FireLedger::new(8);
        ledger.track_area(cortical, 8).unwrap();
        archive_burst_fires(&mut ledger, 1, cortical, &[neuron_idx as u32]);

        let mut leaks = vec![0.0_f32; 16];
        leaks[neuron_idx] = 0.2;
        reg.apply_burstal(1, &ledger, &mut leaks);
        assert!((leaks[neuron_idx] - 0.25).abs() < 1e-4);
    }

    #[test]
    fn apply_burstal_seeds_from_base_leak_on_sync() {
        let neuron_idx: usize = 9;
        let cortical = 18u32;
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            cortical,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.5,
                "rate_ema_tau_bursts": 1.0,
                "gain": 0.0,
                "leak_min": 0.0,
                "leak_max": 1.0,
            }),
            0.42,
            vec![neuron_idx],
        );

        let mut ledger = FireLedger::new(8);
        ledger.track_area(cortical, 8).unwrap();
        archive_burst_silent(&mut ledger, 1);

        let mut leaks = vec![0.0_f32; 16];
        leaks[neuron_idx] = 0.99;
        reg.apply_burstal(1, &ledger, &mut leaks);
        // gain=0 -> g stays at seeded 0.42 regardless of EMA
        assert!((leaks[neuron_idx] - 0.42).abs() < 1e-5);
    }

    #[test]
    fn apply_burstal_base_leak_above_one_clamps_seed() {
        let neuron_idx: usize = 5;
        let cortical = 19u32;
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(
            cortical,
            &json!({
                "enabled": true,
                "target_firing_per_burst": 0.5,
                "rate_ema_tau_bursts": 1.0,
                "gain": 0.0,
                "leak_min": 0.0,
                "leak_max": 1.0,
            }),
            1.7,
            vec![neuron_idx],
        );

        let mut ledger = FireLedger::new(8);
        ledger.track_area(cortical, 8).unwrap();
        archive_burst_silent(&mut ledger, 1);

        let mut leaks = vec![0.0_f32; 16];
        reg.apply_burstal(1, &ledger, &mut leaks);
        assert!((leaks[neuron_idx] - 1.0).abs() < 1e-5);
    }

    fn enabled_with(field: &str, v: Value) -> Value {
        let mut o = json!({ "enabled": true });
        o[field] = v;
        o
    }

    #[test]
    fn validate_disabled_is_ok_none() {
        assert!(matches!(
            RateModulatedLeakConfig::validate(&json!({ "enabled": false, "gain": -5.0 })),
            Ok(None)
        ));
        assert!(matches!(
            RateModulatedLeakConfig::validate(&json!({})),
            Ok(None)
        ));
    }

    #[test]
    fn validate_enabled_defaults_ok() {
        let c = RateModulatedLeakConfig::validate(&json!({ "enabled": true })).unwrap();
        assert!(c.is_some());
    }

    #[test]
    fn validate_rejects_each_bad_field() {
        let cases = [
            ("rate_ema_tau_bursts", json!(0.0), "rate_ema_tau_bursts"),
            ("rate_ema_tau_bursts", json!(-1.0), "rate_ema_tau_bursts"),
            ("gain", json!(-0.1), "gain"),
            (
                "target_firing_per_burst",
                json!(1.5),
                "target_firing_per_burst",
            ),
            (
                "target_firing_per_burst",
                json!(-0.1),
                "target_firing_per_burst",
            ),
            ("leak_min", json!(-0.1), "leak_min"),
            ("leak_min", json!(1.2), "leak_min"),
            ("leak_max", json!(1.2), "leak_max"),
            ("leak_max", json!(-0.2), "leak_max"),
            ("gain", json!("high"), "invalid field types"),
        ];
        for (field, val, needle) in cases {
            let err = RateModulatedLeakConfig::validate(&enabled_with(field, val.clone()))
                .expect_err(&format!("{field}={val} should be rejected"));
            assert!(err.contains(needle), "{field}={val}: {err}");
        }
    }

    #[test]
    fn validate_rejects_min_above_max_and_non_object() {
        let v = json!({ "enabled": true, "leak_min": 0.6, "leak_max": 0.3 });
        let err = RateModulatedLeakConfig::validate(&v).unwrap_err();
        assert!(
            err.contains("leak_min") && err.contains("leak_max"),
            "{err}"
        );
        assert!(RateModulatedLeakConfig::validate(&json!("on")).is_err());
        assert!(RateModulatedLeakConfig::validate(&json!(true)).is_err());
        assert!(RateModulatedLeakConfig::validate(&json!({ "enabled": "yes" })).is_err());
    }

    #[test]
    fn parse_from_property_invalid_is_none() {
        let v = json!({ "enabled": true, "gain": -1.0 });
        assert!(RateModulatedLeakConfig::parse_from_cortical_property(&v).is_none());
    }

    #[test]
    fn reseed_leak_sets_all_g_leak_and_keeps_r_ema() {
        let mut reg = RateModulatedLeakRegistry::default();
        reg.sync_cortical(7, &json!({ "enabled": true }), 0.1, vec![0, 1]);
        reg.areas.get_mut(&7).unwrap().r_ema = vec![0.4, 0.6];
        reg.reseed_leak(7, 1.7);
        let st = &reg.areas[&7];
        assert_eq!(st.g_leak, vec![1.0, 1.0]);
        assert_eq!(st.r_ema, vec![0.4, 0.6]);
        reg.reseed_leak(7, 0.25);
        assert_eq!(reg.areas[&7].g_leak, vec![0.25, 0.25]);
        reg.reseed_leak(99, 0.5); // unknown area: no-op
        reg.clear();
        assert!(reg.is_empty());
    }

    #[test]
    fn rate_ema_by_neuron_maps_registered_area_only() {
        let mut reg = RateModulatedLeakRegistry::default();
        assert!(reg.rate_ema_by_neuron(7).is_none());
        reg.sync_cortical(
            7,
            &json!({ "enabled": true, "target_firing_per_burst": 0.2 }),
            0.3,
            vec![4, 5],
        );
        let m = reg.rate_ema_by_neuron(7).expect("registered area");
        assert_eq!(m.len(), 2);
        assert_eq!(m.get(&4), Some(&0.0));
        reg.remove(7);
        assert!(reg.rate_ema_by_neuron(7).is_none());
    }
}
