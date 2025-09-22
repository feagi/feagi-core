

//region One Dimensional

/// Creates an unsigned percentage type with value range [0.0, 1.0].
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_unsigned_percentage, FeagiDataError};
/// const EPSILON: f32 = 0.0001;
///
/// define_unsigned_percentage!(Opacity, "Opacity value from 0% to 100%");
/// 
/// let opacity = Opacity::new_from_0_1(0.75).unwrap();
/// assert_eq!(opacity.get_as_0_100(), 75.0);
/// assert_eq!(opacity.get_as_u8(), 191);
/// 
/// let from_u8 = Opacity::new_from_u8_0_255(128).unwrap();
/// assert!(from_u8.get_as_0_100() - 50.196078 < EPSILON);
/// 
/// let invalid = Opacity::new_from_0_1(1.5);
/// assert!(invalid.is_err());
/// ```
#[macro_export]
macro_rules! define_unsigned_percentage {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name {
            value: f32,
        }

        impl $name
        {

            //region Constructors

            pub fn new_from_0_1_unchecked(value: f32) -> Self {
                $name { value }
            }

            pub fn new_from_0_1(value: f32) -> Result<$name, FeagiDataError> {
                if value > 1.0 || value < 0.0 {
                    return Err(FeagiDataError::BadParameters("Percentage Value must be between 0 and 1!".into()));
                }
                Ok($name { value })
            }

            pub fn new_from_interp_m1_1(value: f32) -> Result<$name, FeagiDataError> {
                if value > 1.0 || value < -1.0 {
                    return Err(FeagiDataError::BadParameters("Signed Percentage Value to interp from must be between -1 and 1!".into()));
                }
                Ok($name { value: (value + 1.0) / 2.0 })
            }

            pub fn new_from_interp_m1_1_unchecked(value: f32) -> Self {
                $name { value: (value + 1.0) / 2.0 }
            }

            pub fn new_from_u8_0_255(value: u8) -> Result<$name, FeagiDataError> {
                $name::new_from_0_1(value as f32 / u8::MAX as f32)
            }

            pub fn new_from_0_100(value: f32) -> Result<$name, FeagiDataError> {
                if value > 100.0 || value < 0.0 {
                    return Err(FeagiDataError::BadParameters("Percentage Value must be between 0 and 100!".into()));
                }
                Ok($name { value: value / 100.0 })
            }

            pub fn new_from_linear_interp(value: f32, range: &std::ops::Range<f32>) -> Result<$name, FeagiDataError> {
                if value < range.start || value > range.end {
                    return Err(FeagiDataError::BadParameters(format!("Given value {} is out of range {:?}!", value, range)));
                }
                Ok($name { value: Self::linear_interp(value, range) })

            }

            //endregion

            //region Update

            pub(crate) fn inplace_update(&mut self, value: f32)  {
                self.value = value;
            }

            pub fn inplace_update_from_0_1(&mut self, value: f32) -> Result<(), FeagiDataError> {
                if value > 1.0 || value < 0.0 {
                    return Err(FeagiDataError::BadParameters("Percentage Value must be between 0 and 1!".into()));
                }
                self.value = value;
                Ok(())
            }

            pub fn inplace_update_u8_0_255(&mut self, value: u8) -> Result<(), FeagiDataError> {
                self.inplace_update_from_0_1(value as f32 / u8::MAX as f32)
            }

            pub fn inplace_update_0_100(&mut self, value: f32) -> Result<(), FeagiDataError> {
                if value > 100.0 || value < 0.0 {
                    return Err(FeagiDataError::BadParameters("Percentage Value must be between 0 and 100!".into()));
                }
                self.value = value / 100.0;
                Ok(())
            }

            pub fn inplace_update_linear_interp(&mut self, value: f32, range: &std::ops::Range<f32>) -> Result<(), FeagiDataError> {
                if value < range.start || value > range.end {
                    return Err(FeagiDataError::BadParameters(format!("Given value {} is out of range {:?}!", value, range)));
                }
                self.value = Self::linear_interp(value, range);
                Ok(())
            }

            //endregion

            //region Properties

            pub fn get_as_0_1(&self) -> f32 {
                self.value
            }

            pub fn get_as_u8(&self) -> u8 {
                (self.value * u8::MAX as f32) as u8
            }

            pub fn get_as_0_100(&self) -> f32 {
                self.value * 100.0
            }

            //endregion

            //region Internal

            #[inline]
            fn linear_interp(input: f32, range: &std::ops::Range<f32>) -> f32 {
               (input - range.start) / (range.end - range.start)
            }

            //endregion

            }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "Percent({} %)", self.get_as_0_100())
            }
        }

        impl TryFrom<f32> for $name {
            type Error = FeagiDataError;
            fn try_from(value: f32) -> Result<Self, Self::Error> {
                $name::new_from_0_1(value)
            }
        }

        impl TryFrom<&f32> for $name {
            type Error = FeagiDataError;
            fn try_from(value: &f32) -> Result<Self, Self::Error> {
                $name::new_from_0_1(*value)
            }
        }

        impl From<$name> for f32 {
            fn from(value: $name) -> Self {
                value.value
            }
        }

        impl From<&$name> for f32 {
            fn from(value: &$name) -> Self {
                value.value
            }
        }

    }
}
/// Creates bidirectional conversions between two unsigned percentage types.
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_unsigned_percentage, map_unsigned_percentages, FeagiDataError};
/// 
/// define_unsigned_percentage!(Brightness, "Screen brightness percentage");
/// define_unsigned_percentage!(Volume, "Audio volume percentage");
/// map_unsigned_percentages!(Brightness, Volume);
/// 
/// let brightness = Brightness::new_from_0_1(0.8).unwrap();
/// let volume: Volume = brightness.into();
/// assert_eq!(volume.get_as_0_1(), 0.8);
/// 
/// let back_to_brightness: Brightness = volume.into();
/// assert_eq!(back_to_brightness.get_as_0_1(), 0.8);
/// ```
#[macro_export]
macro_rules! map_unsigned_percentages {
    ($percentage_a:ident, $percentage_b:ident) => {

        impl From<$percentage_a> for $percentage_b {
            fn from(value: $percentage_a) -> Self {
                $percentage_b::new_from_0_1_unchecked(value.get_as_0_1())
            }
        }

        impl From<$percentage_b> for $percentage_a {
            fn from(value: $percentage_b) -> Self {
                $percentage_a::new_from_0_1_unchecked(value.get_as_0_1())
            }
        }
    };
}

/// Creates a signed percentage type with value range [-1.0, 1.0].
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_signed_percentage, FeagiDataError};
/// 
/// define_signed_percentage!(Adjustment, "Signed adjustment value from -100% to +100%");
/// 
/// let adj = Adjustment::new_from_m1_1(0.5).unwrap();
/// assert_eq!(adj.get_as_m100_100(), 50.0);
/// 
/// let from_unsigned = Adjustment::new_from_0_1(0.75).unwrap();
/// assert_eq!(from_unsigned.get_as_m1_1(), 0.5);
/// 
/// let negative = Adjustment::new_from_m100_100(-25.0).unwrap();
/// assert_eq!(negative.get_as_m1_1(), -0.25);
/// 
/// let invalid = Adjustment::new_from_m1_1(2.0);
/// assert!(invalid.is_err());
/// ```
#[macro_export]
macro_rules! define_signed_percentage {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name {
            value: f32,
        }

        impl $name
        {

            //region Constructors

            pub fn new_from_m1_1_unchecked(value: f32) -> Self {
                $name { value }
            }

            pub fn new_from_m1_1(value: f32) -> Result<$name, FeagiDataError> {
                if value > 1.0 || value < -1.0 {
                    return Err(FeagiDataError::BadParameters("Signed Percentage Value must be between -1 and 1!".into()));
                }
                Ok($name { value })
            }

            pub fn new_from_0_1(value: f32) -> Result<$name, FeagiDataError> {
                if value > 1.0 || value < 0.0 {
                    return Err(FeagiDataError::BadParameters("Percentage Value to interp from must be between 0 and 1!".into()));
                }
                Ok($name { value: (value - 0.5) * 2.0})
            }

            pub fn new_from_0_1_unchecked(value: f32) -> Self {
                $name { value: (value - 0.5) * 2.0}
            }


            pub fn new_from_m100_100(value: f32) -> Result<$name, FeagiDataError> {
                if value > 100.0 || value < -100.0 {
                    return Err(FeagiDataError::BadParameters("Signed Percentage Value must be between -100 and 100!".into()));
                }
                Ok($name { value: value / 100.0 })
            }

            pub fn new_from_linear_interp(value: f32, range: &std::ops::Range<f32>) -> Result<$name, FeagiDataError> {
                if value < range.start || value > range.end {
                    return Err(FeagiDataError::BadParameters(format!("Given value {} is out of range {:?}!", value, range)));
                }
                Ok($name { value: Self::linear_interp(value, range) })

            }

            //endregion

            //region Update

            pub(crate) fn inplace_update_unchecked(&mut self, value: f32)  {
                self.value = value;
            }

            pub fn inplace_update_from_m1_1(&mut self, value: f32) -> Result<(), FeagiDataError> {
                if value > 1.0 || value < -1.0 {
                    return Err(FeagiDataError::BadParameters("Percentage Value must be between -1 and 1!".into()));
                }
                self.value = value;
                Ok(())
            }

            pub fn inplace_update_m100_100(&mut self, value: f32) -> Result<(), FeagiDataError> {
                if value > 100.0 || value < -100.0 {
                    return Err(FeagiDataError::BadParameters("Percentage Value must be between -1 and 1!".into()));
                }
                self.value = value / 100.0;
                Ok(())
            }

            pub fn inplace_update_linear_interp(&mut self, value: f32, range: &std::ops::Range<f32>) -> Result<(), FeagiDataError> {
                if value < range.start || value > range.end {
                    return Err(FeagiDataError::BadParameters(format!("Given value {} is out of range {:?}!", value, range)));
                }
                self.value = Self::linear_interp(value, range);
                Ok(())
            }

            //endregion

            //region Properties

            pub fn get_as_m1_1(&self) -> f32 {
                self.value
            }

            pub fn get_as_m100_100(&self) -> f32 {
                self.value * 100.0
            }

            //endregion

            //region Internal

            #[inline]
            fn linear_interp(input: f32, range: &std::ops::Range<f32>) -> f32 {
                (range.start + range.end - (2.0 * input)) / (range.start - range.end)
            }

            //endregion

            }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "SignedPercent({} %)", self.get_as_m100_100())
            }
        }

        impl TryFrom<f32> for $name {
            type Error = FeagiDataError;
            fn try_from(value: f32) -> Result<Self, Self::Error> {
                $name::new_from_m1_1(value)
            }
        }

        impl TryFrom<&f32> for $name {
            type Error = FeagiDataError;
            fn try_from(value: &f32) -> Result<Self, Self::Error> {
                $name::new_from_m1_1(*value)
            }
        }

        impl From<$name> for f32 {
            fn from(value: $name) -> Self {
                value.value
            }
        }

        impl From<&$name> for f32 {
            fn from(value: &$name) -> Self {
                value.value
            }
        }

    }
}

/// Creates bidirectional conversions between two signed percentage types.
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_signed_percentage, map_signed_percentages, FeagiDataError};
/// 
/// define_signed_percentage!(Temperature, "Temperature adjustment from -100% to +100%");
/// define_signed_percentage!(Contrast, "Contrast adjustment from -100% to +100%");
/// map_signed_percentages!(Temperature, Contrast);
/// 
/// let temp = Temperature::new_from_m1_1(-0.3).unwrap();
/// let contrast: Contrast = temp.into();
/// assert_eq!(contrast.get_as_m1_1(), -0.3);
/// 
/// let back_to_temp: Temperature = contrast.into();
/// assert_eq!(back_to_temp.get_as_m1_1(), -0.3);
/// ```
#[macro_export]
macro_rules! map_signed_percentages {
    ($percentage_a:ident, $percentage_b:ident) => {

        impl From<$percentage_a> for $percentage_b {
            fn from(value: $percentage_a) -> Self {
                $percentage_b::new_from_m1_1_unchecked(value.get_as_m1_1())
            }
        }

        impl From<$percentage_b> for $percentage_a {
            fn from(value: $percentage_b) -> Self {
                $percentage_a::new_from_m1_1_unchecked(value.get_as_m1_1())
            }
        }
    };
}

//endregion

//region Two Dimensional

/// Creates a 2D percentage type with two percentage values.
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_unsigned_percentage, define_2d_signed_or_unsigned_percentages, FeagiDataError};
/// 
/// define_unsigned_percentage!(Factor, "A scaling factor");
/// define_2d_signed_or_unsigned_percentages!(Scale2D, Factor, "Scale2D", "2D scaling factors");
/// 
/// let factor_a = Factor::new_from_0_1(0.5).unwrap();
/// let factor_b = Factor::new_from_0_1(0.8).unwrap();
/// let scale = Scale2D::new(factor_a, factor_b);
/// 
/// assert_eq!(scale.a.get_as_0_100(), 50.0);
/// assert_eq!(scale.b.get_as_0_100(), 80.0);
/// println!("{}", scale); // Scale2D(Percent(50 %), Percent(80 %))
/// ```
#[macro_export]
macro_rules! define_2d_signed_or_unsigned_percentages {
    ($name:ident, $percentage_type:ty, $friendly_name:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Copy)]
        pub struct $name {
            pub a: $percentage_type,
            pub b: $percentage_type,
        }

        impl $name {
            pub fn new(a: $percentage_type, b: $percentage_type) -> Self {
                Self { a, b }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({}, {})", $friendly_name, self.a, self.b)
            }
        }

    };
}

//endregion

//region Three Dimensional

/// Creates a 3D percentage type with three percentage values.
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_signed_percentage, define_3d_signed_or_unsigned_percentages, FeagiDataError};
/// 
/// define_signed_percentage!(Adjustment, "An adjustment factor");
/// define_3d_signed_or_unsigned_percentages!(Color3D, Adjustment, "Color3D", "3D color adjustments (RGB)");
/// 
/// let r = Adjustment::new_from_m1_1(0.2).unwrap();
/// let g = Adjustment::new_from_m1_1(-0.1).unwrap();
/// let b = Adjustment::new_from_m1_1(0.5).unwrap();
/// let color = Color3D::new(r, g, b);
/// 
/// assert_eq!(color.a.get_as_m100_100(), 20.0);
/// assert_eq!(color.b.get_as_m100_100(), -10.0);
/// assert_eq!(color.c.get_as_m100_100(), 50.0);
/// ```
#[macro_export]
macro_rules! define_3d_signed_or_unsigned_percentages {
    ($name:ident, $percentage_type:ty, $friendly_name:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Copy)]
        pub struct $name {
            pub a: $percentage_type,
            pub b: $percentage_type,
            pub c: $percentage_type,
        }

        impl $name {
            pub fn new(a: $percentage_type, b: $percentage_type, c: $percentage_type) -> Self {
                Self { a, b, c }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({}, {}, {})", $friendly_name, self.a, self.b, self.c)
            }
        }

    };
}

//endregion

//region Four Dimensional

/// Creates a 4D percentage type with four percentage values.
/// 
/// # Example
/// ```
/// use feagi_data_structures::{define_unsigned_percentage, define_4d_signed_or_unsigned_percentages, FeagiDataError};
/// 
/// define_unsigned_percentage!(Channel, "A color channel value");
/// define_4d_signed_or_unsigned_percentages!(RGBA, Channel, "RGBA", "4D color with alpha (RGBA)");
/// 
/// let r = Channel::new_from_0_1(1.0).unwrap();
/// let g = Channel::new_from_0_1(0.5).unwrap();
/// let b = Channel::new_from_0_1(0.2).unwrap();
/// let a = Channel::new_from_0_1(0.8).unwrap();
/// let rgba = RGBA::new(r, g, b, a);
/// 
/// assert_eq!(rgba.a.get_as_0_100(), 100.0);
/// assert_eq!(rgba.b.get_as_0_100(), 50.0);
/// assert_eq!(rgba.c.get_as_0_100(), 20.0);
/// assert_eq!(rgba.d.get_as_0_100(), 80.0);
/// ```
#[macro_export]
macro_rules! define_4d_signed_or_unsigned_percentages {
    ($name:ident, $percentage_type:ty, $friendly_name:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Copy)]
        pub struct $name {
            pub a: $percentage_type,
            pub b: $percentage_type,
            pub c: $percentage_type,
            pub d: $percentage_type,
        }

        impl $name {
            pub fn new(a: $percentage_type, b: $percentage_type, c: $percentage_type, d: $percentage_type) -> Self {
                Self { a, b, c, d }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({}, {}, {}, {})", $friendly_name, self.a, self.b, self.c, self.d)
            }
        }

    };
}

//endregion

