//! Defines the device type the data resides in. CPU is always supported, but other crates may add
//! their own devices

#[doc(hidden)]
/// Shared trait among all Feagi ECS Device traits
pub trait FeagiECSCollectionDataLivesOnDeviceBase {}

/// Defines that the collection data lives on the CPU / RAM
pub trait FeagiECSCollectionDataLivesOnCPU: FeagiECSCollectionDataLivesOnDeviceBase {}

// Other devices should be defined in other crates

//region Collection Data Sync

//region CPU -> Other
pub enum IsCollectionDataInSync {
    DataSynced,
    CPUIsNewer,
    OtherIsNewer,
    Unknown
}


/// For a struct holding 2 collections, them being the same thing but one being a CPU
/// implementation, the other something else. There is sync access to the CPU to allow
/// immediate writes, though writes are not immediately passed to the other implementation.
/// That needs a separate action from the system managing the sync. Ergo, this is a sync from
/// CPU -> other device.
pub trait FeagiECSDataFlowsOutOfCPUSync<
    CPUCollection: FeagiECSCollectionDataLivesOnCPU,
    OtherCollection: FeagiECSCollectionDataLivesOnDeviceBase
>
{
    // Setting the sync flag is up to the specific implementation

    fn get_sync_flag(&self) -> &IsCollectionDataInSync;

    fn get_cpu_data_ref(&self) -> (&CPUCollection, &IsCollectionDataInSync);

    fn get_cpu_data_ref_mut(&mut self) -> (&mut CPUCollection, &IsCollectionDataInSync);
}

/// Denotes a "fake" implementation of FeagiECSDataFlowsOutOfCPUSync where in actuality only
/// one collection exists, the cpu one. This is a stand in for any area that takes a
/// FeagiECSDataFlowsOutOfCPUSync to enable easier composition.
pub trait FeagiECSDataFlowsOutOfCPUSyncFaux<CPUCollection: FeagiECSCollectionDataLivesOnCPU>:
FeagiECSDataFlowsOutOfCPUSync<
    CPUCollection,
    CPUCollection
>
{

}

//endregion


//region Other -> CPU

/// For a struct holding 2 collections, them being the same thing but one being a CPU
/// implementation, the other something else. There is sync access to the CPU to allow
/// immediate reads, though data is not written to the CPU collection instantly with the other
/// collection. That needs a separate action from the system managing the sync. Ergo, this is a
/// sync from the other device -> CPU.
pub trait FeagiECSDataFlowsIntoCPUSync<
    CPUCollection: FeagiECSCollectionDataLivesOnCPU,
    OtherCollection: FeagiECSCollectionDataLivesOnDeviceBase
>
{
    // Setting the sync flag is up to the specific implementation

    fn get_sync_flag(&self) -> &IsCollectionDataInSync;

    fn get_cpu_data_ref(&self) -> (&CPUCollection, &IsCollectionDataInSync);

    // No mut access, the other device handles writing
}

/// Denotes a "fake" implementation of FeagiECSDataFlowsIntoCPUSync where in actuality only
/// one collection exists, the cpu one. This is a stand in for any area that takes a
/// FeagiECSDataFlowsIntoCPUSync to enable easier composition.
pub trait FeagiECSDataFlowsIntoCPUSyncFaux<CPUCollection: FeagiECSCollectionDataLivesOnCPU>:
FeagiECSDataFlowsIntoCPUSync<
    CPUCollection,
    CPUCollection
>
{

}

//endregion

//endregion