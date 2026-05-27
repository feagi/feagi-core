
//region Cortical Area Level

/// Defines a cortical area within a cortical unit in a manner that can be stored in a const array
pub struct IOCorticalAreaDefinition {
    pub friendly_english_name: &'static str,
    pub cortical_sub_unit_index: u8,
    pub cortical_area_data_type: CorticalAreaDataTypeFlag,
    pub relative_position: (i32, i32, i32),
    pub channel_dimensions_min: (u32, u32, u32),
    pub channel_dimensions_default: (u32, u32, u32),
    pub channel_dimensions_max: (u32, u32, u32),
}

/// Defines the type of data (and parameters describing how its processed) represented by a cortical
/// area in an enum that can be fit into a single byte
pub enum CorticalAreaDataTypeFlag {
    Percentage(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage2D(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage3D(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage4D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage2D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage3D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage4D(FrameChangeHandling, PercentageNeuronPositioning),
    Boolean(),
    MiscData(FrameChangeHandling),
    CartesianPlane(FrameChangeHandling),
}

// NOTE: as of now we are within 4 bits for the number of data types

impl CorticalAreaDataTypeFlag {
    pub fn try_from_u8(byte: u8) -> Option<Self> {
        todo!()
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            CorticalAreaDataTypeFlag::Percentage(frame, per) => {
                let mut val: u8 = 0;
                if *frame == FrameChangeHandling::Incremental { val |= 1 << 7; }
                if *per == PercentageNeuronPositioning::Fractional { val |= 1 << 6; }
                val

            }
            CorticalAreaDataTypeFlag::Percentage2D(frame, per) => {
                let mut val: u8 = 1;
                if *frame == FrameChangeHandling::Incremental { val |= 1 << 7; }
                if *per == PercentageNeuronPositioning::Fractional { val |= 1 << 6; }
                val
            }
            CorticalAreaDataTypeFlag::Percentage3D(frame, per) => {
                let mut val: u8 = 2;
                if *frame == FrameChangeHandling::Incremental { val |= 1 << 7; }
                if *per == PercentageNeuronPositioning::Fractional { val |= 1 << 6; }
                val
            }
            CorticalAreaDataTypeFlag::Percentage4D(frame, per) => {
                let mut val: u8 = 3;
                if *frame == FrameChangeHandling::Incremental { val |= 1 << 7; }
                if *per == PercentageNeuronPositioning::Fractional { val |= 1 << 6; }
                val
            }
            CorticalAreaDataTypeFlag::SignedPercentage(frame, per) => {
                let mut val: u8 = 4;
                if *frame == FrameChangeHandling::Incremental { val |= 1 << 7; }
                if *per == PercentageNeuronPositioning::Fractional { val |= 1 << 6; }
                val
            }
            CorticalAreaDataTypeFlag::SignedPercentage2D(frame, per) => {
                let mut val: u8 = 5;
                if *frame == FrameChangeHandling::Incremental { val |= 1 << 7; }
                if *per == PercentageNeuronPositioning::Fractional { val |= 1 << 6; }
                val
            }
            CorticalAreaDataTypeFlag::SignedPercentage3D(frame, per) => {
                let mut val: u8 = 6;
                if *frame == FrameChangeHandling::Incremental { val |= 1 << 7; }
                if *per == PercentageNeuronPositioning::Fractional { val |= 1 << 6; }
                val
            }
            CorticalAreaDataTypeFlag::SignedPercentage4D(frame, per) => {
                let mut val: u8 = 7;
                if *frame == FrameChangeHandling::Incremental { val |= 1 << 7; }
                if *per == PercentageNeuronPositioning::Fractional { val |= 1 << 6; }
                val
            }
            CorticalAreaDataTypeFlag::Boolean() => {
                let mut val: u8 = 8;
                val
            }
            CorticalAreaDataTypeFlag::MiscData(frame) => {
                let mut val: u8 = 9;
                if *frame == FrameChangeHandling::Incremental { val |= 1 << 7; }
                val
            }
            CorticalAreaDataTypeFlag::CartesianPlane(frame) => {
                let mut val: u8 = 10;
                if *frame == FrameChangeHandling::Incremental { val |= 1 << 7; }
                val
            }
        }
    }
}



/// How each data frame is made
#[derive(PartialEq, Hash)]
pub enum FrameChangeHandling {
    /// each frame shows the exact data of that instant
    Absolute,
    /// each frame is essentially a delta with the frame before it
    Incremental,
}

/// For percentage data, how do we position the active neuron voxel?
#[derive(PartialEq, Hash)]
pub enum PercentageNeuronPositioning {
    /// On a linear 0-1 scale with the percentage
    Linear,
    /// A fractional / exponential scale, where each neuron voxel represents half of the
    /// value of the neuron before, starting at 0.5
    Fractional,
}


//endregion