#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FeagiServerBindState {
    Inactive,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeagiServerBindStateChange {
    previous: FeagiServerBindState,
    now: FeagiServerBindState,
}

impl FeagiServerBindStateChange {
    pub fn new(previous: FeagiServerBindState, now: FeagiServerBindState) -> Self {
        Self { previous, now }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub u64);