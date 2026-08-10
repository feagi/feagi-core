// TODO this will be replacing CorticalAreaType, as the name is less confusing but also for better flexibility

/// All high level classes a cortical area can fall under
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CorticalAreaClassFlat {
    Core,
    Interconnect,
    Memory,
    Sensor,
    Motor,
}

// TODO get from class what layouts are supported (return a static arr)
