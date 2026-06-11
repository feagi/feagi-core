use crate::neural_processing_unit_data_structures::packed_cortical_descriptor::PackedCorticalDescriptor;

#[repr(u8)]
#[derive(Copy, Clone, Default)]
pub enum CorticalConfigurationType {
    #[default]
    Dimensional = 0,
    Memory = 64,
}

impl CorticalConfigurationType
{
    pub(crate) const BYTE_MASK: u8 = 192; // First 2 bits
}

impl From<PackedCorticalDescriptor> for CorticalConfigurationType {
    fn from(packed_cortical_descriptor: PackedCorticalDescriptor) -> Self
    {

        let bits = packed_cortical_descriptor.into() & CorticalConfigurationType::BYTE_MASK;
        match bits {
            0 => CorticalConfigurationType::Dimensional, // 0 0
            64 => CorticalConfigurationType::Memory, // 0 1
            128 => panic!("Cortical configuration 128 Not Implemented!"), // 1 0
            192 => panic!("Cortical configuration 192 Not Implemented!"), // 1 1
            _ => panic!("Nonvalid value decoded for CorticalConfigurationType!"),
        }

    }
}
