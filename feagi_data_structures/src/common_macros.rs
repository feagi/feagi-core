//region Index / Count

/// Creates a strongly-typed index wrapper around an integer type.
/// 
/// # Example
/// ```
/// use feagi_data_structures::define_index;
/// 
/// define_index!(NodeId, u32, "Unique identifier for a node");
/// 
/// let id = NodeId::from(42);
/// assert_eq!(*id, 42);
/// let raw: u32 = id.into();
/// assert_eq!(raw, 42);
/// ```
#[macro_export]
macro_rules! define_index {
    ($name:ident, $inner:ty, $doc:expr) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord
        )]
        pub struct $name($inner);

        impl $name {

            // const constructor
            pub const fn from(var: $inner) -> Self {
                Self(var)
            }

            // const return method
            pub const fn get(&self) -> $inner {
                self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                $name(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

/// Creates a non-zero count type with validation.
///
/// # Example
/// ```
/// use feagi_data_structures::{define_nonzero_count, FeagiDataError};
///
/// define_nonzero_count!(ItemCount, u32, "Number of items (must be > 0)");
///
/// let count = ItemCount::new(5).unwrap();
/// assert_eq!(*count, 5);
///
/// let invalid = ItemCount::new(0);
/// assert!(invalid.is_err());
/// ```
#[macro_export]
macro_rules! define_nonzero_count {
    ($name:ident, $base:ty, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name {
            value: $base,
        }

        impl $name {
            /// Creates a new instance, returns Err if validation fails
            pub fn new(value: $base) -> Result<Self, FeagiDataError> {
                if value == 0 {
                    return Err(FeagiDataError::BadParameters("Count cannot be zero!".into()));
                }
                Ok($name{
                    value,
                })
            }
        }
        impl TryFrom<$base> for $name {
            type Error = FeagiDataError;
            fn try_from(value: $base) -> Result<Self, FeagiDataError> {
                $name::new({value})
            }
        }

        impl From<$name> for $base {
            fn from(value: $name) -> $base {
                value.value
            }
        }

        impl std::ops::Deref for $name {
            type Target = $base;
            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.value.fmt(f)
            }
        }

    }
}

// endregion

//region XY

/// Creates a 2D coordinate type with x,y fields.
/// 
/// # Example
/// ```
/// use feagi_data_structures::define_xy_coordinates;
/// 
/// define_xy_coordinates!(Point2D, i32, "Point2D", "A 2D point with integer coordinates");
/// 
/// let point = Point2D::new(10, 20);
/// assert_eq!(point.x, 10);
/// assert_eq!(point.y, 20);
/// println!("{}", point); // Point2D(10, 20)
/// ```
#[macro_export]
macro_rules! define_xy_coordinates {
    ($name:ident, $var_type:ty, $friendly_name:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
        pub struct $name {
            pub x: $var_type,
            pub y: $var_type,
        }

        impl $name {
            pub fn new(x: $var_type, y: $var_type) -> Self {
                Self { x, y }
            }
        }

        impl From<$name> for ($var_type, $var_type) {
            fn from(value: $name) -> Self {
                (value.x, value.y)
            }
        }

        impl From<($var_type, $var_type)> for $name {
            fn from(value: ($var_type, $var_type)) -> Self {
                $name::new(value.0, value.1)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({}, {})", $friendly_name, self.x, self.y)
            }
        }

    };
}

/// Creates a 2D dimension type with width,height fields and validation.
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_xy_dimensions, FeagiDataError};
/// 
/// define_xy_dimensions!(Size2D, u32, "Size2D", 0, "A 2D size with positive dimensions");
/// 
/// let size = Size2D::new(640, 480).unwrap();
/// assert_eq!(size.width, 640);
/// assert_eq!(size.height, 480);
/// 
/// let invalid = Size2D::new(0, 480);
/// assert!(invalid.is_err());
/// ```
#[macro_export]
macro_rules! define_xy_dimensions {
    ($name:ident, $var_type:ty, $friendly_name:expr, $invalid_zero_value:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Copy, Hash, Eq)]
        pub struct $name {
            pub width: $var_type,
            pub height: $var_type,
        }

        impl $name {
            pub fn new(x: $var_type, y: $var_type) -> Result<Self, FeagiDataError> {
                if x == $invalid_zero_value || y == $invalid_zero_value {
                    return Err(FeagiDataError::BadParameters(format!("Value cannot be {:?} in a {:?}!", $invalid_zero_value, $friendly_name)));
                }
                Ok(Self { width: x, height: y })
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{}, {}>", $friendly_name, self.width, self.height)
            }
        }

        impl From<$name> for ($var_type, $var_type) {
            fn from(value: $name) -> Self {
                (value.width, value.height)
            }
        }

        impl TryFrom<($var_type, $var_type)> for $name {
            type Error = FeagiDataError;
            fn try_from(value: ($var_type, $var_type)) -> Result<Self, Self::Error> {
                if value.0 == $invalid_zero_value {
                    return Err(FeagiDataError::BadParameters(format!("X value cannot be zero!")));
                }
                if value.1 == $invalid_zero_value {
                    return Err(FeagiDataError::BadParameters(format!("Y value cannot be zero!")));
                }
                Ok(Self { width: value.0, height: value.1 })
            }
        }

    }
}

//endregion

//region XYZ

/// Creates a 3D coordinate type with x,y,z fields.
/// 
/// # Example
/// ```
/// use feagi_data_structures::define_xyz_coordinates;
/// 
/// define_xyz_coordinates!(Point3D, u32, "Point3D", "A 3D point with u32 coordinates");
/// 
/// let point = Point3D::new(1, 2, 3);
/// assert_eq!(point.x, 1);
/// assert_eq!(point.y, 2);
/// assert_eq!(point.z, 3);
/// println!("{}", point); // Point3D(1.0, 2.0, 3.0)
/// ```
#[macro_export]
macro_rules! define_xyz_coordinates {
    ($name:ident, $var_type:ty, $friendly_name:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
        pub struct $name {
            pub x: $var_type,
            pub y: $var_type,
            pub z: $var_type,
        }

        impl $name {
            pub fn new(x: $var_type, y: $var_type, z: $var_type) -> Self {
                Self { x, y, z }
            }
        }

        impl From<$name> for ($var_type, $var_type, $var_type) {
            fn from(value: $name) -> Self {
                (value.x, value.y, value.z)
            }
        }

        impl From<($var_type, $var_type, $var_type)> for $name {
            fn from(value: ($var_type, $var_type, $var_type)) -> Self {
                $name::new(value.0, value.1, value.2)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({}, {}, {})", $friendly_name, self.x, self.y, self.z)
            }
        }

    };
}

/// Creates a 3D dimension type with width,height,depth fields and validation.
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_xyz_dimensions, FeagiDataError};
/// 
/// define_xyz_dimensions!(Volume3D, u32, "Volume3D", 0, "A 3D volume with positive dimensions");
/// 
/// let vol = Volume3D::new(10, 20, 30).unwrap();
/// assert_eq!(vol.width, 10);
/// assert_eq!(vol.height, 20);
/// assert_eq!(vol.depth, 30);
/// assert_eq!(vol.number_elements(), 6000);
/// 
/// let invalid = Volume3D::new(0, 20, 30);
/// assert!(invalid.is_err());
/// ```
#[macro_export]
macro_rules! define_xyz_dimensions {
    ($name:ident, $var_type:ty, $friendly_name:expr, $invalid_zero_value:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub width: $var_type,
            pub height: $var_type,
            pub depth: $var_type,
        }

        impl $name {
            pub fn new(x: $var_type, y: $var_type, z: $var_type) -> Result<Self, FeagiDataError> {
                if x == $invalid_zero_value || y == $invalid_zero_value || z == $invalid_zero_value {
                    return Err(FeagiDataError::BadParameters(format!("Value cannot be {:?} in a {:?}!", $invalid_zero_value, $friendly_name)));
                }
                Ok(Self { width: x, height: y, depth: z })
            }

            /// Convenience method for creating from tuple (validates)
            pub fn from_tuple(tuple: ($var_type, $var_type, $var_type)) -> Result<Self, FeagiDataError> {
                Self::new(tuple.0, tuple.1, tuple.2)
            }

            /// Convert to tuple
            pub fn to_tuple(&self) -> ($var_type, $var_type, $var_type) {
                (self.width, self.height, self.depth)
            }

            /// Total number of elements (width * height * depth)
            pub fn number_elements(&self) -> $var_type {
                self.width * self.height * self.depth
            }

            /// Alias for number_elements (for compatibility)
            pub fn volume(&self) -> $var_type {
                self.number_elements()
            }

            /// Alias for number_elements (for compatibility)
            pub fn total_voxels(&self) -> $var_type {
                self.number_elements()
            }

            /// Check if a position is within these dimensions
            pub fn contains(&self, pos: ($var_type, $var_type, $var_type)) -> bool {
                pos.0 < self.width && pos.1 < self.height && pos.2 < self.depth
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{}, {}, {}>", $friendly_name, self.width, self.height, self.depth)
            }
        }

        impl From<$name> for ($var_type, $var_type, $var_type) {
            fn from(value: $name) -> Self {
                (value.width, value.height, value.depth)
            }
        }

        impl TryFrom<($var_type, $var_type, $var_type)> for $name {
            type Error = FeagiDataError;
            fn try_from(value: ($var_type, $var_type, $var_type)) -> Result<Self, Self::Error> {
                if value.0 == $invalid_zero_value {
                    return Err(FeagiDataError::BadParameters(format!("X value cannot be zero!")));
                }
                if value.1 == $invalid_zero_value {
                    return Err(FeagiDataError::BadParameters(format!("Y value cannot be zero!")));
                }
                if value.2 == $invalid_zero_value {
                    return Err(FeagiDataError::BadParameters(format!("Z value cannot be zero!")));
                }
                Ok(Self { width: value.0, height: value.1, depth: value.2 })
            }
        }
    }
}

/// Creates bidirectional conversions between two XYZ dimension types.
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_xyz_dimensions, define_xyz_mapping, FeagiDataError};
/// 
/// define_xyz_dimensions!(VolumeA, u32, "VolumeA", 0, "Volume type A");
/// define_xyz_dimensions!(VolumeB, u32, "VolumeB", 0, "Volume type B");
/// define_xyz_mapping!(VolumeA, VolumeB);
/// 
/// let vol_a = VolumeA::new(10, 20, 30).unwrap();
/// let vol_b: VolumeB = vol_a.into();
/// let back_to_a: VolumeA = vol_b.into();
/// assert_eq!(vol_a, back_to_a);
/// ```
#[macro_export]
macro_rules! define_xyz_mapping{
    ($XYZ_a:ident, $XYZ_b:ident) => {
        impl From<$XYZ_a> for $XYZ_b {
            fn from(a: $XYZ_a) -> Self {
                $XYZ_b::new(a.width, a.height, a.depth).unwrap()
            }
        }
        impl From<$XYZ_b> for $XYZ_a {
            fn from(b: $XYZ_b) -> Self {
                $XYZ_a::new(b.width, b.height, b.depth).unwrap()
            }
        }
    }
}

/// Creates a 3D dimension range type for spatial bounds checking.
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_xyz_dimensions, define_xyz_dimension_range, FeagiDataError};
/// 
/// define_xyz_dimensions!(Position3D, u32, "Position3D", 0, "3D position coordinates");
/// define_xyz_dimension_range!(BoundingBox3D, u32, Position3D, "BoundingBox3D", "3D bounding box for spatial queries");
/// 
/// let bounds = BoundingBox3D::new(0..10, 0..20, 0..30).unwrap();
/// let pos = Position3D::new(5, 15, 25).unwrap();
/// assert!(bounds.verify_coordinate_within_range(&pos).is_ok());
/// 
/// let out_of_bounds = Position3D::new(15, 15, 25).unwrap();
/// assert!(bounds.verify_coordinate_within_range(&out_of_bounds).is_err());
/// ```
#[macro_export]
macro_rules! define_xyz_dimension_range {
    ($name:ident, $var_type:ty, $coordinate_type:ty, $friendly_name:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            pub width: std::ops::Range<$var_type>,
            pub height: std::ops::Range<$var_type>,
            pub depth: std::ops::Range<$var_type>
        }

        impl $name {
            /// Creates a new dimension range, ensuring no ranges are empty.
            pub fn new(x: std::ops::Range<$var_type>, y: std::ops::Range<$var_type>, z: std::ops::Range<$var_type>) -> Result<Self, FeagiDataError> {
                Ok($name {width: x, height: y, depth: z})
            }

            /// Verifies that a coordinate falls within all axis ranges.
            pub fn verify_coordinate_within_range(&self, coordinate: &$coordinate_type) -> Result<(), FeagiDataError> {
                if self.width.contains(&coordinate.width) && self.height.contains(&coordinate.height) && self.depth.contains(&coordinate.depth) {
                    return Ok(());
                }
                Err(FeagiDataError::BadParameters(format!("Coordinate {:?} is not contained by this given range of {:?}!", coordinate, self)))

            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{:?}, {:?}, {:?}>", $friendly_name, self.width, self.height, self.depth)
            }
        }
    };
}

//endregion
