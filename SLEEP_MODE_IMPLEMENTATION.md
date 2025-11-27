# Sleep Mode Implementation

**Status:** ✅ Core infrastructure complete, NPU integration in progress  
**Date:** 2025-11-26

---

## Overview

FEAGI's sleep mode provides energy efficiency and memory optimization during periods of low brain activity. It features two-stage sleep (Light → Deep) with automatic state transitions based on IPU activity.

---

## Architecture

### State Machine

```
[Running] (genome frequency, e.g., 30 Hz)
    │
    │ IPU activity < light_sleep_ipu_threshold_neurons
    │ for light_sleep_activity_window_bursts
    ↓
[Light Sleep] (light_sleep_frequency_hz, e.g., 10 Hz)
    │  - Build lazy free-list (interruptible)
    │  - Reduced burst frequency
    │
    │ Still low activity for deep_sleep_min_light_sleep_duration_bursts
    ↓
[Deep Sleep] (deep_sleep_frequency_hz, e.g., 5 Hz)
    │  - Run memory compaction (NON-INTERRUPTIBLE)
    │  - Further reduced frequency
    │
    │ IPU activity > wake_ipu_threshold_neurons
    │ for wake_activity_window_bursts
    ↓
[Running] (back to normal frequency)
```

---

## Configuration

### System-Level (feagi_configuration.toml)

**Location:** `[burst_engine.sleep]`

```toml
[burst_engine.sleep]
enabled = true  # Master kill switch - disables all sleep functionality
```

**This is the ONLY system-level sleep parameter.** All other parameters come from the genome.

### Graceful Fallback

**Sleep is ONLY enabled if BOTH conditions are met:**
1. ✅ System flag `enabled = true` in `feagi_configuration.toml`
2. ✅ Genome has valid `physiology.sleep` section

**If genome is missing sleep config:**
- Sleep manager logs: `⚠️ Sleep Manager: DISABLED - missing 'physiology.sleep' in genome`
- No state transitions occur (stays in `Running` state)
- No errors or crashes
- Suggestion logged to add sleep config to genome

---

### Genome-Level (genome.json)

**Location:** `physiology.sleep` (new section)

```json
{
  "physiology": {
    "sleep": {
      // Light sleep configuration
      "light_sleep_frequency_hz": 10.0,
      "light_sleep_ipu_threshold_neurons": 100,
      "light_sleep_activity_window_bursts": 50,
      
      // Deep sleep configuration
      "deep_sleep_enabled": true,
      "deep_sleep_frequency_hz": 5.0,
      "deep_sleep_ipu_threshold_neurons": 10,
      "deep_sleep_min_light_sleep_duration_bursts": 500,
      "deep_sleep_compaction_fragmentation_threshold": 0.20,
      
      // Wake conditions
      "wake_ipu_threshold_neurons": 200,
      "wake_activity_window_bursts": 10
    }
  }
}
```

**Parameter Descriptions:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `light_sleep_frequency_hz` | float | 10.0 | Burst frequency during light sleep |
| `light_sleep_ipu_threshold_neurons` | int | 100 | Avg IPU firings/burst to enter light sleep |
| `light_sleep_activity_window_bursts` | int | 50 | Monitor IPU activity over this many bursts |
| `deep_sleep_enabled` | bool | true | Enable/disable deep sleep stage |
| `deep_sleep_frequency_hz` | float | 5.0 | Burst frequency during deep sleep |
| `deep_sleep_ipu_threshold_neurons` | int | 10 | Avg IPU firings/burst to enter deep sleep |
| `deep_sleep_min_light_sleep_duration_bursts` | int | 500 | Must be in light sleep for this many bursts before deep sleep |
| `deep_sleep_compaction_fragmentation_threshold` | float | 0.20 | Only compact memory if fragmentation > this (20% = 0.20) |
| `wake_ipu_threshold_neurons` | int | 200 | Avg IPU firings/burst to wake from sleep |
| `wake_activity_window_bursts` | int | 10 | Monitor wake activity over this many bursts (faster response) |

---

## Sleep Optimizations

### Light Sleep
- **Burst frequency reduction:** Configurable via `light_sleep_frequency_hz`
- **Lazy free-list building:** Scans NPU's `valid_mask` and builds a stack of free neuron slots for O(1) allocation
- **Interruptible:** Can wake immediately without data corruption risk

### Deep Sleep
- **Further frequency reduction:** Configurable via `deep_sleep_frequency_hz`
- **Memory compaction:** Moves active neurons to front, eliminates fragmentation
  - ⚠️ **NON-INTERRUPTIBLE:** Blocks wake signals during compaction to prevent synapse corruption
  - Only runs if `fragmentation > deep_sleep_compaction_fragmentation_threshold`
  - Updates ALL synapse references to new neuron positions

---

## Activity Tracking

### IPU Activity Measurement
- **Metric:** Total number of IPU neurons that fired this burst
- **Window:** Rolling average over configurable number of bursts
- **Separate windows:**
  - Light sleep detection: `light_sleep_activity_window_bursts` (slower, e.g., 50 bursts)
  - Wake detection: `wake_activity_window_bursts` (faster, e.g., 10 bursts)

### Future Wake Conditions (TODO)
- OPU (motor) activity
- API structural changes (add/move/delete cortical areas)
- User-requested pause/stop

---

## Logging

### Phase Transitions
```
🛌 Entering Light Sleep at burst 1234: reducing frequency to 10 Hz
🛌 Building lazy free-list for fast neuron allocation...
🛌 ✅ Lazy free-list built in 15.2ms

🛌💤 Entering Deep Sleep at burst 1785: reducing frequency to 5 Hz
🛌💤 Starting memory compaction: fragmentation=23.5% > threshold=20.0%
🛌💤 ✅ Memory compaction complete in 2.34s (⚠️ was non-interruptible)

⏰ Waking up at burst 2103: resuming normal operation
🛌 Clearing lazy free-list to reclaim memory
```

### Sleep Statistics
```
🛌 Exiting Light Sleep: duration=15.32s, bursts=153, total_light_sleep=245s
🛌💤 Exiting Deep Sleep: duration=45.67s, bursts=228, total_deep_sleep=782s
```

---

## Implementation Files

| File | Purpose |
|------|---------|
| `feagi-burst-engine/src/sleep.rs` | Core sleep manager implementation |
| `feagi-state-manager/src/core_state.rs` | Added `LightSleep` and `DeepSleep` states |
| `feagi-config/src/types.rs` | Added `BurstEngineSleepConfig` |
| `feagi_configuration.toml` | Added `[burst_engine.sleep]` section |
| `genome.json` (physiology section) | Sleep parameters |

---

## TODO: NPU Integration

### Required NPU Changes

1. **Lazy Free-List** (Light Sleep)
   ```rust
   // Add to NeuronArray
   pub struct NeuronArray<T> {
       pub lazy_free_list: Option<Vec<usize>>,  // Built during light sleep
       // ...
   }
   
   impl<T> NeuronArray<T> {
       pub fn build_lazy_free_list(&mut self) {
           let free_slots: Vec<usize> = (0..self.count)
               .filter(|i| !self.valid_mask[*i])
               .collect();
           self.lazy_free_list = Some(free_slots);
       }
       
       pub fn clear_lazy_free_list(&mut self) {
           self.lazy_free_list = None; // Reclaim memory
       }
   }
   ```

2. **Fragmentation Query**
   ```rust
   impl<T> NeuronArray<T> {
       pub fn get_fragmentation(&self) -> f32 {
           if self.count == 0 {
               return 0.0;
           }
           (self.count - self.active_count) as f32 / self.count as f32
       }
   }
   ```

3. **Memory Compaction** (Deep Sleep)
   ```rust
   impl<T> NeuronArray<T> {
       pub fn compact(&mut self, synapse_array: &mut SynapseArray) {
           let mut write_idx = 0;
           let mut id_mapping = HashMap::new();
           
           // Step 1: Move active neurons to front
           for read_idx in 0..self.count {
               if self.valid_mask[read_idx] {
                   if read_idx != write_idx {
                       // Move neuron data
                       self.thresholds[write_idx] = self.thresholds[read_idx];
                       // ... (copy all arrays)
                       id_mapping.insert(read_idx, write_idx);
                   }
                   write_idx += 1;
               }
           }
           
           self.count = write_idx;
           self.active_count = write_idx;
           
           // Step 2: Update ALL synapse references (CRITICAL!)
           for synapse in &mut synapse_array.synapses {
               if let Some(&new_target) = id_mapping.get(&synapse.target_id) {
                   synapse.target_id = new_target;
               }
           }
       }
   }
   ```

---

## Safety Guarantees

1. **Light Sleep:** Always safe to interrupt (incomplete free-list is discarded)
2. **Deep Sleep:** Blocks wake during compaction to prevent synapse corruption
3. **Atomic state transitions:** Uses `BurstEngineState` from `feagi-state-manager`
4. **No data loss:** All sleep operations are either fully complete or fully rolled back

---

## Performance Impact

| State | Frequency | CPU Usage | Memory Usage | Responsiveness |
|-------|-----------|-----------|--------------|----------------|
| Running | 30 Hz | 100% | 100% | Immediate |
| Light Sleep | 10 Hz | ~33% | 100% + free-list (~1 MB) | <100ms wake |
| Deep Sleep | 5 Hz | ~17% | 100% (compacted) | Blocked during compaction |

---

## Testing

### Unit Tests
- Activity tracker rolling window
- Sleep phase timing
- State transition logic

### Integration Tests (TODO)
- Full sleep cycle (Running → Light → Deep → Running)
- Compaction correctness (verify synapse integrity)
- Wake responsiveness under various activity patterns

---

## References

- Architecture Discussion: [Context from 2025-11-26 conversation]
- Scan+Hint NPU Optimization: Option chosen for memory efficiency
- State Manager: `feagi-state-manager` crate

---

## Questions / Decisions Made

1. **Frequency reduction:** Fixed sleep frequency (configurable via genome) ✅
2. **Activity metric:** Total IPU neurons fired (rolling average) ✅
3. **Sleep stages:** Two stages (Light + Deep) ✅
4. **Wake conditions:** IPU activity only (for now) ✅
5. **Compaction safety:** Non-interruptible during compaction ✅
6. **Configuration location:** Genome physiology (except master enable) ✅

