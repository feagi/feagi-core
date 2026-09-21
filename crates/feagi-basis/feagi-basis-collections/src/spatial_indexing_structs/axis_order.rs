
/// Due to limitations with constants in mainline rust, we need to use AxisOrderIdentifier as a
/// stand in for `AxisOrder` and convert it ourselves later.
pub type AxisOrderIdentifier = u8;

/// The actual axis order used for indexing calculations
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub struct AxisOrderArray<const NUM_DIMS: usize>{
    #[serde(with = "serde_arrays")]
    data: [usize; NUM_DIMS]
}

impl<const NUM_DIMS: usize> AxisOrderArray<NUM_DIMS> {
    /// Borrow dimensions as a fixed-size slice.
    pub fn as_slice(&self) -> &[usize; NUM_DIMS] {
        &self.data
    }
}


/// An enum storing the various axis orders we use
#[repr(u8)]
#[derive(Debug, Default, Hash, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AxisOrderEnum {
    #[default]
    DefaultIncrementing = 0,
    Decrementing = 1
}

impl AxisOrderEnum {
    
    pub const fn generate_axis_order_array<const NUM_DIMS: usize>(self) -> AxisOrderArray<NUM_DIMS> {
        let mut out = [0usize; NUM_DIMS];
        let mut i = 0usize;
        match self { 
            AxisOrderEnum::DefaultIncrementing => {
                while i < NUM_DIMS {
                    out[i] = i;
                    i = i + 1;
                }
            }
            AxisOrderEnum::Decrementing => {
                i = NUM_DIMS;
                while i != 0 {
                    i = i - 1;
                    out[i] = i
                }
            }
        }
        AxisOrderArray {data: out}
    }

    pub const fn from_identifier<const NUM_DIMS: usize>(u: AxisOrderIdentifier)  -> AxisOrderEnum {
        Self::from_u8::<NUM_DIMS>(u)
    }

    pub const fn from_u8<const NUM_DIMS: usize>(u: u8) -> AxisOrderEnum {
        match u { 
            0 => AxisOrderEnum::DefaultIncrementing,
            1 => AxisOrderEnum::Decrementing,
            _ => panic!("Invalid Axis Order!")
        }
    }
} 
