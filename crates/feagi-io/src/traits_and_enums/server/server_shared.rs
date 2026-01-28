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

    pub fn get_previous(self) -> FeagiServerBindState {
        self.previous
    }

    pub fn get_now(self) -> FeagiServerBindState {
        self.now
    }
}

