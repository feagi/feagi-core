use feagi_data_structures::FeagiDataError;


//region 1D Percentage Types
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Percentage {
    value: f32,
}

impl Percentage {

//region Constructors

    pub fn new_from_0_1_unchecked(value: f32) -> Self {
        Percentage { value }
    }

    pub fn new_from_0_1(value: f32) -> Result<Percentage, FeagiDataError> {
    if value > 1.0 || value < 0.0 {
    return Err(FeagiDataError::BadParameters("Percentage Value must be between 0 and 1!".into()));
    }
        Ok(Percentage { value })
    }

    pub fn new_from_interp_m1_1(value: f32) -> Result<Percentage, FeagiDataError> {
    if value > 1.0 || value < -1.0 {
    return Err(FeagiDataError::BadParameters("Signed Percentage Value to interp from must be between -1 and 1!".into()));
    }
        Ok(Percentage { value: (value + 1.0) / 2.0 })
    }

    pub fn new_from_interp_m1_1_unchecked(value: f32) -> Self {
        Percentage { value: (value + 1.0) / 2.0 }
    }

    pub fn new_from_u8_0_255(value: u8) -> Result<Percentage, FeagiDataError> {
        Percentage::new_from_0_1(value as f32 / u8::MAX as f32)
    }

    pub fn new_from_0_100(value: f32) -> Result<Percentage, FeagiDataError> {
        if value > 100.0 || value < 0.0 {
            return Err(FeagiDataError::BadParameters("Percentage Value must be between 0 and 100!".into()));
        }
        Ok(Percentage { value: value / 100.0 })
    }

    pub fn new_from_linear_interp(value: f32, range: &std::ops::Range<f32>) -> Result<Percentage, FeagiDataError> {
        if value < range.start || value > range.end {
            return Err(FeagiDataError::BadParameters(format!("Given value {} is out of range {:?}!", value, range)));
        }
        Ok(Percentage { value: Self::linear_interp(value, range) })
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

impl std::fmt::Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Percent({} %)", self.get_as_0_100())
    }
}

impl TryFrom<f32> for Percentage {
type Error = FeagiDataError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Percentage::new_from_0_1(value)
    }
}

impl TryFrom<&f32> for Percentage {
    type Error = FeagiDataError;
    fn try_from(value: &f32) -> Result<Self, Self::Error> {
        Percentage::new_from_0_1(*value)
    }
}

impl From<Percentage> for f32 {
    fn from(value: Percentage) -> Self {
        value.value
    }
}

impl From<&Percentage> for f32 {
    fn from(value: &Percentage) -> Self {
        value.value
    }
}

/// A signed percentage value, from -100 to 100%
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SignedPercentage {
    value: f32,
}

impl SignedPercentage {

    //region Constructors

    pub fn new_from_m1_1_unchecked(value: f32) -> Self {
        SignedPercentage { value }
    }

    pub fn new_from_m1_1(value: f32) -> Result<SignedPercentage, FeagiDataError> {
        if value > 1.0 || value < -1.0 {
            return Err(FeagiDataError::BadParameters("Signed Percentage Value must be between -1 and 1!".into()));
        }
        Ok(SignedPercentage { value })
    }

    pub fn new_scaled_from_0_1(value: f32) -> Result<SignedPercentage, FeagiDataError> {
        if value > 1.0 || value < 0.0 {
            return Err(FeagiDataError::BadParameters("Percentage Value to interp from must be between 0 and 1!".into()));
        }
        Ok(SignedPercentage { value: (value - 0.5) * 2.0})
    }

    pub fn new_scaled_from_0_1_unchecked(value: f32) -> Self {
        SignedPercentage { value: (value - 0.5) * 2.0}
    }

    pub fn new_from_m100_100(value: f32) -> Result<SignedPercentage, FeagiDataError> {
        if value > 100.0 || value < -100.0 {
            return Err(FeagiDataError::BadParameters("Signed Percentage Value must be between -100 and 100!".into()));
        }
        Ok(SignedPercentage { value: value / 100.0 })
    }

    pub fn new_from_linear_interp(value: f32, range: &std::ops::Range<f32>) -> Result<SignedPercentage, FeagiDataError> {
        if value < range.start || value > range.end {
            return Err(FeagiDataError::BadParameters(format!("Given value {} is out of range {:?}!", value, range)));
        }
        Ok(SignedPercentage { value: Self::linear_interp(value, range) })

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

impl std::fmt::Display for SignedPercentage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SignedPercent({} %)", self.get_as_m100_100())
    }
}

impl TryFrom<f32> for SignedPercentage {
    type Error = FeagiDataError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        SignedPercentage::new_from_m1_1(value)
    }
}

impl TryFrom<&f32> for SignedPercentage {
    type Error = FeagiDataError;
    fn try_from(value: &f32) -> Result<Self, Self::Error> {
        SignedPercentage::new_from_m1_1(*value)
    }
}

impl From<SignedPercentage> for f32 {
    fn from(value: SignedPercentage) -> Self {
        value.value
    }
}

impl From<&SignedPercentage> for f32 {
    fn from(value: &SignedPercentage) -> Self {
        value.value
    }
}

//endregion

//region 2D Percentage Types

/// Represents 2 percentages over 2 dimensions, going from 0 - 100%
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Percentage2D {
    pub a: Percentage,
    pub b: Percentage,
}

impl Percentage2D {
    pub fn new(a: Percentage, b: Percentage) -> Self {
        Self { a, b }
    }

    pub fn new_zero() -> Self {
        Self {
            a: Percentage::new_from_0_1_unchecked(0.0),
            b: Percentage::new_from_0_1_unchecked(0.0),
        }
    }

    pub fn new_identical_percentages(percentage: Percentage) -> Self {
        Self {
            a: percentage,
            b: percentage,
        }
    }
    
    pub(crate) fn inplace_update_all(&mut self, value: f32) {
        self.a.inplace_update(value);
        self.b.inplace_update(value);
    }
}

impl std::fmt::Display for Percentage2D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Percentage2D({}, {})", self.a, self.b)
    }
}

/// Represents 2 signed percentages over 2 dimensions, going from -100 - 100%
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct SignedPercentage2D {
    pub a: SignedPercentage,
    pub b: SignedPercentage,
}

impl SignedPercentage2D {
    pub fn new(a: SignedPercentage, b: SignedPercentage) -> Self {
        Self { a, b }
    }

    pub fn new_zero() -> Self {
        Self {
            a: SignedPercentage::new_from_m1_1_unchecked(0.0),
            b: SignedPercentage::new_from_m1_1_unchecked(0.0),
        }
    }

    pub fn new_identical_percentages(percentage: SignedPercentage) -> Self {
        Self {
            a: percentage,
            b: percentage,
        }
    }
    
    pub(crate) fn inplace_update_all(&mut self, value: f32) {
        self.a.inplace_update_unchecked(value);
        self.b.inplace_update_unchecked(value);
    }
}

impl std::fmt::Display for SignedPercentage2D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SignedPercentage2D({}, {})", self.a, self.b)
    }
}

//endregion

//region 3D Percentage Types

/// Represents 3 percentages over 3 dimensions, going from 0 - 100%
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Percentage3D {
    pub a: Percentage,
    pub b: Percentage,
    pub c: Percentage,
}

impl Percentage3D {
    pub fn new(a: Percentage, b: Percentage, c: Percentage) -> Self {
        Self { a, b, c }
    }

    pub fn new_zero() -> Self {
        Self {
            a: Percentage::new_from_0_1_unchecked(0.0),
            b: Percentage::new_from_0_1_unchecked(0.0),
            c: Percentage::new_from_0_1_unchecked(0.0),
        }
    }

    pub fn new_identical_percentages(percentage: Percentage) -> Self {
        Self {
            a: percentage,
            b: percentage,
            c: percentage,
        }
    }
    
    pub(crate) fn inplace_update_all(&mut self, value: f32) {
        self.a.inplace_update(value);
        self.b.inplace_update(value);
        self.c.inplace_update(value);
    }
}

impl std::fmt::Display for Percentage3D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Percentage3D({}, {}, {})", self.a, self.b, self.c)
    }
}

/// Represents 3 signed percentages over 3 dimensions, going from -100 - 100%
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct SignedPercentage3D {
    pub a: SignedPercentage,
    pub b: SignedPercentage,
    pub c: SignedPercentage,
}

impl SignedPercentage3D {
    pub fn new(a: SignedPercentage, b: SignedPercentage, c: SignedPercentage) -> Self {
        Self { a, b, c }
    }

    pub fn new_zero() -> Self {
        Self {
            a: SignedPercentage::new_from_m1_1_unchecked(0.0),
            b: SignedPercentage::new_from_m1_1_unchecked(0.0),
            c: SignedPercentage::new_from_m1_1_unchecked(0.0),
        }
    }

    pub fn new_identical_percentages(percentage: SignedPercentage) -> Self {
        Self {
            a: percentage,
            b: percentage,
            c: percentage,
        }
    }
    
    pub(crate) fn inplace_update_all(&mut self, value: f32) {
        self.a.inplace_update_unchecked(value);
        self.b.inplace_update_unchecked(value);
        self.c.inplace_update_unchecked(value);
    }
}

impl std::fmt::Display for SignedPercentage3D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SignedPercentage3D({}, {}, {})", self.a, self.b, self.c)
    }
}

//endregion

//region 4D Percentage Types

/// Represents 4 percentages over 4 dimensions, going from 0 - 100%
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Percentage4D {
    pub a: Percentage,
    pub b: Percentage,
    pub c: Percentage,
    pub d: Percentage,
}

impl Percentage4D {
    pub fn new(a: Percentage, b: Percentage, c: Percentage, d: Percentage) -> Self {
        Self { a, b, c, d }
    }

    pub fn new_zero() -> Self {
        Self {
            a: Percentage::new_from_0_1_unchecked(0.0),
            b: Percentage::new_from_0_1_unchecked(0.0),
            c: Percentage::new_from_0_1_unchecked(0.0),
            d: Percentage::new_from_0_1_unchecked(0.0),
        }
    }

    pub fn new_identical_percentages(percentage: Percentage) -> Self {
        Self {
            a: percentage,
            b: percentage,
            c: percentage,
            d: percentage,
        }
    }
    
    pub(crate) fn inplace_update_all(&mut self, value: f32) {
        self.a.inplace_update(value);
        self.b.inplace_update(value);
        self.c.inplace_update(value);
        self.d.inplace_update(value);
    }
}

impl std::fmt::Display for Percentage4D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Percentage4D({}, {}, {}, {})", self.a, self.b, self.c, self.d)
    }
}

/// Represents 4 signed percentages over 4 dimensions, going from -100 - 100%
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct SignedPercentage4D {
    pub a: SignedPercentage,
    pub b: SignedPercentage,
    pub c: SignedPercentage,
    pub d: SignedPercentage,
}

impl SignedPercentage4D {
    pub fn new(a: SignedPercentage, b: SignedPercentage, c: SignedPercentage, d: SignedPercentage) -> Self {
        Self { a, b, c, d }
    }

    pub fn new_zero() -> Self {
        Self {
            a: SignedPercentage::new_from_m1_1_unchecked(0.0),
            b: SignedPercentage::new_from_m1_1_unchecked(0.0),
            c: SignedPercentage::new_from_m1_1_unchecked(0.0),
            d: SignedPercentage::new_from_m1_1_unchecked(0.0),
        }
    }

    pub fn new_identical_percentages(percentage: SignedPercentage) -> Self {
        Self {
            a: percentage,
            b: percentage,
            c: percentage,
            d: percentage,
        }
    }
    
    pub(crate) fn inplace_update_all(&mut self, value: f32) {
        self.a.inplace_update_unchecked(value);
        self.b.inplace_update_unchecked(value);
        self.c.inplace_update_unchecked(value);
        self.d.inplace_update_unchecked(value);
    }
}

impl std::fmt::Display for SignedPercentage4D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SignedPercentage4D({}, {}, {}, {})", self.a, self.b, self.c, self.d)
    }
}

//endregion