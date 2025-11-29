# Sleep Mode Graceful Fallback Behavior

**Date:** 2025-11-26

---

## Overview

Sleep mode now gracefully handles missing genome configuration without errors or crashes. This ensures backward compatibility with existing genomes that don't have the `physiology.sleep` section.

---

## Enable Conditions

Sleep mode is **ONLY active** when **BOTH** of these conditions are true:

1. ✅ **System flag enabled:** `burst_engine.sleep.enabled = true` in `feagi_configuration.toml`
2. ✅ **Genome has sleep config:** Valid `physiology.sleep` section in `genome.json`

---

## Logging Examples

### Case 1: System Disabled

**Config:**
```toml
# feagi_configuration.toml
[burst_engine.sleep]
enabled = false
```

**Log Output:**
```
🛌 Sleep Manager: DISABLED by system configuration (feagi_configuration.toml)
```

**Behavior:** No sleep transitions, stays in `Running` state.

---

### Case 2: Missing Genome Config

**Config:**
```toml
# feagi_configuration.toml
[burst_engine.sleep]
enabled = true
```

**Genome:**
```json
{
  "physiology": {
    "simulation_timestep": 0.06666666666666667,
    "max_age": 10000000
    // NO "sleep" section!
  }
}
```

**Log Output:**
```
⚠️  Sleep Manager: DISABLED - missing 'physiology.sleep' in genome
   Add sleep configuration to genome.json to enable sleep mode
```

**Behavior:** No sleep transitions, stays in `Running` state. Suggests adding config to genome.

---

### Case 3: Fully Enabled

**Config:**
```toml
# feagi_configuration.toml
[burst_engine.sleep]
enabled = true
```

**Genome:**
```json
{
  "physiology": {
    "sleep": {
      "light_sleep_frequency_hz": 10.0,
      "light_sleep_ipu_threshold_neurons": 100,
      "light_sleep_activity_window_bursts": 50,
      "deep_sleep_enabled": true,
      "deep_sleep_frequency_hz": 5.0,
      "deep_sleep_ipu_threshold_neurons": 10,
      "deep_sleep_min_light_sleep_duration_bursts": 500,
      "deep_sleep_compaction_fragmentation_threshold": 0.20,
      "wake_ipu_threshold_neurons": 200,
      "wake_activity_window_bursts": 10
    }
  }
}
```

**Log Output:**
```
🛌 Sleep Manager: ENABLED
   Light sleep frequency: 10 Hz (threshold: 100 neurons/burst over 50 bursts)
   Deep sleep enabled: true (frequency: 5 Hz, threshold: 10 neurons/burst)
   Wake threshold: 200 neurons/burst over 10 bursts
```

**Behavior:** Full sleep functionality active.

---

## Detection Logic

### How Missing Config is Detected

The `SleepConfig::is_valid_from_genome()` method checks for sentinel values:

```rust
pub fn is_valid_from_genome(&self) -> bool {
    // If frequencies are > 0, assume config came from genome
    self.light_sleep_frequency_hz > 0.0 
        && self.deep_sleep_frequency_hz > 0.0
        && self.light_sleep_ipu_threshold_neurons > 0
        && self.wake_ipu_threshold_neurons > 0
}
```

**Rationale:**
- Valid sleep frequencies are always > 0 Hz
- If all are zero, config was never loaded from genome
- Simple, fast, no external state needed

---

## Adding Sleep to Existing Genomes

### Step 1: Edit genome.json

Add this section under `"physiology"`:

```json
{
  "physiology": {
    // ... existing physiology params ...
    "sleep": {
      "light_sleep_frequency_hz": 10.0,
      "light_sleep_ipu_threshold_neurons": 100,
      "light_sleep_activity_window_bursts": 50,
      "deep_sleep_enabled": true,
      "deep_sleep_frequency_hz": 5.0,
      "deep_sleep_ipu_threshold_neurons": 10,
      "deep_sleep_min_light_sleep_duration_bursts": 500,
      "deep_sleep_compaction_fragmentation_threshold": 0.20,
      "wake_ipu_threshold_neurons": 200,
      "wake_activity_window_bursts": 10
    }
  }
}
```

### Step 2: Verify JSON is valid

```bash
python3 -m json.tool genome.json > /dev/null
```

### Step 3: Restart FEAGI

Load the genome and check logs for:
```
🛌 Sleep Manager: ENABLED
```

---

## Parameter Tuning Guide

### Conservative (Less Sleep)
```json
{
  "light_sleep_frequency_hz": 15.0,           // ← Higher frequency (less reduction)
  "light_sleep_ipu_threshold_neurons": 50,    // ← Lower threshold (harder to enter)
  "wake_ipu_threshold_neurons": 100,          // ← Lower threshold (easier to wake)
  "deep_sleep_enabled": false                 // ← Disable deep sleep entirely
}
```

### Aggressive (More Sleep)
```json
{
  "light_sleep_frequency_hz": 5.0,            // ← Lower frequency (more reduction)
  "light_sleep_ipu_threshold_neurons": 200,   // ← Higher threshold (easier to enter)
  "wake_ipu_threshold_neurons": 500,          // ← Higher threshold (harder to wake)
  "deep_sleep_enabled": true,
  "deep_sleep_compaction_fragmentation_threshold": 0.15  // ← Lower threshold (compact sooner)
}
```

### Disable Sleep for Specific Genome
```json
// Simply omit the "sleep" section entirely
{
  "physiology": {
    "simulation_timestep": 0.06666666666666667,
    "max_age": 10000000
    // No "sleep" section = sleep disabled for this genome
  }
}
```

---

## Safety Features

1. **No crashes:** Missing config never causes errors
2. **Clear logging:** Always logs why sleep is disabled
3. **Backward compatible:** Old genomes continue to work
4. **Fail-safe:** Invalid values (0 Hz, etc.) are detected and rejected
5. **Min values enforced:** Activity windows clamped to minimum 1 burst (prevents divide-by-zero)

---

## Testing Checklist

- [ ] System disabled (`enabled = false`) → logs "DISABLED by system configuration"
- [ ] Missing genome config → logs "DISABLED - missing 'physiology.sleep' in genome"
- [ ] Valid config → logs "ENABLED" with parameter summary
- [ ] Invalid values (0 Hz) → detected and disabled
- [ ] Old genomes without sleep section → continue to work normally
- [ ] Essential genome has sleep config → verified ✅

---

## References

- Implementation: `feagi-burst-engine/src/sleep.rs`
- Essential genome: `feagi-evo/genomes/essential_genome.json`
- System config: `feagi_configuration.toml`
- Architecture doc: `SLEEP_MODE_IMPLEMENTATION.md`


