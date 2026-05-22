
/// Specifies that the type can be used for Linear Indexing
pub trait LinearIndexCountType:
Copy
+ Clone
+ Send
+ Sync
+ Default
+ core::ops::Add<Output = Self>
+ core::ops::Sub<Output = Self>
+ core::ops::Mul<Output = Self>
+ core::ops::Div<Output = Self>
+ core::ops::AddAssign
+ core::ops::SubAssign
+ core::ops::MulAssign
+ core::ops::DivAssign
+ core::cmp::PartialOrd
+ core::ops::Rem<Output = Self>
+ core::ops::RemAssign
{ 
    fn to_usize(self) -> usize;
}

impl LinearIndexCountType for usize {
    fn to_usize(self) -> usize {
        self
    }
}

impl LinearIndexCountType for u8 {
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl LinearIndexCountType for u16 {
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl LinearIndexCountType for u32 {
    fn to_usize(self) -> usize {
        self as usize
    }
}

#[cfg(feature = "support_64bit_indexing")]
impl LinearIndexCountType for u64 {
    fn to_usize(self) -> usize {
        self
    }
}