
/// Due to limitations with constants in mainline rust, we need to use AxisOrderIdentifier as a
/// stand in for `AxisOrder` and convert it ourselves later.
pub type AxisOrderIdentifier = u8;

pub type AxisOrderArray<const NUM_DIMS: usize> = [usize; NUM_DIMS];

#[repr(u8)]
#[derive(Debug, Default, Hash, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AxisOrder<const NUM_DIMS: usize> {
    #[default]
    DefaultIncrementing = 0
}

impl<const NUM_DIMS: usize> AxisOrder<NUM_DIMS> {
    
    pub const fn generate_axis_order_array(self) -> AxisOrderArray<NUM_DIMS> {
        let mut out = [0usize; NUM_DIMS];
        let mut i = 0usize;
        match self { 
            AxisOrder::DefaultIncrementing => {
                while i < NUM_DIMS {
                    out[i] = i;
                    i = i + 1;
                };
                out
            }
        }
    }

    pub const fn from_identifier(u: AxisOrderIdentifier)  -> AxisOrder<NUM_DIMS> {
        Self::from_u8(u)
    }

    pub const fn from_u8(u: u8) -> AxisOrder<NUM_DIMS> {
        match u { 
            0 => AxisOrder::DefaultIncrementing,
            _ => panic!("Invalid Axis Order!")
        }
    }
} 