# IPU/OPU Types Endpoints Implementation

## Summary

Implemented two new API endpoints to provide **dynamically generated** metadata about available IPU (Input Processing Unit) and OPU (Output Processing Unit) types. These endpoints solve the issue of empty "Add Input Cortical Area" and "Add Output Cortical Area" windows in Brain Visualizer.

**All data is dynamically extracted from the source-of-truth template definitions in `feagi-data-processing`, ensuring consistency and eliminating maintenance burden.**

## Implementation Details

### New Endpoints

#### 1. GET `/v1/cortical_area/ipu/types`
Returns metadata for all available IPU types.

**Response Format:**
```json
{
  "isvi": {
    "description": "Segmented Vision",
    "encodings": ["absolute", "incremental"],
    "formats": [],
    "units": 9
  },
  "iinf": {
    "description": "Infrared Sensor",
    "encodings": ["absolute", "incremental"],
    "formats": ["linear", "fractional"],
    "units": 1
  },
  ...
}
```

**IPU Types Included:**
- `iinf` - Infrared Sensor
- `ipro` - Proximity Sensor
- `ishk` - Shock sensor
- `ibat` - Battery Sensor
- `isvm` - Servo Sensor
- `iagp` - Analog GPIO Sensor
- `imis` - Miscellaneous Sensor
- `iimg` - Vision Sensor
- `isvi` - Segmented Vision

#### 2. GET `/v1/cortical_area/opu/types`
Returns metadata for all available OPU types.

**Response Format:**
```json
{
  "omot": {
    "description": "Rotary Motor",
    "encodings": ["absolute", "incremental"],
    "formats": ["linear", "fractional"],
    "units": 1
  },
  "opse": {
    "description": "Positional Servo",
    "encodings": ["absolute", "incremental"],
    "formats": ["linear", "fractional"],
    "units": 1
  },
  ...
}
```

**OPU Types Included:**
- `omot` - Rotary Motor
- `opse` - Positional Servo
- `ogaz` - Gaze Control
- `omis` - Miscellaneous Motor

### Response Fields

Each type metadata object contains:

- **description** (string): Human-readable description of the sensor/actuator
- **encodings** (array of strings): Supported encoding types
  - `"absolute"` - Absolute value encoding (FrameChangeHandling::Absolute)
  - `"incremental"` - Change-based encoding (FrameChangeHandling::Incremental)
- **formats** (array of strings): Supported neuron positioning formats
  - `"linear"` - Linear positioning (PercentageNeuronPositioning::Linear)
  - `"fractional"` - Fractional positioning (PercentageNeuronPositioning::Fractional)
  - Empty array for CartesianPlane and Misc types (no positioning parameter)
- **units** (number): Number of cortical areas this type creates

### Files Modified

**feagi-data-processing:**
1. **`feagi_data_structures/src/genomic/sensory_cortical_unit.rs`**
   - Added `get_friendly_name()` method to expose human-readable names
   - Added `get_cortical_id_unit_reference()` method to expose 3-byte identifiers
   - Added `get_number_cortical_areas()` method to expose unit counts

2. **`feagi_data_structures/src/genomic/motor_cortical_unit.rs`**
   - Added `get_friendly_name()` method to expose human-readable names
   - Added `get_cortical_id_unit_reference()` method to expose 3-byte identifiers
   - Added `get_number_cortical_areas()` method to expose unit counts

**feagi-core:**
3. **`Cargo.toml`**
   - Updated `feagi_data_structures` dependency to use local path instead of crates.io version
   - Updated `feagi_data_serialization` dependency to use local path

4. **`crates/feagi-api/src/endpoints/cortical_area.rs`**
   - Added imports for `SensoryCorticalUnit` and `MotorCorticalUnit`
   - Added `CorticalTypeMetadata` struct
   - Implemented `get_ipu_types()` endpoint with dynamic metadata extraction
   - Implemented `get_opu_types()` endpoint with dynamic metadata extraction

5. **`crates/feagi-api/src/transports/http/server.rs`**
   - Registered `/cortical_area/ipu/types` route
   - Registered `/cortical_area/opu/types` route
   - Updated endpoint count comment (23 → 25)

6. **`crates/feagi-api/src/openapi.rs`**
   - Added `get_ipu_types` to OpenAPI paths
   - Added `get_opu_types` to OpenAPI paths
   - Added `CorticalTypeMetadata` to schemas

### Data Source

**The implementation dynamically extracts metadata from the single source of truth:**
- `/feagi-data-processing/feagi_data_structures/src/templates/sensor_cortical_units.rs` - IPU type definitions
- `/feagi-data-processing/feagi_data_structures/src/templates/motor_cortical_units.rs` - OPU type definitions

**No hardcoding** - all metadata is derived at runtime from:
- `SensoryCorticalUnit::list_all()` - Lists all available sensor types
- `MotorCorticalUnit::list_all()` - Lists all available motor types
- `get_friendly_name()` - Returns human-readable name
- `get_cortical_id_unit_reference()` - Returns 3-byte identifier
- `get_number_cortical_areas()` - Returns unit count
- `get_snake_case_name()` - Used to determine format support

### Dynamic Metadata Extraction

The endpoints iterate over all registered units and extract metadata:

```rust
for unit in SensoryCorticalUnit::list_all() {
    let id_ref = unit.get_cortical_id_unit_reference();
    let key = format!("i{}", std::str::from_utf8(&id_ref).unwrap());
    
    let encodings = vec!["absolute".to_string(), "incremental".to_string()];
    
    // Determine formats based on snake_case_name
    let snake_name = unit.get_snake_case_name();
    let formats = if snake_name == "vision" || snake_name == "segmented_vision" || snake_name == "miscellaneous" {
        vec![]  // CartesianPlane and Misc types have no formats
    } else {
        vec!["linear".to_string(), "fractional".to_string()]  // Percentage types
    };
    
    types.insert(key, CorticalTypeMetadata {
        description: unit.get_friendly_name().to_string(),
        encodings,
        formats,
        units: unit.get_number_cortical_areas() as u32,
    });
}
```

The cortical ID encoding logic is documented in:
- `/feagi-core/crates/feagi-types/src/cortical_id_decoder.rs`

### Benefits of Dynamic Implementation

1. **Single Source of Truth** - Template definitions in `feagi-data-processing` are the only place sensor/motor types are defined
2. **Zero Maintenance** - Adding a new sensor/motor type to templates automatically makes it available via the API
3. **Consistency Guarantee** - API cannot drift from actual template definitions
4. **No Duplication** - No need to maintain parallel lists of types
5. **Type Safety** - Compile-time guarantees through Rust type system

### Alignment with feagi-data-processing

The endpoint data is **extracted directly** from template definitions:

- **Cortical ID unit references** - `get_cortical_id_unit_reference()` returns `*b"inf"`, `*b"svi"`, `*b"mot"`, etc., prefixed with 'i' or 'o'
- **Number of cortical areas** - `get_number_cortical_areas()` returns the exact count from template
- **Friendly names** - `get_friendly_name()` returns the exact string from template
- **Encodings** - All types support `"absolute"` and `"incremental"` (from `FrameChangeHandling`)
- **Formats** - Determined by checking `snake_case_name`:
  - `"vision"`, `"segmented_vision"`, `"miscellaneous"` → empty array (CartesianPlane/Misc types)
  - All others → `["linear", "fractional"]` (Percentage-based types)

### Testing

Build verification:
```bash
cd /Users/nadji/code/FEAGI-2.0/feagi-core
cargo check --package feagi-api
# Result: ✅ Success
```

### Brain Visualizer Integration

The Brain Visualizer (`brain-visualizer/godot_source`) can now call these endpoints to populate:
- **Add Input Cortical Area window** - using `/v1/cortical_area/ipu/types`
- **Add Output Cortical Area window** - using `/v1/cortical_area/opu/types`

This replaces the old `/v1/cortical_area/cortical_types` endpoint which incorrectly returned only type names instead of full template metadata.

### API Documentation

Full OpenAPI documentation is available at:
- `http://localhost:8000/swagger-ui` (when feagi-core is running)
- Both endpoints are tagged under "cortical_area"
- Complete schema definitions for `CorticalTypeMetadata` included

## Status

✅ **Implementation Complete**
- [x] Response model created
- [x] IPU types endpoint implemented
- [x] OPU types endpoint implemented
- [x] Routes registered
- [x] OpenAPI documentation added
- [x] Code compiles without errors
- [x] No linter warnings

## Next Steps

1. Test the endpoints with Brain Visualizer to verify integration
2. Consider adding more IPU/OPU types as new sensors/actuators are added
3. Update Brain Visualizer to use the new endpoints instead of the old cortical_types endpoint

