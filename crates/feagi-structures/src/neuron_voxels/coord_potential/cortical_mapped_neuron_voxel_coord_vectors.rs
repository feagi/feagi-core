//! Aggregates per-cortical-area [`NeuronVoxelCoordVector`] collections keyed by [`CorticalID`].
//!
//! This is the quantization-generic successor to the pre-refactor
//! `CorticalMappedXYZPNeuronVoxels` type. It is the canonical container used by
//! downstream serialization and I/O crates that need to move neuron voxel data
//! across the wire for multiple cortical areas in a single payload.
//!
//! The type is parameterized over the four primitive quantization parameters
//! used by [`MultiCorticalNeuronVoxelCollectionBase`] / [`MultiCorticalNeuronVoxelCollectionAlloc`].
//! Callers that have an `NPUQuantization` in scope should pass the associated
//! types (`Q::Value`, `Q::Coord`, `Q::NeuronIndex`, `Q::CorticalIndex`) at construction.

#[cfg(feature = "alloc")]
use ahash::AHashMap;
use core::marker::PhantomData;

use crate::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use crate::genomic::cortical_area::CorticalID;
use crate::neuron_voxels::coord_potential::NeuronVoxelCoordVector;
use crate::neuron_voxels::descriptors::SingleCorticalNeuronVoxelCollectionType;
use crate::neuron_voxels::traits::{
    MultiCorticalNeuronVoxelCollectionAlloc,
    MultiCorticalNeuronVoxelCollectionBase,
    SingleCorticalNeuronVoxelCollectionBase,
};
use crate::neuron_voxels::FeagiStructuresNeuronVoxelError;

/// Sparse per-cortical-area mapping of [`NeuronVoxelCoordVector`] collections.
///
/// Primarily intended as the in-memory representation for serializing or
/// transmitting neuron voxel snapshots spanning multiple cortical areas.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub struct CorticalMappedNeuronVoxelCoordVectors<
    VoxelPotentialQuant,
    CoordQuant,
    NeuronVoxelIndexQuant,
    CorticalAreaIndexQuant,
> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
    CorticalAreaIndexQuant: QuantizableUIntType,
{
    /// Hash map storing per-cortical-area neuron voxel collections.
    pub mappings: AHashMap<
        CorticalID,
        NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>,
    >,
    /// Kept to satisfy the trait bound; has no runtime representation.
    _cortical_area_index_quant: PhantomData<CorticalAreaIndexQuant>,
    /// Cached list of contained cortical IDs, kept in sync with `mappings` so the
    /// `MultiCorticalNeuronVoxelCollectionBase::get_contained_cortical_area_ids`
    /// contract (returning `&[CorticalID]`) can be satisfied without allocating per call.
    cached_ids: Vec<CorticalID>,
    /// Cached per-CA collection-type map to satisfy the corresponding trait method.
    cached_types: AHashMap<CorticalID, SingleCorticalNeuronVoxelCollectionType>,
}

#[cfg(feature = "alloc")]
impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>
    CorticalMappedNeuronVoxelCoordVectors<
        VoxelPotentialQuant,
        CoordQuant,
        NeuronVoxelIndexQuant,
        CorticalAreaIndexQuant,
    >
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
    CorticalAreaIndexQuant: QuantizableUIntType,
{
    /// Byte size of a single per-cortical-area header in the serialization format:
    /// cortical id + start-byte-offset (u32) + byte-count (u32).
    pub const NUMBER_BYTES_PER_CORTICAL_ID_HEADER: usize =
        CorticalID::NUMBER_OF_BYTES + core::mem::size_of::<u32>() + core::mem::size_of::<u32>();

    /// Byte size of the cortical-count prefix in the serialization format.
    pub const NUMBER_BYTES_CORTICAL_COUNT_HEADER: usize = core::mem::size_of::<u16>();

    /// Creates an empty mapping.
    pub fn new() -> Self {
        Self {
            mappings: AHashMap::new(),
            _cortical_area_index_quant: PhantomData,
            cached_ids: Vec::new(),
            cached_types: AHashMap::new(),
        }
    }

    /// Creates an empty mapping with pre-allocated capacity for `capacity` cortical areas.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            mappings: AHashMap::with_capacity(capacity),
            _cortical_area_index_quant: PhantomData,
            cached_ids: Vec::with_capacity(capacity),
            cached_types: AHashMap::with_capacity(capacity),
        }
    }

    /// Number of cortical areas currently stored.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Whether no cortical areas are stored.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Capacity of the underlying map (number of cortical areas before rehashing).
    pub fn capacity(&self) -> usize {
        self.mappings.capacity()
    }

    /// Returns `true` if `cortical_id` has an entry.
    pub fn contains_key(&self, cortical_id: &CorticalID) -> bool {
        self.mappings.contains_key(cortical_id)
    }

    /// Borrow the collection for a cortical area.
    pub fn get(
        &self,
        cortical_id: &CorticalID,
    ) -> Option<&NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>>
    {
        self.mappings.get(cortical_id)
    }

    /// Mutably borrow the collection for a cortical area.
    pub fn get_mut(
        &mut self,
        cortical_id: &CorticalID,
    ) -> Option<&mut NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>>
    {
        self.mappings.get_mut(cortical_id)
    }

    /// Insert/replace the collection for a cortical area. Returns any prior collection.
    pub fn insert(
        &mut self,
        cortical_id: CorticalID,
        collection: NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>,
    ) -> Option<NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>>
    {
        let previous = self.mappings.insert(cortical_id, collection);
        if previous.is_none() {
            self.cached_ids.push(cortical_id);
        }
        self.cached_types.insert(
            cortical_id,
            SingleCorticalNeuronVoxelCollectionType::CoordVector,
        );
        previous
    }

    /// Remove and return the collection for a cortical area (if present).
    pub fn remove(
        &mut self,
        cortical_id: &CorticalID,
    ) -> Option<NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>>
    {
        let removed = self.mappings.remove(cortical_id);
        if removed.is_some() {
            self.cached_ids.retain(|id| id != cortical_id);
            self.cached_types.remove(cortical_id);
        }
        removed
    }

    /// Iterate over `(cortical_id, collection)` pairs.
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            &CorticalID,
            &NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>,
        ),
    > {
        self.mappings.iter()
    }

    /// Mutable iterator over `(cortical_id, collection)` pairs.
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<
        Item = (
            &CorticalID,
            &mut NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>,
        ),
    > {
        self.mappings.iter_mut()
    }
}

#[cfg(feature = "alloc")]
impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant> Default
    for CorticalMappedNeuronVoxelCoordVectors<
        VoxelPotentialQuant,
        CoordQuant,
        NeuronVoxelIndexQuant,
        CorticalAreaIndexQuant,
    >
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
    CorticalAreaIndexQuant: QuantizableUIntType,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>
    MultiCorticalNeuronVoxelCollectionBase<
        VoxelPotentialQuant,
        CoordQuant,
        NeuronVoxelIndexQuant,
        CorticalAreaIndexQuant,
    >
    for CorticalMappedNeuronVoxelCoordVectors<
        VoxelPotentialQuant,
        CoordQuant,
        NeuronVoxelIndexQuant,
        CorticalAreaIndexQuant,
    >
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
    CorticalAreaIndexQuant: QuantizableUIntType,
{
    fn get_contained_cortical_collection_type(
        &self,
        cortical_id: &CorticalID,
    ) -> Result<&SingleCorticalNeuronVoxelCollectionType, FeagiStructuresNeuronVoxelError> {
        self.cached_types.get(cortical_id).ok_or(
            FeagiStructuresNeuronVoxelError::NoCorticalIDInNeuronCollection {
                context: "CorticalMappedNeuronVoxelCoordVectors::get_contained_cortical_collection_type",
                cortical_id: *cortical_id,
            },
        )
    }

    fn get_contained_cortical_area_ids(&self) -> &[CorticalID] {
        &self.cached_ids
    }

    fn get_base_collection_implementation(
        &self,
        cortical_id: &CorticalID,
    ) -> Result<
        &impl SingleCorticalNeuronVoxelCollectionBase<
            VoxelPotentialQuant,
            CoordQuant,
            NeuronVoxelIndexQuant,
        >,
        FeagiStructuresNeuronVoxelError,
    > {
        self.mappings.get(cortical_id).ok_or(
            FeagiStructuresNeuronVoxelError::NoCorticalIDInNeuronCollection {
                context: "CorticalMappedNeuronVoxelCoordVectors::get_base_collection_implementation",
                cortical_id: *cortical_id,
            },
        )
    }

    fn get_base_collection_implementation_mut(
        &mut self,
        cortical_id: &CorticalID,
    ) -> Result<
        &mut impl SingleCorticalNeuronVoxelCollectionBase<
            VoxelPotentialQuant,
            CoordQuant,
            NeuronVoxelIndexQuant,
        >,
        FeagiStructuresNeuronVoxelError,
    > {
        let cortical_id_copy = *cortical_id;
        self.mappings.get_mut(cortical_id).ok_or(
            FeagiStructuresNeuronVoxelError::NoCorticalIDInNeuronCollection {
                context:
                    "CorticalMappedNeuronVoxelCoordVectors::get_base_collection_implementation_mut",
                cortical_id: cortical_id_copy,
            },
        )
    }
}

#[cfg(feature = "alloc")]
impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>
    MultiCorticalNeuronVoxelCollectionAlloc<
        VoxelPotentialQuant,
        CoordQuant,
        NeuronVoxelIndexQuant,
        CorticalAreaIndexQuant,
    >
    for CorticalMappedNeuronVoxelCoordVectors<
        VoxelPotentialQuant,
        CoordQuant,
        NeuronVoxelIndexQuant,
        CorticalAreaIndexQuant,
    >
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
    CorticalAreaIndexQuant: QuantizableUIntType,
{
    fn get_contained_cortical_collection_types(
        &self,
    ) -> &AHashMap<CorticalID, SingleCorticalNeuronVoxelCollectionType> {
        &self.cached_types
    }

    fn remove_by_cortical_id(
        &mut self,
        cortical_id: &CorticalID,
    ) -> Result<(), FeagiStructuresNeuronVoxelError> {
        match self.remove(cortical_id) {
            Some(_) => Ok(()),
            None => Err(
                FeagiStructuresNeuronVoxelError::NoCorticalIDInNeuronCollection {
                    context: "CorticalMappedNeuronVoxelCoordVectors::remove_by_cortical_id",
                    cortical_id: *cortical_id,
                },
            ),
        }
    }
}
