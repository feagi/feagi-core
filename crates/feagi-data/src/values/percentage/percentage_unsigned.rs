/// Stores a percentage 0 - 100 % as a u8, but can be accessible as a float
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct PercentageUnsigned(u8);

impl PercentageUnsigned {
    const F32_MAX: f32 = u8::MAX as f32;
    const F64_MAX: f64 = f64::MAX;

    // TODO decimal quantization funcs!

    pub fn from_f32_clamped(value_0_1: f32) -> PercentageUnsigned {
        let value_0_1 = value_0_1.clamp(0.0, 1.0);
        let u = (value_0_1 * (u8::MAX as f32)) as usize as u8;
        PercentageUnsigned(u)
    }

    pub fn from_f32_unchecked(value_0_1: f32) -> PercentageUnsigned {
        let u = (value_0_1 * (u8::MAX as f32)) as usize as u8;
        PercentageUnsigned(u)
    }

    pub fn to_f32_0_1(self) -> f32 {
        self.0 as f32 / Self::F32_MAX
    }

    pub fn from_f64_clamped(value_0_1: f64) -> PercentageUnsigned {
        let value_0_1 = value_0_1.clamp(0.0, 1.0);
        let u = (value_0_1 * (u8::MAX as f64)) as usize as u8;
        PercentageUnsigned(u)
    }

    pub fn from_f64_unchecked(value_0_1: f64) -> PercentageUnsigned {
        let u = (value_0_1 * (u8::MAX as f64)) as usize as u8;
        PercentageUnsigned(u)
    }

    pub fn to_f64_0_1(self) -> f64 {
        self.0 as f64 / Self::F64_MAX
    }
}
