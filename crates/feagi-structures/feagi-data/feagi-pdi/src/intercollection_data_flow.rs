use crate::main_traits::{PDICollection, PDIElement, PDIExecutor};
use crate::tag_device::PDITagCPU;

/// Used to designate if data between the cpu and the external device is in sync
#[derive(Debug, PartialEq, Hash, Eq, Copy, Clone)]
pub enum PDICollectionDataSyncStatus {
    DataSynced,
    CPUIsNewer,
    OtherIsNewer,
    Unknown
}

impl PDICollectionDataSyncStatus {
    pub fn is_not_synced(&self) -> bool
    {
        self != &PDICollectionDataSyncStatus::DataSynced
    }
}

//region CPU -> Other

/// Specifies a sync from CPU -> another device. For a struct holding 2 collections, them being the
/// same thing but one being a CPU implementation, the other something else. There is sync access 
/// to the CPU to allow immediate writes, though writes are not immediately passed to the other 
/// implementation. That needs a separate action from the system managing the sync.
pub trait PDIDataFlowsOutOfCPUSync<PDIE, CPUCollection, OtherCollection>: PDIExecutor
where
    PDIE: PDIElement,
    CPUCollection: PDICollection + PDITagCPU,
    OtherCollection: PDICollection


{
    // Setting the sync flag is up to the specific implementation

    fn get_sync_flag(&self) -> &PDICollectionDataSyncStatus;

    fn get_cpu_data_ref(&self) -> (&CPUCollection, &PDICollectionDataSyncStatus);

    fn get_cpu_data_ref_mut(&mut self) -> (&mut CPUCollection, &PDICollectionDataSyncStatus);
}

/// Denotes a "fake" implementation of PDIDataFlowsOutOfCPUSync where in actuality only
/// one collection exists, the cpu one. This is a stand in for any area that takes a
/// PDIDataFlowsOutOfCPUSync to enable easier composition.
pub trait PDIDataFlowsOutOfCPUSyncFaux<PDIE, CPUCollection>:
PDIDataFlowsOutOfCPUSync<PDIE, CPUCollection, CPUCollection>
where
    PDIE: PDIElement,
    CPUCollection: PDICollection + PDITagCPU,
{

}
//endregion


//region Other -> CPU

/// For a struct holding 2 collections, them being the same thing but one being a CPU
/// implementation, the other something else. There is sync access to the CPU to allow
/// immediate reads, though data is not written to the CPU collection instantly with the other
/// collection. That needs a separate action from the system managing the sync. Ergo, this is a
/// sync from the other device -> CPU.
pub trait PDIDataFlowsIntoCPUSync<PDIE, OtherCollection, CPUCollection>: PDIExecutor
where
    PDIE: PDIElement,
    CPUCollection: PDICollection + PDITagCPU,
    OtherCollection: PDICollection

{
    // Setting the sync flag is up to the specific implementation

    fn get_sync_flag(&self) -> &PDICollectionDataSyncStatus;

    fn get_cpu_data_ref(&self) -> (&CPUCollection, &PDICollectionDataSyncStatus);

    // No mut access, the other device handles writing
}

/// Denotes a "fake" implementation of PDIDataFlowsIntoCPUSync where in actuality only
/// one collection exists, the cpu one. This is a stand in for any area that takes a
/// PDIDataFlowsIntoCPUSync to enable easier composition.
pub trait PDIDataFlowsIntoCPUSyncFaux<PDIE, CPUCollection>:
PDIDataFlowsIntoCPUSync<PDIE, CPUCollection, CPUCollection>
where
    PDIE: PDIElement,
    CPUCollection: PDICollection + PDITagCPU,
{

}

//endregion

//endregion