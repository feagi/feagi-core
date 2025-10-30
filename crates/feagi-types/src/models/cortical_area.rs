/*!
CorticalArea data model.

Represents a 3D cortical area containing neurons with specific functional roles.
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{FeagiError, Dimensions, Position};

/// Result type for cortical area operations
pub type Result<T> = std::result::Result<T, FeagiError>;

/// Type of cortical area (functional classification)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AreaType {
    /// Sensory input areas
    Sensory,
    /// Motor output areas
    Motor,
    /// Memory/association areas
    Memory,
    /// Custom/user-defined areas
    Custom,
}

impl Default for AreaType {
    fn default() -> Self {
        Self::Custom
    }
}

impl std::fmt::Display for AreaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sensory => write!(f, "sensory"),
            Self::Motor => write!(f, "motor"),
            Self::Memory => write!(f, "memory"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Cortical area metadata (genome representation)
///
/// This struct contains only the static metadata that defines a cortical area.
/// Runtime state (neuron counts, positions) is maintained separately in the
/// NPU and state manager for performance.
///
/// # Design Notes
///
/// - Immutable after creation (use builder pattern for updates)
/// - Lightweight: safe to clone for queries
/// - Serializable: can be saved/loaded from genome files
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalArea {
    /// Unique 6-character identifier
    pub cortical_id: String,

    /// Integer index assigned by ConnectomeManager
    /// Used for fast array indexing in NPU
    pub cortical_idx: u32,

    /// Human-readable name
    pub name: String,

    /// 3D dimensions (width, height, depth in voxels)
    pub dimensions: Dimensions,

    /// 3D position in brain space (can be negative)
    /// This is the origin point (min corner) of the area
    pub position: (i32, i32, i32),

    /// Functional type of this area
    pub area_type: AreaType,
    
    // ========================================================================
    // NEURAL PARAMETERS (Python API compatibility)
    // ========================================================================
    
    /// Is this cortical area visible in visualization?
    #[serde(default = "default_visible")]
    pub visible: bool,
    
    /// Sub-group name for hierarchical organization
    #[serde(default)]
    pub sub_group: Option<String>,
    
    /// Number of neurons per voxel (default: 1)
    #[serde(default = "default_neurons_per_voxel")]
    pub neurons_per_voxel: u32,
    
    /// Postsynaptic current strength
    #[serde(default = "default_postsynaptic_current")]
    pub postsynaptic_current: f64,
    
    /// Plasticity constant for synaptic learning
    #[serde(default = "default_plasticity_constant")]
    pub plasticity_constant: f64,
    
    /// Degeneration rate (0.0 = no degeneration)
    #[serde(default)]
    pub degeneration: f64,
    
    /// Use uniform PSP distribution?
    #[serde(default)]
    pub psp_uniform_distribution: bool,
    
    /// Firing threshold increment per spike
    #[serde(default = "default_firing_threshold_increment")]
    pub firing_threshold_increment: f64,
    
    /// Maximum firing threshold limit
    #[serde(default = "default_firing_threshold_limit")]
    pub firing_threshold_limit: f64,
    
    /// Number of consecutive fires allowed
    #[serde(default = "default_consecutive_fire_count")]
    pub consecutive_fire_count: u32,
    
    /// Snooze period (refractory cooldown) in ticks
    #[serde(default = "default_snooze_period")]
    pub snooze_period: u32,
    
    /// Refractory period (absolute) in ticks
    #[serde(default = "default_refractory_period")]
    pub refractory_period: u32,
    
    /// Leak coefficient for membrane potential decay
    #[serde(default = "default_leak_coefficient")]
    pub leak_coefficient: f64,
    
    /// Leak variability (randomness in leak)
    #[serde(default)]
    pub leak_variability: f64,
    
    /// Is burst engine active for this area?
    #[serde(default = "default_burst_engine_active")]
    pub burst_engine_active: bool,

    /// Additional user-defined properties
    /// Stored as JSON for flexibility
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

// Default value functions for serde
fn default_visible() -> bool {
    true
}

fn default_neurons_per_voxel() -> u32 {
    1
}

fn default_postsynaptic_current() -> f64 {
    1.0
}

fn default_plasticity_constant() -> f64 {
    0.5
}

fn default_firing_threshold_increment() -> f64 {
    0.1
}

fn default_firing_threshold_limit() -> f64 {
    10.0
}

fn default_consecutive_fire_count() -> u32 {
    3
}

fn default_snooze_period() -> u32 {
    5
}

fn default_refractory_period() -> u32 {
    2
}

fn default_leak_coefficient() -> f64 {
    0.01
}

fn default_burst_engine_active() -> bool {
    true
}

impl CorticalArea {
    /// Create a new cortical area with validation and default neural parameters
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Unique 6-character identifier
    /// * `cortical_idx` - Integer index for fast lookups
    /// * `name` - Human-readable name
    /// * `dimensions` - 3D dimensions (width, height, depth)
    /// * `position` - 3D position in brain space
    /// * `area_type` - Functional type
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - cortical_id is not exactly 6 characters
    /// - dimensions are zero
    /// - name is empty
    ///
    pub fn new(
        cortical_id: String,
        cortical_idx: u32,
        name: String,
        dimensions: Dimensions,
        position: (i32, i32, i32),
        area_type: AreaType,
    ) -> Result<Self> {
        // Validate cortical_id (must be 6 characters)
        if cortical_id.len() != 6 {
            return Err(FeagiError::InvalidArea(format!(
                "cortical_id must be exactly 6 characters, got '{}'",
                cortical_id
            )));
        }

        // Validate dimensions (must be > 0)
        if dimensions.width == 0 || dimensions.height == 0 || dimensions.depth == 0 {
            return Err(FeagiError::InvalidArea(format!(
                "dimensions must be > 0, got {:?}",
                dimensions
            )));
        }

        // Validate name
        if name.is_empty() {
            return Err(FeagiError::InvalidArea(
                "name cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            cortical_id,
            cortical_idx,
            name,
            dimensions,
            position,
            area_type,
            // Neural parameters with sensible defaults
            visible: default_visible(),
            sub_group: None,
            neurons_per_voxel: default_neurons_per_voxel(),
            postsynaptic_current: default_postsynaptic_current(),
            plasticity_constant: default_plasticity_constant(),
            degeneration: 0.0,
            psp_uniform_distribution: false,
            firing_threshold_increment: default_firing_threshold_increment(),
            firing_threshold_limit: default_firing_threshold_limit(),
            consecutive_fire_count: default_consecutive_fire_count(),
            snooze_period: default_snooze_period(),
            refractory_period: default_refractory_period(),
            leak_coefficient: default_leak_coefficient(),
            leak_variability: 0.0,
            burst_engine_active: default_burst_engine_active(),
            properties: HashMap::new(),
        })
    }

    /// Create a cortical area with custom properties
    pub fn with_properties(mut self, properties: HashMap<String, serde_json::Value>) -> Self {
        self.properties = properties;
        self
    }

    /// Add a single property
    pub fn add_property(mut self, key: String, value: serde_json::Value) -> Self {
        self.properties.insert(key, value);
        self
    }
    
    // ========================================================================
    // NEURAL PARAMETER SETTERS (Builder Pattern)
    // ========================================================================
    
    /// Set visibility for visualization
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
    
    /// Set sub-group name
    pub fn with_sub_group(mut self, sub_group: Option<String>) -> Self {
        self.sub_group = sub_group;
        self
    }
    
    /// Set neurons per voxel
    pub fn with_neurons_per_voxel(mut self, count: u32) -> Self {
        self.neurons_per_voxel = count;
        self
    }
    
    /// Set postsynaptic current
    pub fn with_postsynaptic_current(mut self, current: f64) -> Self {
        self.postsynaptic_current = current;
        self
    }
    
    /// Set plasticity constant
    pub fn with_plasticity_constant(mut self, constant: f64) -> Self {
        self.plasticity_constant = constant;
        self
    }
    
    /// Set degeneration rate
    pub fn with_degeneration(mut self, rate: f64) -> Self {
        self.degeneration = rate;
        self
    }
    
    /// Set PSP uniform distribution
    pub fn with_psp_uniform_distribution(mut self, uniform: bool) -> Self {
        self.psp_uniform_distribution = uniform;
        self
    }
    
    /// Set firing threshold increment
    pub fn with_firing_threshold_increment(mut self, increment: f64) -> Self {
        self.firing_threshold_increment = increment;
        self
    }
    
    /// Set firing threshold limit
    pub fn with_firing_threshold_limit(mut self, limit: f64) -> Self {
        self.firing_threshold_limit = limit;
        self
    }
    
    /// Set consecutive fire count
    pub fn with_consecutive_fire_count(mut self, count: u32) -> Self {
        self.consecutive_fire_count = count;
        self
    }
    
    /// Set snooze period
    pub fn with_snooze_period(mut self, period: u32) -> Self {
        self.snooze_period = period;
        self
    }
    
    /// Set refractory period
    pub fn with_refractory_period(mut self, period: u32) -> Self {
        self.refractory_period = period;
        self
    }
    
    /// Set leak coefficient
    pub fn with_leak_coefficient(mut self, coefficient: f64) -> Self {
        self.leak_coefficient = coefficient;
        self
    }
    
    /// Set leak variability
    pub fn with_leak_variability(mut self, variability: f64) -> Self {
        self.leak_variability = variability;
        self
    }
    
    /// Set burst engine active state
    pub fn with_burst_engine_active(mut self, active: bool) -> Self {
        self.burst_engine_active = active;
        self
    }

    /// Check if a 3D position is within this area's bounds
    ///
    /// # Arguments
    ///
    /// * `pos` - Absolute position in brain space
    ///
    /// # Returns
    ///
    /// `true` if the position is inside this area, `false` otherwise
    ///
    pub fn contains_position(&self, pos: (i32, i32, i32)) -> bool {
        let (x, y, z) = pos;
        let (ox, oy, oz) = self.position;

        x >= ox
            && y >= oy
            && z >= oz
            && x < ox + self.dimensions.width as i32
            && y < oy + self.dimensions.height as i32
            && z < oz + self.dimensions.depth as i32
    }

    /// Convert absolute brain position to relative position within this area
    ///
    /// # Arguments
    ///
    /// * `pos` - Absolute position in brain space
    ///
    /// # Returns
    ///
    /// Relative position (0,0,0) to (width-1, height-1, depth-1) if inside area
    ///
    /// # Errors
    ///
    /// Returns error if position is outside this area's bounds
    ///
    pub fn to_relative_position(&self, pos: (i32, i32, i32)) -> Result<Position> {
        if !self.contains_position(pos) {
            return Err(FeagiError::OutOfBounds {
                x: pos.0,
                y: pos.1,
                z: pos.2,
                width: self.dimensions.width,
                height: self.dimensions.height,
                depth: self.dimensions.depth,
            });
        }

        let (ox, oy, oz) = self.position;
        Ok((
            pos.0 - ox,
            pos.1 - oy,
            pos.2 - oz,
        ))
    }

    /// Convert relative position within area to absolute brain position
    pub fn to_absolute_position(&self, rel_pos: Position) -> Result<(i32, i32, i32)> {
        if !self.dimensions.contains((rel_pos.0 as u32, rel_pos.1 as u32, rel_pos.2 as u32)) {
            return Err(FeagiError::OutOfBounds {
                x: rel_pos.0,
                y: rel_pos.1,
                z: rel_pos.2,
                width: self.dimensions.width,
                height: self.dimensions.height,
                depth: self.dimensions.depth,
            });
        }

        let (ox, oy, oz) = self.position;
        Ok((
            ox + rel_pos.0 as i32,
            oy + rel_pos.1 as i32,
            oz + rel_pos.2 as i32,
        ))
    }

    /// Get the total number of voxels in this area
    pub fn total_voxels(&self) -> usize {
        self.dimensions.total_voxels()
    }

    /// Get a property value by key
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cortical_area_creation() {
        let area = CorticalArea::new(
            "iav001".to_string(),
            0,
            "Visual Input".to_string(),
            Dimensions::new(128, 128, 20),
            (0, 0, 0),
            AreaType::Sensory,
        )
        .unwrap();

        assert_eq!(area.cortical_id, "iav001");
        assert_eq!(area.name, "Visual Input");
        assert_eq!(area.total_voxels(), 128 * 128 * 20);
    }

    #[test]
    fn test_invalid_cortical_id() {
        let result = CorticalArea::new(
            "short".to_string(), // Too short
            0,
            "Test".to_string(),
            Dimensions::new(10, 10, 10),
            (0, 0, 0),
            AreaType::Custom,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_contains_position() {
        let area = CorticalArea::new(
            "test01".to_string(),
            0,
            "Test Area".to_string(),
            Dimensions::new(10, 10, 10),
            (5, 5, 5),
            AreaType::Custom,
        )
        .unwrap();

        assert!(area.contains_position((5, 5, 5))); // Min corner
        assert!(area.contains_position((14, 14, 14))); // Max corner
        assert!(!area.contains_position((4, 5, 5))); // Outside (x too small)
        assert!(!area.contains_position((15, 5, 5))); // Outside (x too large)
    }

    #[test]
    fn test_position_conversion() {
        let area = CorticalArea::new(
            "test02".to_string(),
            0,
            "Test Area".to_string(),
            Dimensions::new(10, 10, 10),
            (100, 200, 300),
            AreaType::Custom,
        )
        .unwrap();

        // Area spans from (100,200,300) to (109,209,309)
        // Absolute (105, 207, 308) should map to relative (5, 7, 8)
        let rel_pos = area.to_relative_position((105, 207, 308)).unwrap();
        assert_eq!(rel_pos, (5, 7, 8));

        // Convert back
        let abs_pos = area.to_absolute_position(rel_pos).unwrap();
        assert_eq!(abs_pos, (105, 207, 308));

        // Test out of bounds
        let result = area.to_relative_position((99, 200, 300));
        assert!(result.is_err());
    }

    #[test]
    fn test_properties() {
        let area = CorticalArea::new(
            "test03".to_string(),
            0,
            "Test".to_string(),
            Dimensions::new(10, 10, 10),
            (0, 0, 0),
            AreaType::Sensory,
        )
        .unwrap()
        .add_property("resolution".to_string(), serde_json::json!(128))
        .add_property("modality".to_string(), serde_json::json!("visual"));

        assert_eq!(area.get_property("resolution"), Some(&serde_json::json!(128)));
        assert_eq!(
            area.get_property("modality"),
            Some(&serde_json::json!("visual"))
        );
        assert_eq!(area.get_property("nonexistent"), None);
    }
}

