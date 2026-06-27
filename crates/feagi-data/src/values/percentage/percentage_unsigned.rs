
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Default,
)]

#[repr(transparent)]
pub struct PercentageUnsigned(u8);

impl PercentageUnsigned {

    // TODO decimal quantization funcs!

    pub fn from_f32_clamped(value_0_1: f32) -> PercentageUnsigned {
        // TODO check NaN?
        let value_0_1 = value_0_1.clamp(0.0, 1.0);
        let u = (value_0_1 * (u8::MAX as f32)) as usize as u8;
        PercentageUnsigned(u)
    }

    pub fn from_f32_unchecked(value_0_1: f32) -> PercentageUnsigned {
        let u = (value_0_1 * (u8::MAX as f32)) as usize as u8;
        PercentageUnsigned(u)
    }

    pub fn to_f32_0_1(self) -> f32 {
        self.0 as f32 / u8::MAX as f32
    }
}

