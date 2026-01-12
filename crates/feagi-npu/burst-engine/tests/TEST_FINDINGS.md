# Comprehensive Neural Dynamics Test Suite - Findings

**Date:** 2025-12-25  
**Test Suite:** `comprehensive_neural_dynamics.rs`  
**Purpose:** Systematically validate synaptic propagation and neural dynamics

## Executive Summary

**Test Results: 7 PASSED, 15 FAILED**

The comprehensive test suite has revealed **critical bugs** in the neural dynamics implementation, particularly around:
1. Membrane potential reset logic with `mp_charge_accumulation=false`
2. Synaptic propagation PSP delivery
3. MP accumulation across bursts

## Test Coverage Matrix

### ✓ Threshold Scenarios
- ✅ PSP < threshold → No fire (PASSED)
- ❌ PSP = threshold → Fire (FAILED)
- ❌ PSP > threshold → Fire (FAILED)

### MP Accumulation
- ✅ `mp_charge_accumulation=false` → Reset each burst (PASSED)
- ❌ `mp_charge_accumulation=true` → Accumulate across bursts (FAILED)

### Leak Coefficient
- ❌ No leak (0.0) → Preserve potential (FAILED - potential is 0 instead of 100!)
- ❌ Partial leak (0.5) → Decay to 50% (FAILED - potential is 0 instead of 50!)
- ✅ Full leak (1.0) → Reset to resting (PASSED)

### PSP Uniformity
- ❌ `psp_uniform_distribution=false` → Divide among synapses (FAILED)
- ❌ `psp_uniform_distribution=true` → Full PSP to each (FAILED)

### Multiple Synapses
- ❌ Multiple from same source (FAILED)
- ❌ Multiple from different sources (FAILED)

### Synapse Types
- ❌ Excitatory (FAILED)
- ✅ Inhibitory (PASSED)
- ❌ Mixed (FAILED)

### Refractory Periods
- ✅ Refractory blocks firing (PASSED)

### Edge Cases
- ✅ Zero weight → No propagation (PASSED)
- ❌ Maximum PSP (255×255) (FAILED)
- ✅ Excitability=0 prevents firing (PASSED)

### Complex Scenarios
- ❌ Chain propagation with delay (FAILED)
- ❌ Convergence-divergence networks (FAILED)
- ❌ Feedback loops (FAILED)

---

## Critical Bugs Discovered

### 🔴 BUG #1: Membrane Potential Reset Issue
**Test:** `test_no_leak_preserves_potential`  
**Expected:** Injected 100, with no leak, MP should remain 100 after 3 bursts  
**Actual:** MP = 0

**Analysis:** The `mp_charge_accumulation=false` reset logic in Phase 1 is resetting the membrane potential **even when it should be preserved**. The bug is that neurons with `mp_acc=false` are being reset to 0 at the START of each burst, but this happens BEFORE checking if they should have accumulated potential from the CURRENT burst.

**Location:** `feagi-npu/burst-engine/src/npu.rs` lines 2306-2319

```rust
// CRITICAL FIX: Reset membrane potentials for neurons with mp_charge_accumulation=false
for idx in 0..neuron_storage.count() {
    if neuron_storage.valid_mask()[idx] && !neuron_storage.mp_charge_accumulation()[idx] {
        // Reset membrane potential for non-accumulating neurons
        neuron_storage.membrane_potentials_mut()[idx] = T::zero();
    }
}
```

**Issue:** This reset happens at the start of Phase 1, but then:
1. Synaptic propagation adds PSP to FCL
2. Neural dynamics (Phase 2) adds FCL to membrane potential
3. But by the time we check the MP (after `process_burst()`), neurons with `mp_acc=false` have their MP intact from Phase 2

**However**, for neurons that DON'T fire, their MP should be preserved if they have `mp_acc=true`, but reset if they have `mp_acc=false` on the NEXT burst.

**The real issue:** The leak test injects directly to the neuron, which should preserve the potential, but `mp_acc=false` is resetting it at the START of the next burst, making it impossible to observe accumulated potential.

### 🔴 BUG #2: Synaptic Propagation Not Delivering PSP
**Tests:** Multiple tests showing synapses not propagating

**Expected:** When neuron A fires, target neuron B should receive PSP in the next burst  
**Actual:** Target neurons not firing despite PSP > threshold

**Possible Causes:**
1. Weight × Conductance calculation incorrect
2. PSP values (u8 range 0-255) not being converted correctly
3. Synaptic propagation engine not finding synapses
4. FCL not accumulating PSP correctly

### 🔴 BUG #3: MP Accumulation Across Bursts Not Working
**Test:** `test_mp_accumulation_true_accumulates_across_bursts`

**Expected:** With `mp_acc=true`, PSP should accumulate: 100 + 100 = 200 >= threshold  
**Actual:** Target doesn't fire even after 2 inputs

**Analysis:** Either:
1. MP is being reset despite `mp_acc=true`
2. Synaptic propagation not working (related to Bug #2)
3. Leak is draining the potential between bursts

---

## Passing Tests Analysis

The 7 passing tests reveal what IS working correctly:

1. ✅ **Threshold checking works**: PSP < threshold correctly prevents firing
2. ✅ **MP reset for `mp_acc=false` works**: Neurons don't accumulate when flag is false
3. ✅ **Excitability=0 works**: Prevents firing probabilistically
4. ✅ **Full leak (1.0) works**: Completely drains potential
5. ✅ **Inhibitory synapses work**: Correctly subtract from potential
6. ✅ **Refractory periods work**: Block firing during countdown
7. ✅ **Zero weight works**: No propagation when weight=0

---

## Root Cause Analysis

The pattern of failures suggests **the synaptic propagation PSP delivery is the primary issue**. Most failing tests involve synaptic connections, while tests that inject directly work.

**Hypothesis:** The issue is in how PSP values are stored/retrieved:
- Genome stores PSP as f32 (e.g., 1.0)
- Code expects u8 values for weight and conductance
- Conversion between these might be incorrect
- OR: PSP uniformity division is happening when it shouldn't

**Action Required:**
1. Trace through `phase1_injection_with_synapses` to see PSP calculation
2. Verify synaptic propagation engine is finding and processing synapses
3. Check FCL accumulation logic
4. Verify PSP uniformity flag handling

---

## Recommendations

### Immediate Actions
1. **Fix synaptic propagation PSP delivery** - This will fix 12+ tests
2. **Review MP reset timing** for `mp_acc=false` neurons
3. **Add debug logging** to synaptic propagation to trace PSP values

### Testing Strategy
1. Run individual failing tests with `--nocapture` to see debug output
2. Add print statements in synaptic_propagation.rs to trace PSP
3. Verify synapse creation is storing correct weight/conductance values

### Long-term
1. Add unit tests for PSP calculation formula
2. Add integration tests for genome→synapse conversion
3. Document expected PSP value ranges (u8 vs f32 confusion)

---

## Test Suite Value

This comprehensive test suite has successfully:
- ✅ Identified critical bugs that affect real-world usage
- ✅ Provided reproducible test cases for debugging
- ✅ Validated what IS working correctly
- ✅ Created a regression test suite for future changes
- ✅ Documented expected behavior systematically

**Conclusion:** The test suite is working exactly as intended - illuminating critical neural dynamics issues that need to be fixed before production use.

