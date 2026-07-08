use feagi_data::{create_wrapped_contiguous_slice, create_wrapped_contiguous_slice_mut, create_wrapped_contiguous_vector, create_wrapped_quantized_decimal, create_wrapped_quantized_index, create_wrapped_quantized_index_coordinate, create_wrapped_quantized_index_dimension};
use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelCoordinateAxis;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;
use feagi_data::values::spatial::quantizable_index::{QuantizedIndexCoord4D, QuantizedIndexDimension4D};

// TODO the following macro should be common througouh

/// Given a name and level, creates a linear index type, and slice, slicemut, vector structs of it
macro_rules! make_index_and_linear_collections {
    (
        $name:ident
    ) => {
        ::paste::paste! {
            create_wrapped_quantized_index!(
                pub [<$name Index>]
            );

            create_wrapped_contiguous_slice!(
                pub [<$name IndexedSlice>],
                [<$name Index>]
            );

            create_wrapped_contiguous_slice_mut!(
                pub [<$name IndexedSliceMut>],
                [<$name Index>],
                [<$name IndexedSlice>]
            );

            create_wrapped_contiguous_vector!(
                pub [<$name IndexedVector>],
                [<$name Index>],
                [<$name IndexedSlice>],
                [<$name IndexedSliceMut>]
            );
        }

    };
}


create_wrapped_quantized_index!(
    /// The current burst index of a given engine
    pub BurstIndex
);

impl<Q: QuantizedIndexCountTrait> BurstIndex<Q> {
    pub fn new_from_middle() -> Self {
        Self(Q::QUANT_MAX / (Q::QUANT_ONE + Q::QUANT_ONE))
    }
}

//region Connectome Level Indexing
make_index_and_linear_collections!(CorticalConnectome);
//endregion



//region Engine Level Indexing
make_index_and_linear_collections!(CorticalEngine);
make_index_and_linear_collections!(NeuronEngine);
//endregion


//region Neuron History Enabled Level Indexing
make_index_and_linear_collections!(NeuronWithHistory);
//endregion


//region MP Quantization + Engine Level Indexing
make_index_and_linear_collections!(NeuronMP);
//endregion



//region Neuron Model + MP Quantization + Engine Level Indexing
make_index_and_linear_collections!(CorticalModel);
make_index_and_linear_collections!(NeuronModel);
//endregion



//region Cortical Local Level Indexing
make_index_and_linear_collections!(NeuronCorticalLocal);
//endregion



// TODO shouldnt these all not be NPU specific?
create_wrapped_quantized_index!(
    /// Index of a neuron within a voxel. Most voxels only have 1 neuron, but some have more
    pub NeuronVoxelDensityIndex
);

create_wrapped_quantized_index_coordinate!(
    /// Represents a 4D coordinate of a neuron within a dimensional cortical_area area, with the
    /// 4th dimension being the density index
    pub DimensionCorticalAreaCoordinate,
    QuantizedIndexCoord4D,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis), (3, d, NeuronVoxelDensityIndex)
);

create_wrapped_quantized_index_dimension!(
    /// Represents the dimensions and density of a dimensional cortical_area area
    pub DimensionalCorticalAreaDimensions,
    QuantizedIndexDimension4D,
    DimensionCorticalAreaCoordinate,
    NeuronCorticalLocalIndex,
    (0, x, NeuronVoxelCoordinateAxis), (1, y, NeuronVoxelCoordinateAxis), (2, z, NeuronVoxelCoordinateAxis), (3, d, NeuronVoxelDensityIndex)
);