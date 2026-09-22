

/// 4 Bits representing the class of the cortical group that one belongs to
#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum CorticalGroupClass {
    /// For cortical groups such as Brain Regions, SensoriMotor Unit
    Other = 0,
    /// Cortical Area Groups that process data in some manner
    CorticalProcessor = 1,
    Reserved1 = 2,
    Reserved2 = 3
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum CorticalGroupType {
    BrainRegion,
    SensoriMotorUnit, // Unique in that this is never used in a UUID context

}

