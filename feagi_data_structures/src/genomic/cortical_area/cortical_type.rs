use crate::genomic::cortical_area::io_cortical_area_data_type::IOCorticalAreaDataType;

pub enum CorticalType {
    Custom,
    Memory,
    Core(CoreCorticalType),
    BrainInput(IOCorticalAreaDataType),
    BrainOutput(IOCorticalAreaDataType)
}


pub enum CoreCorticalType {
    Power,
    Death
}