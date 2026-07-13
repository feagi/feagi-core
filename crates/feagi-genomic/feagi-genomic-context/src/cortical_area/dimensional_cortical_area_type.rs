// TODO methods to convert cortical_area area type to this

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum DimensionCorticalAreaType {
    Sensor,
    Motor,
    Core,
    Custom,
}
