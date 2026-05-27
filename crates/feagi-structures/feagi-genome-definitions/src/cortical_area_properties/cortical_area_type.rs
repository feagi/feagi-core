use crate::cortical_area_properties::cortical_area_types::{CoreCorticalType, CustomCorticalType, MemoryCorticalType};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CorticalAreaType {
    Core(CoreCorticalType),
    Custom(CustomCorticalType),
    Memory(MemoryCorticalType),
    BrainInput(), // TODO perhaps we should have the data types?
    BrainOutput(),
}
