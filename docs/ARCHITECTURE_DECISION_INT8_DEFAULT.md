# Architecture Decision: INT8 as Default Quantization

**Date**: November 4, 2025  
**Status**: ✅ **IMPLEMENTED**  
**Impact**: 🔴 **HIGH** - Changes default behavior

---

## Decision

**FEAGI now defaults to INT8 quantization** instead of FP32 when `quantization_precision` is missing from a genome.

---

## Rationale

### Memory Efficiency

| Precision | Memory per 100 Neurons | Relative |
|-----------|------------------------|----------|
| **FP32** | 4.8 KB | Baseline |
| **INT8** | 2.8 KB | **42% reduction** |

**Benefits**:
- **ESP32**: 2x neuron capacity (6,600 → 11,400 neurons)
- **DGX H100**: 2x capacity (1-2B → 2-4B neurons)
- **Desktop**: More efficient memory usage

### Performance

- **Computation**: INT8 is faster on modern hardware (SIMD, Tensor Cores)
- **Memory bandwidth**: 4x less data transfer vs FP32
- **Cache efficiency**: More neurons fit in L1/L2 cache

### Future-Proofing

- Hardware trend towards INT8/INT4 acceleration (NPUs, edge devices)
- FEAGI's target platforms (ESP32, embedded) benefit most from quantization
- Aligns with modern ML inference practices

---

## Implementation

### Code Changes

**1. Runtime Physiology Default**:
```rust
// feagi-evolutionary/src/runtime.rs
pub fn default_quantization_precision() -> String {
    "int8".to_string()  // Changed from "fp32"
}
```

**2. QuantizationSpec Default**:
```rust
// feagi-types/src/numeric.rs
impl Default for QuantizationSpec {
    fn default() -> Self {
        Self {
            precision: Precision::INT8,  // Changed from FP32
            ...
        }
    }
}
```

**3. Validator Auto-Fix**:
```rust
// feagi-evolutionary/src/validator.rs
// Now auto-fixes missing quantization_precision to "int8" (was "fp32")
```

**4. Error Handling**:
```rust
// feagi-brain-development/src/neuroembryogenesis.rs
// Parse errors now default to INT8 (was FP32)
```

---

## Impact Analysis

### Backward Compatibility

⚠️ **BREAKING CHANGE** for genomes without explicit `quantization_precision`:

**Before**:
```json
{
  "physiology": {
    // quantization_precision missing
  }
}
// → Defaulted to FP32
```

**After**:
```json
{
  "physiology": {
    // quantization_precision missing
  }
}
// → Defaults to INT8 (NEW!)
```

**Migration Path**:
- Genomes that want FP32 must explicitly specify: `"quantization_precision": "fp32"`
- Existing genomes without the field will get INT8 (memory efficient)
- Genome validator auto-fixes missing fields to INT8

### Who Is Affected?

1. **New Users**: Get INT8 by default (optimal for most use cases)
2. **Existing Users without field**: Automatic upgrade to INT8 (may notice different behavior)
3. **Existing Users with explicit FP32**: No change (field is honored)
4. **ESP32 Users**: Immediate benefit (2x capacity)
5. **HPC Users**: Immediate benefit (2x capacity on large-scale brains)

---

## Testing Strategy

### Phase 6 Testing (Current)

- [ ] Verify INT8 genome → INT8 connectome (end-to-end)
- [ ] Verify missing field → INT8 (auto-fix)
- [ ] Verify explicit FP32 → FP32 (override)
- [ ] Accuracy validation (INT8 vs FP32 firing patterns >85% similar)
- [ ] Memory measurements (confirm 42% reduction)
- [ ] Performance benchmarks

### Rollback Plan

If INT8 accuracy is insufficient:
1. Change defaults back to FP32
2. Update validator
3. Document INT8 as opt-in
4. Continue Phase 6 tuning

---

## User Communication

### Documentation Updates Needed

1. **Genome Reference**: Document `quantization_precision` field
2. **Migration Guide**: How to explicitly request FP32
3. **Performance Guide**: When to use each precision
4. **Changelog**: Highlight this breaking change

### Recommended User Guidance

**When to use FP32**:
- Maximum numerical accuracy required
- Debugging numerical issues
- Research/validation work
- Plenty of memory available

**When to use INT8** (default):
- Memory-constrained environments (ESP32, edge devices)
- Large-scale brains (>1M neurons)
- Production deployments
- Performance-critical applications

**When to use FP16** (future):
- Balance between FP32 accuracy and INT8 efficiency
- GPU acceleration (Tensor Cores optimized for FP16)
- Medium-scale brains (100K-1M neurons)

---

## Risks & Mitigation

### Risk 1: Accuracy Degradation

**Risk**: INT8 may have <85% firing pattern similarity vs FP32

**Mitigation**:
- Phase 6 includes extensive accuracy tuning
- Quantization range optimization
- Saturation detection
- If insufficient, rollback to FP32 default

**Status**: Phase 6 in progress

---

### Risk 2: User Confusion

**Risk**: Users may not understand why behavior changed

**Mitigation**:
- Clear logging: "Using INT8 quantization (default)"
- Documentation updates
- Migration guide
- Changelog announcement

**Status**: Documentation in Phase 7

---

### Risk 3: Unexpected Bugs

**Risk**: INT8 path may have undiscovered issues

**Mitigation**:
- Comprehensive testing in Phase 6
- Gradual rollout (opt-in initially, then default)
- Monitoring and telemetry
- Easy override (`"quantization_precision": "fp32"`)

**Status**: Testing in Phase 6

---

## Success Criteria

- [ ] INT8 accuracy >85% vs FP32
- [ ] Memory reduction 40-50% confirmed
- [ ] Performance equal or better than FP32
- [ ] All tests passing
- [ ] Documentation complete
- [ ] No user complaints about accuracy

---

## Timeline

- **November 4, 2025**: Decision made, defaults changed
- **November 4-7, 2025**: Phase 6 integration and testing
- **November 8-12, 2025**: Phase 7 documentation
- **Future**: Monitor user feedback, tune as needed

---

## Alternatives Considered

### Alternative 1: Keep FP32 as Default

**Pros**: Zero breaking changes, safer  
**Cons**: Users miss out on memory efficiency by default  
**Decision**: Rejected - memory efficiency is core to FEAGI's mission

### Alternative 2: Make Quantization Mandatory

**Pros**: Forces users to make explicit choice  
**Cons**: Poor UX, breaks existing genomes without the field  
**Decision**: Rejected - sensible defaults are better

### Alternative 3: Dynamic Selection Based on Platform

**Pros**: ESP32 gets INT8, desktop gets FP32 automatically  
**Cons**: Non-deterministic, harder to test, platform coupling  
**Decision**: Rejected - genome should fully specify brain behavior

---

## References

- **Quantization Implementation Checklist**: `QUANTIZATION_IMPLEMENTATION_CHECKLIST.md`
- **Phase 4 Complete**: `QUANTIZATION_PHASE_4_COMPLETE.md`
- **Phase 5 Complete**: `QUANTIZATION_PHASE_5_COMPLETE.md`
- **Issues Log**: `QUANTIZATION_ISSUES_LOG.md`

---

## Approval

**Architect**: Approved (User decision, November 4, 2025)  
**Implementation**: ✅ Complete  
**Testing**: 🔵 In Progress (Phase 6)  
**Documentation**: ⏳ Pending (Phase 7)

---

**Decision Status**: ✅ **IMPLEMENTED**  
**Rollback Plan**: Available (change defaults back to FP32)  
**Monitoring**: Phase 6 will validate decision

---

*Last Updated: November 4, 2025*  
*Document Status: Active*


