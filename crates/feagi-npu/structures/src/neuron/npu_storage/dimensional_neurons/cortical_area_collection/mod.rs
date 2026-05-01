/// Cortical Area Collections are generic Neuron Data collection handlers,
/// tailored to various devices.

mod resizable_cortical_area_collection_ram;

#[cfg(feature = "alloc")]
pub(crate) use resizable_cortical_area_collection_ram::ResizableCorticalAreaCollectionRam;