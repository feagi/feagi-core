# Feagi-EVO Compatibility Fixes Needed

This document outlines the compilation errors in `feagi-evo` that need to be fixed to work with the updated `feagi-data-processing` API.

## Summary of Changes in feagi-data-processing

1. **CorticalArea:**
   - ❌ Removed `AreaType` enum → Use `CorticalAreaType` instead
   - ✅ Changed `position: (i32, i32, i32)` → `position: GenomeCoordinate3D`
   - ✅ Using `HashMap<String, Value>` for properties (flexible approach)

2. **BrainRegion:**
   - ❌ Removed `RegionType::Custom` → Use `RegionType::Undefined`
   - ✅ Changed `region_id: String` → `region_id: RegionID` (UUID-based)
   - ✅ Changed `cortical_areas: HashSet<String>` → `HashSet<CorticalID>`
   - ✅ Properties now use struct instead of HashMap

## Required Fixes in feagi-evo

### File: `src/genome/parser.rs`

#### Fix 1: Update Import
**Line 50:**
```rust
// Before:
use feagi_data_structures::genomic::cortical_area::{CorticalAreaDimensions as Dimensions, CorticalArea, AreaType};

// After:
use feagi_data_structures::genomic::cortical_area::{CorticalAreaDimensions as Dimensions, CorticalArea};
// Remove AreaType - it no longer exists
```

#### Fix 2: Convert Position Tuple to GenomeCoordinate3D
**Line 334:**
```rust
// Before:
position,

// After:
GenomeCoordinate3D::new(position.0, position.1, position.2),
// Or if position is (x, y, z):
position.into(), // If Into trait is implemented
```

#### Fix 3: Fix RegionType
**Line 451:**
```rust
// Before:
let region_type = RegionType::Custom;

// After:
let region_type = RegionType::Undefined;
```

#### Fix 4: Convert String to RegionID
**Line 454:**
```rust
// Before:
region_id.clone(),

// After:
RegionID::from_string(&region_id)?, // If parsing from string
// Or:
RegionID::from_string(region_id.as_str())?, 
```

#### Fix 5: Use CorticalID instead of String
**Line 465:**
```rust
// Before:
region.add_area(cortical_id.as_base_64());

// After:
region.add_area(cortical_id); // Use CorticalID directly
```

#### Fix 6: Use Property Setters for BrainRegion
**Line 478, 481, etc:**
```rust
// Before:
region.properties.insert("description".to_string(), serde_json::json!(desc));
region.properties.insert("coordinate_2d".to_string(), serde_json::json!(coord_2d));

// After:
region.set_description(desc);
region.set_coordinate_2d(GenomeCoordinate2D::new(x, y));
```

### File: `src/templates.rs`

#### Fix 1: Update Import
**Line 18:**
```rust
// Before:
use feagi_data_structures::genomic::cortical_area::{CorticalAreaDimensions, CorticalArea, AreaType};

// After:
use feagi_data_structures::genomic::cortical_area::{CorticalAreaDimensions, CorticalArea};
```

#### Fix 2: Convert Position Tuple
**Lines 72, 83:**
```rust
// Before:
(0, 0, -10),

// After:
GenomeCoordinate3D::new(0, 0, -10),
```

#### Fix 3: Fix Properties Assignment
**Lines 71, 93:**
```rust
// Before:
area.properties = props; // where props is HashMap<String, Value>

// After:
// Properties are already HashMap, so direct assignment should work
// But verify the structure matches
```

## Quick Reference: New Types

### Imports Needed
```rust
use feagi_data_structures::genomic::descriptors::{GenomeCoordinate2D, GenomeCoordinate3D};
use feagi_data_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};
use feagi_data_structures::genomic::cortical_area::{CorticalArea, CorticalID, CorticalAreaType};
```

### Type Conversions

#### String → RegionID
```rust
let region_id = RegionID::from_string("root")?;
// Or for UUID strings:
let region_id = RegionID::from_string(uuid_string)?;
```

#### (i32, i32, i32) → GenomeCoordinate3D
```rust
let coord = GenomeCoordinate3D::new(x, y, z);
// Or:
let coord: GenomeCoordinate3D = (x, y, z).into(); // If trait implemented
```

#### String → CorticalID
```rust
// For base64:
let cortical_id = CorticalID::try_from_base_64(base64_string)?;
// For bytes:
let cortical_id = CorticalID::try_from_bytes(&byte_array)?;
```

## Notes

- CorticalArea properties remain `HashMap<String, Value>` for flexibility (see PROPERTIES_STRUCT_MIGRATION_PROPOSAL.md)
- BrainRegion properties now use typed struct - must use setters
- All coordinate positions must use typed coordinate structs
- Region IDs are now UUIDs instead of strings

