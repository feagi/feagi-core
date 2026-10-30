

// TODO macro for sensor motor should bea  template that can generate nested enums here

use crate::basis::cortical_area::classes::sub_classes::core_cortical_area_type::CoreCorticalClass;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
pub enum CorticalAreaClassEnum {
    Core(CoreCorticalClass),
    Interconnect,
    //Memory,
    //BrainInput(),
    //BrainOutput(),
}

impl CorticalAreaClassEnum {
    

    
    
}



/*

impl core::fmt::Display for CorticalAreaType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CorticalAreaType::Core(c) => write!(f, "Core({})", c),
            CorticalAreaType::Interconnect(c) => write!(f, "Interconnect({})", c),
            CorticalAreaType::Memory(c) => write!(f, "Memory({})", c),
            CorticalAreaType::BrainInput() => write!(f, "BrainInput()", ),
            CorticalAreaType::BrainOutput() => write!(f, "BrainOutput()", ),
        }
    }
}


 */