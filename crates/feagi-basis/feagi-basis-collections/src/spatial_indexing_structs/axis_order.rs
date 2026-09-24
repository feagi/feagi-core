

// Yes, we arent using enums. This is due to typing restrictions with the rust stable toolchain

pub trait AxisOrder<const NUM_DIMS: usize> {
    const AXIS_ORDER_ARRAY: AxisOrderArray<NUM_DIMS>;
}

/// The default axis incrementing pattern, x -> y -> z ....
#[derive(Debug, PartialEq, Hash, Clone, ::serde::Serialize, ::serde::Deserialize )]
pub struct AxisOrderIncrementing<const NUM_DIMS: usize>;

impl<const NUM_DIMS: usize> AxisOrder<NUM_DIMS> for  AxisOrderIncrementing<NUM_DIMS> {
    const AXIS_ORDER_ARRAY: AxisOrderArray<NUM_DIMS> = {
        let mut out = [0u8; NUM_DIMS];
        let mut i = 0u8;
        while i < NUM_DIMS as u8 {
            out[i as usize] = i;
            i = i + 1;
        };
        AxisOrderArray {data: out}
    };
}


/// Starts with the last index and goes backwards (z -> y -> x ...)
#[derive(Debug, PartialEq, Hash, Clone, ::serde::Serialize, ::serde::Deserialize )]
pub struct AxisOrderDecrementing<const NUM_DIMS: usize>;

impl<const NUM_DIMS: usize> AxisOrder<NUM_DIMS> for  AxisOrderDecrementing<NUM_DIMS> {
    const AXIS_ORDER_ARRAY: AxisOrderArray<NUM_DIMS> = {
        let mut out = [0u8; NUM_DIMS];
        let mut i = NUM_DIMS as u8;
        while i != 0 {
            i = i - 1;
            out[i as usize] = i
        }
        AxisOrderArray {data: out}
    };
}



/// The actual axis order used for indexing calculations
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub struct AxisOrderArray<const NUM_DIMS: usize>{
    #[serde(with = "serde_arrays")]
    data: [u8; NUM_DIMS]
}

impl<const NUM_DIMS: usize> AxisOrderArray<NUM_DIMS> {
    /// Borrow dimensions as a fixed-size slice.
    pub fn as_slice(&self) -> &[u8; NUM_DIMS] {
        &self.data
    }
}