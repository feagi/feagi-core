//! Test to reproduce user's projector issue:
//! - Area A: 1x1x1 (1 neuron) with PSP=1, psp_uniform=false
//! - Area B: 1x10x1 (10 neurons) with threshold=10, mp_acc=false
//! - 1 neuron in A connected to ALL 10 neurons in B (projector behavior)
//! - Expected: PSP divided 1/10 = 0, NO neurons fire
//! - Actual (reported): ALL neurons fire!

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::npu::RustNPU;
use feagi_npu_neural::synapse::SynapseType;
use feagi_npu_neural::types::{SynapticPsp, SynapticWeight};
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};

#[test]
fn test_projector_psp_division_issue() {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 1000, 10000, 20).unwrap();

    // Register cortical areas (avoid area=1 for power injection)
    npu.register_cortical_area(10, CoreCorticalType::Death.to_cortical_id().as_base_64()); // Area A
    npu.register_cortical_area(11, CoreCorticalType::Power.to_cortical_id().as_base_64()); // Area B

    // Create neuron in Area A (source)
    let neuron_a = npu
        .add_neuron(
            10.0,     // threshold (high so it only fires from direct injection)
            f32::MAX, // threshold_limit (MAX = no limit, SIMD-friendly encoding)
            0.0,      // leak
            0.0,      // resting_potential
            0,        // neuron_type
            0,        // refractory_period
            1.0,      // excitability
            u16::MAX, // consecutive_fire_limit (MAX = unlimited, SIMD-friendly encoding)
            0,        // snooze_period
            false,    // mp_charge_accumulation
            10,       // cortical_area (A)
            0,
            0,
            0,
        )
        .unwrap();

    // Create 10 neurons in Area B (targets) with threshold=10
    let mut neurons_b = Vec::new();
    for i in 0..10 {
        let neuron = npu
            .add_neuron(
                10.0,     // threshold = 10
                f32::MAX, // threshold_limit (MAX = no limit, SIMD-friendly encoding)
                0.0,      // leak
                0.0,      // resting_potential
                0,        // neuron_type
                0,        // refractory_period
                1.0,      // excitability
                u16::MAX, // consecutive_fire_limit (MAX = unlimited, SIMD-friendly encoding)
                0,        // snooze_period
                false,    // mp_charge_accumulation = FALSE
                11,       // cortical_area (B)
                0,
                i,
                0,
            )
            .unwrap();
        neurons_b.push(neuron);
    }

    // Create synapses: neuron_a → all 10 neurons in B
    // Using weight=1, psp=1 (PSP = 1×1 = 1)
    println!("\n=== Creating Synapses ===");
    for (i, &target) in neurons_b.iter().enumerate() {
        npu.add_synapse(
            neuron_a,
            target,
            SynapticWeight(1),      // weight = 1
            SynapticPsp(1), // PSP = 1
            SynapseType::Excitatory,
        )
        .unwrap();
        println!(
            "Synapse {}: {} -> {} (weight=1, psp=1)",
            i, neuron_a.0, target.0
        );
    }

    // CRITICAL: Rebuild synapse index
    npu.rebuild_synapse_index();
    println!("Rebuilt synapse index");

    // Set psp_uniform_distribution = FALSE for Area A
    let mut psp_flags = ahash::AHashMap::new();
    let area_a_id =
        CorticalID::try_from_base_64(&CoreCorticalType::Death.to_cortical_id().as_base_64())
            .unwrap();
    psp_flags.insert(area_a_id, false); // FALSE = divide PSP
    npu.set_psp_uniform_distribution_flags(psp_flags);
    println!("Set psp_uniform_distribution = FALSE for Area A");

    // Verify synapse count
    let synapse_count = npu.get_synapse_count();
    println!("\nTotal synapses created: {}", synapse_count);
    assert_eq!(synapse_count, 10, "Should have 10 synapses");

    println!("\n=== Burst 1: Fire neuron A ===");
    npu.inject_sensory_with_potentials(&[(neuron_a, 20.0)]);
    let result1 = npu.process_burst().unwrap();
    println!("Burst 1 fired {} neurons", result1.fired_neurons.len());
    println!(
        "Neuron A fired: {}",
        result1.fired_neurons.contains(&neuron_a)
    );
    assert!(
        result1.fired_neurons.contains(&neuron_a),
        "Neuron A should fire from injection"
    );

    println!("\n=== Burst 2: Check Area B propagation ===");
    let result2 = npu.process_burst().unwrap();
    println!("Burst 2 fired {} neurons", result2.fired_neurons.len());

    // Check each neuron in B
    for (i, &neuron) in neurons_b.iter().enumerate() {
        let mp = npu
            .get_neuron_property_by_index(neuron.0 as usize, "membrane_potential")
            .expect("Should have MP");
        let fired = result2.fired_neurons.contains(&neuron);
        println!(
            "  Neuron B[{}] (ID {}): MP={:.2}, Fired={}",
            i, neuron.0, mp, fired
        );
    }

    // EXPECTED BEHAVIOR with psp_uniform=false:
    // - Source has 10 outgoing synapses
    // - PSP = 1, divided by 10 = 1/10 = 0 (integer division!)
    // - Each synapse contributes: weight × psp = 1 × 0 = 0
    // - Each neuron in B should get MP = 0
    // - NO neurons should fire (0 < threshold 10)

    let fired_count = neurons_b
        .iter()
        .filter(|n| result2.fired_neurons.contains(n))
        .count();

    println!("\n=== ANALYSIS ===");
    println!(
        "PSP division: 1 / 10 synapses = {} (integer division)",
        1u8 / 10u8
    );
    println!("Expected contribution per synapse: weight(1) × psp(0) = 0");
    println!("Expected neurons fired in B: 0");
    println!("Actual neurons fired in B: {}", fired_count);

    if fired_count > 0 {
        println!(
            "\n🚨 BUG CONFIRMED: Neurons fired despite PSP division should make contribution = 0!"
        );
    } else {
        println!("\n✅ Correct: No neurons fired (PSP divided to 0)");
    }

    // This should pass, but user reports it fails!
    assert_eq!(
        fired_count, 0,
        "NO neurons in B should fire when PSP is divided to 0"
    );
}
