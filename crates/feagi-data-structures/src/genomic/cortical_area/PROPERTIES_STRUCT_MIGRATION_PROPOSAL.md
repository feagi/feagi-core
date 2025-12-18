# Cortical Area Properties Struct Migration Proposal

## Current State

**Status:** Using `HashMap<String, serde_json::Value>` for properties

**Rationale:** 
- Provides maximum flexibility during active development
- Accommodates 25+ different property types currently in use
- Allows rapid experimentation without type system constraints
- Supports arbitrary properties for research/experimentation

**Current Properties in Use:**
- Neural parameters: `synapse_attractivity`, `refractory_period`, `firing_threshold`, `leak_coefficient`, `neuron_excitability`, `postsynaptic_current`, etc.
- Memory parameters: `longterm_mem_threshold`, `lifespan_growth_rate`, `init_lifespan`, `temporal_depth`, etc.
- Configuration: `cortical_group`, `group_id`, `sub_group_id`, `neurons_per_voxel`, `cortical_mapping_dst`, `2d_coordinate`, `visualization`, etc.
- Plus arbitrary properties for future extensions

## Problem Statement

Each cortical area type has different property requirements:
- **IPU/OPU (Input/Output)**: Group IDs, coordinate mappings, IO-specific configuration
- **Memory Areas**: Memory-specific parameters like lifespan, temporal depth, long-term thresholds
- **Custom Areas**: Neuron model-specific parameters (varies by model type)
- **Core Areas**: Minimal properties (standard behavior)

Additionally, we need to support multiple custom neuron models in the future:
- Leaky Integrate-and-Fire (current)
- Izhikevich (future)
- Hodgkin-Huxley (future)
- Other research models

**Current HashMap approach lacks:**
- Type safety (typos in property names won't be caught)
- Validation (invalid values possible)
- Self-documentation (what properties are valid for each type?)
- Compile-time guarantees

## Proposed Solution: Type-Specific Properties with Shared Fields

### Architecture Overview

Use an enum-based property system where:
1. **Common properties** are shared across all cortical types
2. **Type-specific properties** are grouped by cortical area type
3. **Neuron model variants** are extensible within custom areas

### Design Pattern: Enum-Based Properties

```rust
/// Cortical area properties - type-safe based on cortical area type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "property_type", content = "properties")]
pub enum CorticalAreaProperties {
    /// Properties for core areas (Death, Power)
    Core(CoreAreaProperties),
    
    /// Properties for custom neuron models
    Custom(CustomAreaProperties),
    
    /// Properties for memory areas
    Memory(MemoryAreaProperties),
    
    /// Properties for input/output areas
    IO(IOAreaProperties),
}
```

### Common Properties

Properties shared across all cortical area types:

```rust
/// Common properties shared across all cortical area types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommonAreaProperties {
    /// Number of neurons per voxel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub neurons_per_voxel: Option<u32>,
    
    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Enable/disable visualization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visualization: Option<bool>,
}
```

### Type-Specific Property Structs

#### Core Area Properties

```rust
/// Properties specific to core cortical areas (Death, Power)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreAreaProperties {
    #[serde(flatten)]
    pub common: CommonAreaProperties,
    // Minimal - core areas have standard behavior
}
```

#### Custom Area Properties (Extensible for Multiple Neuron Models)

```rust
/// Properties for custom neuron models
/// Extensible to support multiple neuron model variants
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomAreaProperties {
    #[serde(flatten)]
    pub common: CommonAreaProperties,
    
    /// Neuron model-specific parameters
    pub model: CustomNeuronModel,
}

/// Different custom neuron models with their specific parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "model_type")]
pub enum CustomNeuronModel {
    /// Leaky Integrate-and-Fire neuron model
    LeakyIntegrateFire {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        firing_threshold: Option<f64>,
        
        #[serde(default, skip_serializing_if = "Option::is_none")]
        leak_coefficient: Option<f64>,
        
        #[serde(default, skip_serializing_if = "Option::is_none")]
        refractory_period: Option<u32>,
        
        #[serde(default, skip_serializing_if = "Option::is_none")]
        neuron_excitability: Option<f64>,
        
        #[serde(default, skip_serializing_if = "Option::is_none")]
        postsynaptic_current: Option<f64>,
        
        #[serde(default, skip_serializing_if = "Option::is_none")]
        postsynaptic_current_max: Option<f64>,
        
        #[serde(default, skip_serializing_if = "Option::is_none")]
        synapse_attractivity: Option<f64>,
        
        #[serde(default, skip_serializing_if = "Option::is_none")]
        degeneration: Option<f64>,
        
        #[serde(default, skip_serializing_if = "Option::is_none")]
        psp_uniform_distribution: Option<bool>,
        
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mp_charge_accumulation: Option<bool>,
        
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mp_driven_psp: Option<bool>,
    },
    
    /// Izhikevich neuron model (future)
    // Izhikevich {
    //     a: f64,
    //     b: f64,
    //     c: f64,
    //     d: f64,
    //     // ... other Izhikevich-specific parameters
    // },
    
    /// Hodgkin-Huxley model (future)
    // HodgkinHuxley {
    //     g_na: f64,      // Sodium conductance
    //     g_k: f64,       // Potassium conductance
    //     g_l: f64,       // Leak conductance
    //     // ... other HH-specific parameters
    // },
}
```

#### Memory Area Properties

```rust
/// Properties specific to memory cortical areas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryAreaProperties {
    #[serde(flatten)]
    pub common: CommonAreaProperties,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longterm_mem_threshold: Option<u32>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifespan_growth_rate: Option<f64>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init_lifespan: Option<u32>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_depth: Option<u32>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_mem_type: Option<bool>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consecutive_fire_cnt_max: Option<u32>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snooze_length: Option<u32>,
}
```

#### IO Area Properties

```rust
/// Properties specific to input/output cortical areas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IOAreaProperties {
    #[serde(flatten)]
    pub common: CommonAreaProperties,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cortical_group: Option<String>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u8>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_group_id: Option<String>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinate_2d: Option<GenomeCoordinate2D>,
    
    /// Destination mapping for cortical connections
    /// Complex nested structure - may need custom serialization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cortical_mapping_dst: Option<serde_json::Value>,
}
```

## Key Benefits

### 1. Type Safety ✓
- Cannot set memory properties on IO areas
- Compiler enforces correct property usage
- Typos in property names caught at compile time

### 2. Extensibility ✓
- Easy to add new neuron models (Izhikevich, Hodgkin-Huxley, etc.)
- Each model can have its own parameter set
- Adding new models is a matter of adding enum variants

### 3. Self-Documentation ✓
- Properties grouped by cortical type
- Clear what properties apply to each type
- IDE autocomplete shows valid properties

### 4. Pattern Matching ✓
```rust
match &area.properties {
    CorticalAreaProperties::Custom(custom) => {
        match &custom.model {
            CustomNeuronModel::LeakyIntegrateFire { firing_threshold, .. } => {
                // Access LIF-specific parameters
                if let Some(threshold) = firing_threshold {
                    // Use threshold
                }
            }
            // Future models can be added here
        }
    }
    CorticalAreaProperties::Memory(mem) => {
        // Access memory-specific parameters
        if let Some(lifespan) = mem.init_lifespan {
            // Use lifespan
        }
    }
    _ => {}
}
```

### 5. Validation ✓
- Can add per-type validation in constructors
- Type system enforces correctness
- Invalid combinations prevented at compile time

### 6. Future-Proof ✓
- Adding Izhikevich model: just add new variant to `CustomNeuronModel`
- Adding Hodgkin-Huxley: same pattern
- No breaking changes to existing code

## Usage Examples

### Creating a Custom LIF Area

```rust
let lif_props = CustomAreaProperties {
    common: CommonAreaProperties {
        neurons_per_voxel: Some(4),
        visualization: Some(true),
        description: Some("Custom LIF neuron area".to_string()),
        ..Default::default()
    },
    model: CustomNeuronModel::LeakyIntegrateFire {
        firing_threshold: Some(0.5),
        leak_coefficient: Some(0.1),
        refractory_period: Some(5),
        neuron_excitability: Some(1.0),
        postsynaptic_current: Some(0.8),
        synapse_attractivity: Some(0.6),
        ..Default::default()
    },
};

let area = CorticalArea::new(
    cortical_id,
    idx,
    "Custom LIF Area".to_string(),
    dims,
    position,
    CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
    CorticalAreaProperties::Custom(lif_props),
)?;
```

### Creating a Memory Area

```rust
let memory_props = MemoryAreaProperties {
    common: CommonAreaProperties {
        neurons_per_voxel: Some(1),
        visualization: Some(true),
        ..Default::default()
    },
    longterm_mem_threshold: Some(1000),
    lifespan_growth_rate: Some(0.01),
    init_lifespan: Some(100),
    temporal_depth: Some(5),
    is_mem_type: Some(true),
    consecutive_fire_cnt_max: Some(10),
    snooze_length: Some(20),
};

let area = CorticalArea::new(
    cortical_id,
    idx,
    "Memory Storage Area".to_string(),
    dims,
    position,
    CorticalAreaType::Memory(MemoryCorticalType::Memory),
    CorticalAreaProperties::Memory(memory_props),
)?;
```

### Creating an IO Area

```rust
let io_props = IOAreaProperties {
    common: CommonAreaProperties {
        neurons_per_voxel: Some(1),
        visualization: Some(true),
        ..Default::default()
    },
    cortical_group: Some("vision_input".to_string()),
    group_id: Some(5),
    sub_group_id: Some("left_eye".to_string()),
    coordinate_2d: Some(GenomeCoordinate2D::new(10, 20)),
    cortical_mapping_dst: Some(serde_json::json!({
        "target_area": "visual_cortex",
        "connections": []
    })),
};

let area = CorticalArea::new(
    cortical_id,
    idx,
    "Visual Input Area".to_string(),
    dims,
    position,
    CorticalAreaType::BrainInput(io_data_flag),
    CorticalAreaProperties::IO(io_props),
)?;
```

## Migration Strategy

### Phase 1: Current State (HashMap)
- **Status:** ✅ Current implementation
- Use `HashMap<String, serde_json::Value>` for maximum flexibility
- Document property keys in code comments

### Phase 2: Preparation (Future)
- Identify all property keys in use across codebase
- Categorize properties by cortical type
- Document property semantics and validation rules
- Create property key constants to reduce typos

### Phase 3: Struct Implementation (Future)
- Implement struct-based properties as proposed
- Add conversion helpers: `HashMap -> Struct` for migration
- Support both formats temporarily for backward compatibility
- Update genome parser to use struct-based properties

### Phase 4: Full Migration (Future)
- Remove HashMap support
- All code uses struct-based properties
- Full type safety and validation

## Implementation Notes

### Serialization Considerations

1. **Enum Tagging:** Use `#[serde(tag = "property_type")]` to preserve type information in JSON
   ```json
   {
     "property_type": "Custom",
     "properties": {
       "model_type": "LeakyIntegrateFire",
       "firing_threshold": 0.5,
       ...
     }
   }
   ```

2. **Backward Compatibility:** May need custom deserializer to read old HashMap format during migration

3. **Optional Fields:** All fields optional to support partial specification and gradual migration

### Validation Strategy

1. **Per-Type Validators:** Each property struct can implement validation logic
   ```rust
   impl CustomAreaProperties {
       pub fn validate(&self) -> Result<(), FeagiDataError> {
           if let CustomNeuronModel::LeakyIntegrateFire { firing_threshold, .. } = &self.model {
               if let Some(threshold) = firing_threshold {
                   if *threshold < 0.0 || *threshold > 1.0 {
                       return Err(FeagiDataError::BadParameters(
                           "firing_threshold must be between 0.0 and 1.0".to_string()
                       ));
                   }
               }
           }
           Ok(())
       }
   }
   ```

2. **Builder Pattern:** Optional builder for complex property construction
   ```rust
   let props = CustomAreaProperties::builder()
       .neurons_per_voxel(4)
       .firing_threshold(0.5)
       .leak_coefficient(0.1)
       .build()?;
   ```

## Future Neuron Models

### Adding Izhikevich Model

```rust
// Just add to CustomNeuronModel enum:
Izhikevich {
    a: f64,           // Recovery time constant
    b: f64,           // Sensitivity of recovery variable
    c: f64,           // After-spike reset value of v
    d: f64,           // After-spike reset of u
    threshold: f64,   // Spike threshold
    // ... other Izhikevich parameters
},
```

### Adding Hodgkin-Huxley Model

```rust
// Add to CustomNeuronModel enum:
HodgkinHuxley {
    g_na: f64,        // Sodium conductance (max)
    g_k: f64,         // Potassium conductance (max)
    g_l: f64,         // Leak conductance
    e_na: f64,        // Sodium reversal potential
    e_k: f64,         // Potassium reversal potential
    e_l: f64,         // Leak reversal potential
    cm: f64,          // Membrane capacitance
    // ... other HH parameters
},
```

## Conclusion

This proposal provides a path from the current flexible HashMap approach to a strongly-typed, extensible struct-based system that:

1. ✅ Maintains current flexibility (HashMap for now)
2. ✅ Provides clear migration path to type safety
3. ✅ Supports multiple neuron models extensibly
4. ✅ Groups properties logically by cortical type
5. ✅ Enables compile-time validation and documentation

**Next Steps:**
1. Continue using HashMap for current development
2. Document all property keys currently in use
3. Plan struct migration timeline when codebase stabilizes
4. Implement struct-based system when ready for type safety benefits

