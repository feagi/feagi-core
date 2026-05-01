// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! Reward-modulated STDP (R-STDP) integration tests.
//!
//! These tests exercise the unified plasticity loop with `PlasticityMode::RStdp`:
//!   - Eligibility traces accumulate on co-firing without committing weight.
//!   - End-of-burst commits scale by `R(t) = density(reward) - density(pain)`.
//!   - Trace decays each burst with `eligibility_decay_bursts`.
//!   - `PlasticityMode::Off` is a true no-op even on co-firing.
//!
//! Network topology (shared across tests):
//!   area 10: src   (3 LIF neurons)
//!   area 11: dst   (3 LIF neurons)
//!   area 12: reward detector (1 LIF neuron)
//!   area 13: punishment detector (1 LIF neuron)
//!
//! Plastic mapping is always 10→11 with one pre-existing src[0]→dst[0] synapse so we can
//! observe weight evolution without invoking the bidirectional synapse-creation path.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::npu::{PlasticityMode, StdpMappingParams};
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::CoreCorticalType;

type RstdpTestNetwork = (
    RustNPU<StdRuntime, f32, CPUBackend>,
    Vec<NeuronId>,
    Vec<NeuronId>,
    NeuronId,
    NeuronId,
);

/// Build a network with src(10), dst(11), reward(12), punishment(13) areas.
///
/// Returns (npu, src_neurons, dst_neurons, reward_neuron, pain_neuron).
fn create_rstdp_network() -> RstdpTestNetwork {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 100, 1000, 10).unwrap();

    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());

    npu.register_cortical_area(10, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(11, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(12, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(13, CoreCorticalType::Death.to_cortical_id().as_base_64());

    let mut src = Vec::new();
    let mut dst = Vec::new();
    for i in 0..3 {
        src.push(
            npu.add_neuron(1.0, f32::MAX, 0.1, 0.0, 0, 0, 1.0, 0, 0, true, 10, i, 0, 0)
                .unwrap(),
        );
        dst.push(
            npu.add_neuron(1.0, f32::MAX, 0.1, 0.0, 0, 0, 1.0, 0, 0, true, 11, i, 0, 0)
                .unwrap(),
        );
    }
    let reward = npu
        .add_neuron(1.0, f32::MAX, 0.1, 0.0, 0, 0, 1.0, 0, 0, true, 12, 0, 0, 0)
        .unwrap();
    let pain = npu
        .add_neuron(1.0, f32::MAX, 0.1, 0.0, 0, 0, 1.0, 0, 0, true, 13, 0, 0, 0)
        .unwrap();

    // Fire-ledger tracking: plasticity_window=1 needs depth≥1; reward/pain density needs depth≥1.
    npu.configure_fire_ledger_window(10, 1).unwrap();
    npu.configure_fire_ledger_window(11, 1).unwrap();
    npu.configure_fire_ledger_window(12, 1).unwrap();
    npu.configure_fire_ledger_window(13, 1).unwrap();

    (npu, src, dst, reward, pain)
}

/// Plastic mapping params with a default R-STDP shape; tests override mode/decay/sources.
fn rstdp_params(
    mode: PlasticityMode,
    eligibility_decay_bursts: u32,
    reward_source_area: Option<u32>,
    punishment_source_area: Option<u32>,
) -> StdpMappingParams {
    StdpMappingParams {
        plasticity_window: 1,
        plasticity_constant: 4,
        ltp_multiplier: 2,
        ltd_multiplier: 1,
        bidirectional_stdp: false,
        synapse_psp: 100.0,
        synapse_type: SynapseType::Excitatory,
        plasticity_mode: mode,
        eligibility_decay_bursts,
        reward_source_area,
        punishment_source_area,
        max_weight: f32::INFINITY,
        plasticity_eta: 1.0,
    }
}

/// Add a single src[0]→dst[0] excitatory synapse with `initial_weight`.
///
/// PSP is set to 0.0 so propagated contribution (`weight * psp`) is always 0. This prevents
/// dst from firing on subsequent bursts via delayed propagation (the synapse delay is 1
/// burst, so a src fire at burst T would otherwise drive dst to threshold at burst T+1 and
/// trip an unintended LTD step). Plasticity itself still operates on `weight` directly, so
/// dropping PSP keeps the test focused on R-STDP weight-commit semantics.
///
/// `rebuild_synapse_index` ensures the propagation engine and STDP mapping index both pick
/// up the new edge. `register_stdp_mapping` rebuilds the mapping index again afterwards, so
/// callers may register the plastic mapping either before or after this call.
fn wire_test_synapse(
    npu: &mut RustNPU<StdRuntime, f32, CPUBackend>,
    src: NeuronId,
    dst: NeuronId,
    initial_weight: f32,
) {
    npu.add_synapse(
        src,
        dst,
        SynapticWeight(initial_weight),
        SynapticPsp(0.0),
        SynapseType::Excitatory,
        0,
        1,
    )
    .unwrap();
    npu.rebuild_synapse_index();
}

fn synapse_weight(npu: &RustNPU<StdRuntime, f32, CPUBackend>, src: NeuronId) -> f32 {
    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(outgoing.len(), 1, "expected exactly one outgoing synapse");
    outgoing[0].1
}

/// `PlasticityMode::Off` must not change weights, even on perfect co-firing.
#[test]
fn test_off_mode_is_no_op() {
    let (mut npu, src, dst, _reward, _pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    let params = rstdp_params(PlasticityMode::Off, 0, None, None);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0)]);
    npu.process_burst().unwrap();

    assert_eq!(
        synapse_weight(&npu, src[0]),
        5.0,
        "Off mode must leave weight untouched"
    );
}

/// R-STDP with no reward/pain firing accumulates a positive eligibility trace on co-firing
/// but commits zero weight change (R(t)=0).
#[test]
fn test_rstdp_no_reward_means_no_weight_change() {
    let (mut npu, src, dst, _reward, _pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    let params = rstdp_params(PlasticityMode::RStdp, 10, Some(12), Some(13));
    npu.register_stdp_mapping(10, 11, params).unwrap();

    // Burst 1: src + dst co-fire, reward+pain silent. Trace builds, R(t)=0, weight unchanged.
    npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0)]);
    npu.process_burst().unwrap();

    assert_eq!(
        synapse_weight(&npu, src[0]),
        5.0,
        "Without reward, R-STDP must not commit weight change"
    );
}

/// R-STDP with a delayed reward: build a trace on burst 1, then fire the reward detector on
/// burst 2 → weight grows by ~ decay_factor * delta_plus. Validates eligibility persistence
/// and end-of-burst R(t) modulation.
#[test]
fn test_rstdp_delayed_reward_commits_weight() {
    let (mut npu, src, dst, reward, _pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    let decay_bursts = 10u32;
    let params = rstdp_params(PlasticityMode::RStdp, decay_bursts, Some(12), Some(13));
    npu.register_stdp_mapping(10, 11, params).unwrap();

    // Burst 1: src+dst co-fire, reward silent.
    npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0)]);
    npu.process_burst().unwrap();
    assert_eq!(
        synapse_weight(&npu, src[0]),
        5.0,
        "burst 1: trace builds but no reward yet"
    );

    // Burst 2: reward fires; src/dst silent. Trace has decayed once but is still positive.
    npu.inject_sensory_with_potentials(&[(reward, 128.0)]);
    npu.process_burst().unwrap();

    let w_after = synapse_weight(&npu, src[0]);
    assert!(
        w_after > 5.0,
        "reward should drive positive weight commit, got {}",
        w_after
    );

    // Sanity: commit cannot exceed delta_plus * R (R = 1.0 with single reward neuron firing).
    // delta_plus = plasticity_constant * ltp_multiplier = 4 * 2 = 8. Decay over 1 burst at
    // tau=10 leaves trace ≈ 8 * exp(-1/10) ≈ 7.24. So expected delta ≈ 7.24, weight ≈ 12.24.
    let expected_delta = 8.0_f32 * (-1.0_f32 / decay_bursts as f32).exp();
    let observed_delta = w_after - 5.0;
    assert!(
        (observed_delta - expected_delta).abs() < 0.1,
        "expected weight delta near {}, got {}",
        expected_delta,
        observed_delta
    );
}

/// `plasticity_eta` scales the weight commit `w += eta * R * e` without changing trace build-up.
/// At `0.5`, the first reward commit should be half the `eta = 1.0` reference.
#[test]
fn test_rstdp_plasticity_eta_scales_weight_commit() {
    let (mut npu, src, dst, reward, _pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    let decay_bursts = 10u32;
    let mut params = rstdp_params(PlasticityMode::RStdp, decay_bursts, Some(12), Some(13));
    params.plasticity_eta = 0.5;
    npu.register_stdp_mapping(10, 11, params).unwrap();

    npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0)]);
    npu.process_burst().unwrap();
    npu.inject_sensory_with_potentials(&[(reward, 128.0)]);
    npu.process_burst().unwrap();

    let w_eta = synapse_weight(&npu, src[0]) - 5.0;
    let expected_full = 8.0_f32 * (-1.0_f32 / decay_bursts as f32).exp();
    assert!(
        (w_eta - 0.5 * expected_full).abs() < 0.1,
        "eta=0.5 should halve the weight delta; got {}, expected ~{}",
        w_eta,
        0.5 * expected_full
    );
}

/// Punishment area firing produces R(t) < 0 → weight decreases despite co-firing.
#[test]
fn test_rstdp_punishment_drives_negative_weight_change() {
    let (mut npu, src, dst, _reward, pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 20.0);

    let params = rstdp_params(PlasticityMode::RStdp, 10, Some(12), Some(13));
    npu.register_stdp_mapping(10, 11, params).unwrap();

    // Burst 1: src+dst co-fire, no reward/pain.
    npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0)]);
    npu.process_burst().unwrap();
    assert_eq!(synapse_weight(&npu, src[0]), 20.0);

    // Burst 2: pain fires. R(t) = 0 - 1.0 = -1.0; trace is positive → negative weight delta.
    npu.inject_sensory_with_potentials(&[(pain, 128.0)]);
    npu.process_burst().unwrap();

    let w_after = synapse_weight(&npu, src[0]);
    assert!(
        w_after < 20.0,
        "punishment should reduce weight, got {}",
        w_after
    );
}

/// Pleasure and punishment that fire together cancel: R(t) = density(reward) - density(pain) = 0,
/// so no weight change even with a positive eligibility trace.
#[test]
fn test_rstdp_balanced_reward_and_punishment_zero_change() {
    let (mut npu, src, dst, reward, pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    let params = rstdp_params(PlasticityMode::RStdp, 10, Some(12), Some(13));
    npu.register_stdp_mapping(10, 11, params).unwrap();

    // Burst 1: build trace.
    npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0)]);
    npu.process_burst().unwrap();
    assert_eq!(synapse_weight(&npu, src[0]), 5.0);

    // Burst 2: equal reward and pain density (both 1-neuron areas, both fire fully).
    npu.inject_sensory_with_potentials(&[(reward, 128.0), (pain, 128.0)]);
    npu.process_burst().unwrap();

    assert_eq!(
        synapse_weight(&npu, src[0]),
        5.0,
        "Balanced reward and punishment must net to zero weight change"
    );
}

/// Trace decay: a long delay between trace formation and reward yields a smaller commit than
/// an immediate one. Compares weight delta after 1 vs 5 silent bursts of separation.
#[test]
fn test_rstdp_trace_decay_reduces_late_commit() {
    fn run(silent_bursts: usize) -> f32 {
        let (mut npu, src, dst, reward, _pain) = create_rstdp_network();
        wire_test_synapse(&mut npu, src[0], dst[0], 5.0);
        let params = rstdp_params(PlasticityMode::RStdp, 5, Some(12), Some(13));
        npu.register_stdp_mapping(10, 11, params).unwrap();

        // Build trace.
        npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0)]);
        npu.process_burst().unwrap();

        // Let trace decay across silent bursts.
        for _ in 0..silent_bursts {
            npu.process_burst().unwrap();
        }

        // Apply reward.
        npu.inject_sensory_with_potentials(&[(reward, 128.0)]);
        npu.process_burst().unwrap();

        synapse_weight(&npu, src[0]) - 5.0
    }

    let early = run(0); // 1 burst of decay (the reward burst itself)
    let late = run(4); // 5 bursts of decay before the reward burst

    assert!(early > 0.0, "early reward must commit a positive delta");
    assert!(late > 0.0, "late reward must still commit a positive delta");
    assert!(
        late < early,
        "late commit must be smaller than early commit (decay): early={}, late={}",
        early,
        late
    );
}

/// Verifies that `eligibility_decay_bursts == 0` makes R-STDP trace fully reset each burst —
/// meaning it behaves like classic STDP with R(t) modulation but no temporal credit assignment.
/// Without persistent traces, a delayed reward cannot commit anything (trace was wiped).
#[test]
fn test_rstdp_zero_decay_means_no_temporal_credit() {
    let (mut npu, src, dst, reward, _pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    let params = rstdp_params(PlasticityMode::RStdp, 0, Some(12), Some(13));
    npu.register_stdp_mapping(10, 11, params).unwrap();

    // Burst 1: build trace via co-fire (no reward).
    npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0)]);
    npu.process_burst().unwrap();

    // Burst 2: reward fires alone. Trace was wiped at start of this burst (decay=0) → R*0 = 0.
    npu.inject_sensory_with_potentials(&[(reward, 128.0)]);
    npu.process_burst().unwrap();

    assert_eq!(
        synapse_weight(&npu, src[0]),
        5.0,
        "Zero-decay R-STDP must lose pre-reward eligibility"
    );
}

/// Wireheading lint check 1: register a plastic mapping that declares area 12 as the
/// reward source, then attempt to register a second plastic mapping whose destination is
/// area 12. The second registration must fail.
#[test]
fn test_wireheading_rejects_plastic_input_into_reward_area() {
    let (mut npu, src, dst, reward, _pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    // First mapping: 10 -> 11, declares 12 (reward) and 13 (pain) as protected.
    let p1 = rstdp_params(PlasticityMode::RStdp, 10, Some(12), Some(13));
    npu.register_stdp_mapping(10, 11, p1).unwrap();

    // Second mapping: 10 -> 12 with plain STDP. dst=12 is protected -> must reject.
    let p2 = rstdp_params(PlasticityMode::Stdp, 0, None, None);
    let err = npu
        .register_stdp_mapping(10, 12, p2)
        .expect_err("expected wireheading rejection for plastic mapping into reward area");
    let msg = format!("{}", err);
    assert!(
        msg.contains("Wireheading"),
        "error must explicitly mention wireheading, got: {}",
        msg
    );

    // The reward area neuron is still inert in this assertion; we only care about registration.
    let _ = reward;
}

/// Wireheading lint check 2: register a plain plastic mapping that targets area 12 first,
/// then attempt to declare area 12 as a reward source on a *different* plastic mapping. The
/// declaration must fail because area 12 already receives plastic input.
#[test]
fn test_wireheading_rejects_protected_area_with_existing_plastic_input() {
    let (mut npu, src, dst, _reward, _pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    // First mapping: 10 -> 12 with plain STDP. Area 12 is now plastic-input.
    let p1 = rstdp_params(PlasticityMode::Stdp, 0, None, None);
    npu.register_stdp_mapping(10, 12, p1).unwrap();

    // Second mapping: 10 -> 11, R-STDP, declares 12 as reward source -> must reject.
    let p2 = rstdp_params(PlasticityMode::RStdp, 10, Some(12), None);
    let err = npu
        .register_stdp_mapping(10, 11, p2)
        .expect_err("expected wireheading rejection for declaring already-plastic dst as reward");
    let msg = format!("{}", err);
    assert!(
        msg.contains("Wireheading"),
        "error must explicitly mention wireheading, got: {}",
        msg
    );
}

/// Off-mode mappings are exempt from the wireheading lint: a mapping with plasticity_mode=Off
/// pointing into a reward source area is the legitimate hard-wired detector path itself.
#[test]
fn test_wireheading_allows_off_mode_input_into_reward_area() {
    let (mut npu, src, dst, _reward, _pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    // Mapping declaring area 12 as reward source.
    let p1 = rstdp_params(PlasticityMode::RStdp, 10, Some(12), None);
    npu.register_stdp_mapping(10, 11, p1).unwrap();

    // Off-mode mapping into the reward area. This represents the hard-wired sensor->detector
    // path that builds the reward signal and must be allowed.
    let p2 = rstdp_params(PlasticityMode::Off, 0, None, None);
    npu.register_stdp_mapping(10, 12, p2)
        .expect("Off-mode mapping into reward area must be allowed");
}

/// `max_weight` clamps positive R-STDP commits at the configured ceiling. Drives sustained
/// pre-post co-firing with continuous reward and verifies the synaptic weight saturates at
/// `max_weight` rather than growing without bound. Regression for the runaway-PSP postmortem
/// (cartpole motor weights reached ~5.4M before the clamp existed; see
/// feagi-mcp/docs/FAQ.md "My motor PSPs are saturating to millions").
#[test]
fn test_rstdp_max_weight_clamps_runaway_potentiation() {
    let (mut npu, src, dst, reward, _pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    let cap: f32 = 7.5;
    let mut params = rstdp_params(PlasticityMode::RStdp, 10, Some(12), Some(13));
    params.max_weight = cap;
    npu.register_stdp_mapping(10, 11, params).unwrap();

    // Drive 50 bursts of co-fire + reward. With delta_plus=8 and decay=10 the steady-state
    // trace is ≈ 8/(1-exp(-1/10)) ≈ 84 per burst; without a clamp the weight would grow into
    // the thousands. Clamp must hold the weight at exactly `cap` instead.
    for _ in 0..50 {
        npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0), (reward, 128.0)]);
        npu.process_burst().unwrap();
    }

    let w_final = synapse_weight(&npu, src[0]);
    assert!(
        (w_final - cap).abs() < 1e-4,
        "max_weight must clamp positive growth at cap={}, got w_final={}",
        cap,
        w_final
    );
}

/// `max_weight = f32::INFINITY` (the default) preserves the legacy unbounded-growth behaviour
/// so existing genomes round-trip cleanly through the new code path. Mirrors the "delayed
/// reward commits weight" regression but adds an explicit infinity cap to confirm the
/// .min(f32::INFINITY) edge case behaves as identity.
#[test]
fn test_rstdp_max_weight_infinity_preserves_legacy_growth() {
    let (mut npu, src, dst, reward, _pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 5.0);

    let mut params = rstdp_params(PlasticityMode::RStdp, 10, Some(12), Some(13));
    params.max_weight = f32::INFINITY;
    npu.register_stdp_mapping(10, 11, params).unwrap();

    // Single co-fire burst, then a reward burst (matches existing
    // test_rstdp_delayed_reward_commits_weight scenario).
    npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0)]);
    npu.process_burst().unwrap();
    npu.inject_sensory_with_potentials(&[(reward, 128.0)]);
    npu.process_burst().unwrap();

    let w_after = synapse_weight(&npu, src[0]);
    assert!(
        w_after > 5.0,
        "infinity cap must allow weight to grow as in the legacy path, got {}",
        w_after
    );

    // Same expected_delta computation as test_rstdp_delayed_reward_commits_weight (delta_plus
    // = 4 * 2 = 8, one burst of decay at tau=10).
    let expected_delta = 8.0_f32 * (-1.0_f32 / 10.0_f32).exp();
    let observed_delta = w_after - 5.0;
    assert!(
        (observed_delta - expected_delta).abs() < 0.1,
        "infinity cap must not perturb growth magnitude: expected ≈ {}, got {}",
        expected_delta,
        observed_delta
    );
}

/// `max_weight` is a one-sided ceiling: punishment / LTD must still be able to drive the
/// weight all the way to 0 even when a clamp is configured. Regression for the case where a
/// naïve clamp implementation would also clamp the lower bound.
#[test]
fn test_rstdp_max_weight_does_not_block_punishment_to_zero() {
    let (mut npu, src, dst, _reward, pain) = create_rstdp_network();
    wire_test_synapse(&mut npu, src[0], dst[0], 20.0);

    let mut params = rstdp_params(PlasticityMode::RStdp, 10, Some(12), Some(13));
    params.max_weight = 50.0; // Well above starting weight; clamp inactive on this path.
    npu.register_stdp_mapping(10, 11, params).unwrap();

    // Burst 1: build a positive trace via co-fire (no reward, no pain).
    npu.inject_sensory_with_potentials(&[(src[0], 128.0), (dst[0], 128.0)]);
    npu.process_burst().unwrap();
    assert_eq!(synapse_weight(&npu, src[0]), 20.0);

    // Drive sustained pain. R(t) = -1 each burst and the trace is positive, so commits are
    // negative. Floor must remain 0.0; the clamp must not interfere.
    for _ in 0..50 {
        npu.inject_sensory_with_potentials(&[(pain, 128.0)]);
        npu.process_burst().unwrap();
    }

    let w_final = synapse_weight(&npu, src[0]);
    assert!(
        (0.0..1.0).contains(&w_final),
        "punishment must drive weight toward 0 even with max_weight set; got {}",
        w_final
    );
}
