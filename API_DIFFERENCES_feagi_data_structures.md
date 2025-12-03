# API Differences: feagi_data_structures

**Comparison**: Published (0.0.50-beta.59) vs Local (0.0.50-beta.57+changes)  
**Date**: December 3, 2025  
**Impact**: Breaking changes - feagi-core cannot compile with published version

---

## Executive Summary

The local `feagi-data-processing/feagi_data_structures` contains **breaking API changes** that are not published to crates.io yet. The published version (0.0.50-beta.59) exports `IOCorticalAreaDataFlag` while the local version exports `IOCorticalAreaDataType` - these are incompatible.

**Recommendation**: Publish new version of feagi_data_structures before publishing feagi-core.

---

## Critical Breaking Changes

### 1. Enum Renamed

**Published (0.0.50-beta.59)**:
```rust
pub enum IOCorticalAreaDataFlag { ... }
```

**Local**:
```rust
pub enum IOCorticalAreaDataType { ... }
```

**Impact**: All code using `IOCorticalAreaDataFlag` must be updated to `IOCorticalAreaDataType`

---

### 2. Public Export Changed

**File**: `src/genomic/cortical_area/mod.rs`

**Published (0.0.50-beta.59)**:
```rust
pub use io_cortical_area_data_type::{IOCorticalAreaDataFlag};
```

**Local**:
```rust
pub use io_cortical_area_data_type::{IOCorticalAreaDataType};
```

**Impact**: Import statements must change:
```diff
- use feagi_data_structures::genomic::cortical_area::IOCorticalAreaDataFlag;
+ use feagi_data_structures::genomic::cortical_area::IOCorticalAreaDataType;
```

---

### 3. Boolean Variant Removed

**Published (0.0.50-beta.59)**:
```rust
pub enum IOCorticalAreaDataFlag {
    Boolean,           // ← Present
    Percentage(...),
    Percentage2D(...),
    // ... etc
}
```

**Local**:
```rust
pub enum IOCorticalAreaDataType {
    // Boolean removed!
    Percentage(...),
    Percentage2D(...),
    // ... etc
}
```

**Impact**: Code using `IOCorticalAreaDataFlag::Boolean` will break

---

### 4. Bit Field Layout Changed

**Published (0.0.50-beta.59)**:
```rust
// Bits 0-7 -> Enum (8 bits for variant)
// Bit 8 -> FrameChangeHandling
// Bit 9 -> PercentageNeuronPositioning
// Bit 10-15 -> RESERVED

let variant = value & 0xFF;           // Bits 0-7
let frame_handling = (value >> 8) & 0x01;  // Bit 8
let positioning = (value >> 9) & 0x01;     // Bit 9
```

**Local**:
```rust
// Bits 0-3 -> Enum (4 bits for variant)
// Bit 4 -> FrameChangeHandling
// Bit 5 -> PercentageNeuronPositioning

let variant = value & 0x0F;           // Bits 0-3
let frame_handling = (value >> 4) & 0x01;  // Bit 4
let positioning = (value >> 5) & 0x01;     // Bit 5
```

**Impact**: 
- Serialization format incompatibility
- Data saved with old version cannot be read by new version
- Binary protocol breaking change

---

### 5. Variant Numbering Changed

**Published (0.0.50-beta.59)**:
```rust
match variant {
    0 => Ok(IOCorticalAreaDataFlag::Boolean),
    1 => Ok(IOCorticalAreaDataFlag::Percentage(...)),
    2 => Ok(IOCorticalAreaDataFlag::Percentage2D(...)),
    3 => Ok(IOCorticalAreaDataFlag::Percentage3D(...)),
    4 => Ok(IOCorticalAreaDataFlag::Percentage4D(...)),
    5 => Ok(IOCorticalAreaDataFlag::SignedPercentage(...)),
    6 => Ok(IOCorticalAreaDataFlag::SignedPercentage2D(...)),
    7 => Ok(IOCorticalAreaDataFlag::SignedPercentage3D(...)),
    8 => Ok(IOCorticalAreaDataFlag::SignedPercentage4D(...)),
    9 => Ok(IOCorticalAreaDataFlag::CartesianPlane(...)),
    10 => Ok(IOCorticalAreaDataFlag::Misc(...)),
    _ => Err(...)
}
```

**Local**:
```rust
match variant {
    // Boolean removed, all indices shifted down by 1
    0 => Ok(IOCorticalAreaDataType::Percentage(...)),       // was 1
    1 => Ok(IOCorticalAreaDataType::Percentage2D(...)),     // was 2
    2 => Ok(IOCorticalAreaDataType::Percentage3D(...)),     // was 3
    3 => Ok(IOCorticalAreaDataType::Percentage4D(...)),     // was 4
    4 => Ok(IOCorticalAreaDataType::SignedPercentage(...)), // was 5
    5 => Ok(IOCorticalAreaDataType::SignedPercentage2D(...)), // was 6
    6 => Ok(IOCorticalAreaDataType::SignedPercentage3D(...)), // was 7
    7 => Ok(IOCorticalAreaDataType::SignedPercentage4D(...)), // was 8
    8 => Ok(IOCorticalAreaDataType::CartesianPlane(...)),   // was 9
    9 => Ok(IOCorticalAreaDataType::Misc(...)),             // was 10
    _ => Err(...)
}
```

**Impact**: 
- All serialized data will deserialize to wrong variants
- Data corruption risk if mixing versions

---

### 6. Method Removed

**Published (0.0.50-beta.59)**:
```rust
impl IOCorticalAreaDataFlag {
    pub const fn to_data_type_configuration_flag(&self) -> DataTypeConfigurationFlag {
        // ... implementation
    }
}
```

**Local**:
```rust
impl IOCorticalAreaDataType {
    // Method removed!
}
```

**Impact**: Code calling `.to_data_type_configuration_flag()` will break

---

## Additional File Changes

20 files modified in `feagi_data_structures/src/`:

```
src/common_macros.rs
src/shared_enums.rs
src/feagi_signal.rs
src/error.rs
src/lib.rs
src/genomic/motor_cortical_unit.rs
src/genomic/cortical_area/io_cortical_area_data_type.rs  ← Primary change
src/genomic/cortical_area/cortical_id.rs
src/genomic/cortical_area/cortical_area.rs
src/genomic/cortical_area/mod.rs  ← Export change
src/genomic/cortical_area/descriptors.rs
src/genomic/cortical_area/cortical_type.rs
src/genomic/brain_regions/mod.rs
src/genomic/mod.rs
src/genomic/sensory_cortical_unit.rs
src/genomic/descriptors.rs
src/templates/motor_cortical_units.rs
src/templates/motor_types.rs
src/templates/sensor_cortical_units.rs
src/templates/mod.rs
```

---

## Where feagi-core Uses These APIs

**File**: `feagi-core/crates/feagi-types/src/cortical_type_adapter.rs`

```rust
use feagi_data_structures::genomic::cortical_area::{
    CorticalAreaType, CoreCorticalType, CustomCorticalType, MemoryCorticalType,
    IOCorticalAreaDataType,  // ← Uses NEW API
};
use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
```

This import **fails** when using the published version (0.0.50-beta.59) because:
- Published exports: `IOCorticalAreaDataFlag`
- Code expects: `IOCorticalAreaDataType`

---

## Compilation Error

```
error[E0432]: unresolved import `feagi_data_structures::genomic::cortical_area::IOCorticalAreaDataType`
  --> crates/feagi-types/src/cortical_type_adapter.rs:24:5
   |
24 |     IOCorticalAreaDataType,
   |     ^^^^^^^^^^^^^^^^^^^^^^
   |     |
   |     no `IOCorticalAreaDataType` in `genomic::cortical_area`
   |     help: a similar name exists in the module: `IOCorticalAreaDataFlag`
```

---

## Migration Path

### Option 1: Publish feagi_data_structures First (Recommended)

```bash
cd /Users/nadji/code/FEAGI-2.0/feagi-data-processing

# 1. Bump version (current: 0.0.50-beta.57)
# Edit Cargo.toml: version = "0.0.50-beta.60"

# 2. Publish to crates.io
cargo publish

# 3. Update feagi-core
cd /Users/nadji/code/FEAGI-2.0/feagi-core
# Edit Cargo.toml workspace dependencies:
# feagi_data_structures = "0.0.50-beta.60"
# feagi_data_serialization = "0.0.50-beta.60"

# 4. Publish feagi-core
cargo publish
```

### Option 2: Use Path Dependencies (Temporary)

Keep using path dependencies for now:

```toml
# feagi-core/Cargo.toml
[workspace.dependencies]
feagi_data_structures = { path = "../feagi-data-processing/feagi_data_structures" }
feagi_data_serialization = { path = "../feagi-data-processing/feagi_data_serialization" }
```

Update CI workflows to checkout both repos:

```yaml
- name: Checkout feagi-data-processing
  uses: actions/checkout@v4
  with:
    repository: feagi/feagi-data-processing
    path: feagi-data-processing
```

---

## Semantic Versioning Implications

These are **BREAKING CHANGES** that should trigger a major or minor version bump:

**Current**: 0.0.50-beta.59  
**Recommended**:
- **Option A**: `0.1.0-beta.1` (minor bump for breaking changes in 0.x)
- **Option B**: `0.0.51-beta.1` (patch bump + beta reset)
- **Option C**: `0.0.50-beta.60` (continue beta series)

Since you're in beta (0.0.x), any of these are acceptable. Option C is simplest for continuity.

---

## Testing Checklist

Before publishing new version:

- [ ] Verify serialization/deserialization compatibility
- [ ] Update all usages of `IOCorticalAreaDataFlag` → `IOCorticalAreaDataType`
- [ ] Remove/update code using `Boolean` variant
- [ ] Test data migration from old format
- [ ] Update documentation
- [ ] Run full test suite
- [ ] Check dependent projects (feagi-core, brain-visualizer, etc.)

---

## Backward Compatibility

**None** - These are breaking changes. All dependent code must be updated.

**Affected Projects**:
- feagi-core (confirmed - won't compile)
- brain-visualizer (if it uses these APIs)
- Any other projects depending on feagi_data_structures

---

## Recommendation

**Publish feagi_data_structures version 0.0.50-beta.60 before publishing feagi-core.**

**Steps**:
1. Publish feagi_data_structures (2-5 minutes)
2. Update feagi-core dependencies (1 minute)
3. Verify feagi-core compiles (1 minute)
4. Publish feagi-core (2-5 minutes)

**Total time**: ~10 minutes to resolve and publish both

---

## Status

**Current State**:
- Published: feagi_data_structures 0.0.50-beta.59 (OLD API)
- Local: feagi_data_structures 0.0.50-beta.57 + unpublished changes (NEW API)
- feagi-core: Depends on NEW API (not yet published)

**Blocked**: feagi-core cannot be published until feagi_data_structures is updated

---

**Report Generated**: December 3, 2025  
**Author**: AI Development Assistant  
**Priority**: HIGH - Blocks feagi-core publication

