#[doc = $doc]
#[repr(transparent)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct NonzeroCount(usize);

impl NonzeroCount {
    pub fn new(n: usize) -> Result<Self, FeagiNeuronError> {
        if n == 0 {
            return Err(FeagiNeuronError::NeuronCoordinateOutOfRange());
        }
    }
}

impl std::ops::Deref for NonzeroCount {
    type Target = NonzeroCount;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}