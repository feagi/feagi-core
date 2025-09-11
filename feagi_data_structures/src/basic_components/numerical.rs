use crate::FeagiDataError;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Percentage {
    value: f32,
}


impl Percentage {
    pub fn new_from_0_1(value: f32) -> Result<Self, FeagiDataError> {
        if value > 1.0 || value < 0.0 {
            return Err(FeagiDataError::BadParameters("Percentage Value must be between 0 and 1!".into()));
        }
        Ok(Percentage { value })
    }

    pub fn new_from_u8_0_255(value: u8) -> Result<Self, FeagiDataError> {
        Percentage::new_from_0_1(value as f32 / u8::MAX as f32)
    }

    pub fn new_from_0_100(value: f32) -> Result<Self, FeagiDataError> {
        if value > 100.0 || value < 0.0 {
            return Err(FeagiDataError::BadParameters("Percentage Value must be between 0 and 100!".into()));
        }
        Ok(Percentage { value: value / 100.0 })
    }

    pub fn get_as_0_1(&self) -> f32 {
        self.value
    }

    pub fn get_as_u8(&self) -> u8 {
        (self.value * u8::MAX as f32) as u8
    }

    pub fn get_as_0_100(&self) -> f32 {
        self.value * 100.0
    }
}


