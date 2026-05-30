use crate::collection::{FeagiECSCollection};
use crate::element::FeagiECSElement;
use crate::tag_device::FeagiECSTagCPU;

/// Used to designate if data between the cpu and the external device is in sync
#[derive(Debug, PartialEq, Hash, Eq, Copy, Clone)]
pub enum ECSCollectionDataSyncStatus {
    DataSynced,
    CPUIsNewer,
    OtherIsNewer,
    Unknown
}

impl ECSCollectionDataSyncStatus {
    pub fn is_not_synced(&self) -> bool
    {
        self != &ECSCollectionDataSyncStatus::DataSynced
    }
}

//region CPU -> Other

/// Specifies a sync from CPU -> another device. For a struct holding 2 collections, them being the
/// same thing but one being a CPU implementation, the other something else. There is sync access 
/// to the CPU to allow immediate writes, though writes are not immediately passed to the other 
/// implementation. That needs a separate action from the system managing the sync.
pub trait FeagiECSDataFlowsOutOfCPUSync<ECSElement, CPUCollection, OtherCollection>
where
    ECSElement: FeagiECSElement,
    CPUCollection: FeagiECSCollection<ECSElement> + FeagiECSTagCPU,
    OtherCollection: FeagiECSCollection<ECSElement>
    

{
    // Setting the sync flag is up to the specific implementation

    fn get_sync_flag(&self) -> &ECSCollectionDataSyncStatus;

    fn get_cpu_data_ref(&self) -> (&CPUCollection, &ECSCollectionDataSyncStatus);

    fn get_cpu_data_ref_mut(&mut self) -> (&mut CPUCollection, &ECSCollectionDataSyncStatus);
}

/// Denotes a "fake" implementation of FeagiECSDataFlowsOutOfCPUSync where in actuality only
/// one collection exists, the cpu one. This is a stand in for any area that takes a
/// FeagiECSDataFlowsOutOfCPUSync to enable easier composition.
pub trait FeagiECSDataFlowsOutOfCPUSyncFaux<ECSElement, CPUCollection>:
FeagiECSDataFlowsOutOfCPUSync<ECSElement, CPUCollection, CPUCollection>
where
    ECSElement: FeagiECSElement,
    CPUCollection: FeagiECSCollection<ECSElement> + FeagiECSTagCPU,
{

}
//endregion


//region Other -> CPU

/// For a struct holding 2 collections, them being the same thing but one being a CPU
/// implementation, the other something else. There is sync access to the CPU to allow
/// immediate reads, though data is not written to the CPU collection instantly with the other
/// collection. That needs a separate action from the system managing the sync. Ergo, this is a
/// sync from the other device -> CPU.
pub trait FeagiECSDataFlowsIntoCPUSync<ECSElement, OtherCollection, CPUCollection>
where
    ECSElement: FeagiECSElement,
    OtherCollection: FeagiECSCollection<ECSElement>,
    CPUCollection: FeagiECSCollection<ECSElement> + FeagiECSTagCPU,
    
{
    // Setting the sync flag is up to the specific implementation

    fn get_sync_flag(&self) -> &ECSCollectionDataSyncStatus;

    fn get_cpu_data_ref(&self) -> (&CPUCollection, &ECSCollectionDataSyncStatus);

    // No mut access, the other device handles writing
}

/// Denotes a "fake" implementation of FeagiECSDataFlowsIntoCPUSync where in actuality only
/// one collection exists, the cpu one. This is a stand in for any area that takes a
/// FeagiECSDataFlowsIntoCPUSync to enable easier composition.
pub trait FeagiECSDataFlowsIntoCPUSyncFaux<ECSElement, CPUCollection>:
FeagiECSDataFlowsIntoCPUSync<ECSElement, CPUCollection, CPUCollection>
where
    ECSElement: FeagiECSElement,
    CPUCollection: FeagiECSCollection<ECSElement> + FeagiECSTagCPU,
{

}

//endregion

//endregion