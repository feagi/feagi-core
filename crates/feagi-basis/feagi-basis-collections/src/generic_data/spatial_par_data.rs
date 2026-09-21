use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use serde::{Deserialize, Serialize};

use crate::feagi_collection_error::{FeagiDataCollectionError, FeagiFailInvalidDimensions};
use crate::generic_data::par_data::{GenericParData, GenericParDataMut, ParDataArray};
use crate::spatial_indexing_structs::axis_order::AxisOrderIdentifier;
use crate::spatial_indexing_structs::SpatialCoordinate;
use crate::spatial_indexing_structs::SpatialDimensions;
use crate::spatial_indexing_structs::spatial_indexing_helpers::SpatialIndexingHelper;

#[cfg(feature = "heapless")]
use crate::generic_data::par_data::ParDataHeaplessVec;
#[cfg(feature = "heapless")]
use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use crate::generic_data::par_data::ParDataVector;
use crate::spatial_indexing_structs::helper_implementations::owning_spatial_indexing_helper::OwningSpatialIndexingHelper;

/// Spatial nonmut access to elements
pub trait GenericSpatialParData<QLinear, GPD, SIH, D, const NUM_DIMS: usize, const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    GPD: GenericParData<QLinear, D>,
    SIH: SpatialIndexingHelper<QLinear, NUM_DIMS, AXIS_ORDER_IDENTIFIER>,
    D: core::fmt::Debug,
{
    fn get_data_view<'a>(&'a self) -> &'a GPD
    where
        GPD: 'a;

    fn get_indexing_helper<'a>(&'a self) -> &'a SIH
    where
        SIH: 'a;

    //region Default Impls

    fn get_dimensions<'a>(&'a self) -> &'a SpatialDimensions<SIH::DimensionsQuant, NUM_DIMS>
    where
        SIH: 'a,
    {
        self.get_indexing_helper().get_dimensions()
    }

    /// Gets the element at the given coordinate or None if out of bounds
    fn get_coord<'a>(&'a self, coordinate: SpatialCoordinate<SIH::CoordinateQuant, NUM_DIMS>) -> Option<&'a D>
    where
        GPD: 'a,
    {
        let linear = self.get_indexing_helper().coordinate_to_linear(&coordinate);
        self.get_data_view().get(linear)
    }

    /// Iterates each coordinate with its corresponding data element, in linear index order.
    fn iter_coord_data<'a>(&'a self) -> impl Iterator<Item = (SpatialCoordinate<SIH::CoordinateQuant, NUM_DIMS>, &'a D)> + 'a
    where
        GPD: 'a,
        SIH: 'a,
        D: 'a,
    {
        let coordinate_iter = self.get_indexing_helper().iter_coordinates();
        let data_iter = self.get_data_view().iter();
        coordinate_iter.zip(data_iter)
    }

    /// Parallel iterator over each coordinate and its corresponding data element.
    #[cfg(feature = "expose_rayon")]
    fn rayon_iter_coord_data<'a>(
        &'a self,
    ) -> rayon::iter::Map<rayon::range::Iter<usize>, impl Fn(usize) -> (SpatialCoordinate<SIH::CoordinateQuant, NUM_DIMS>, &'a D) + Send + Sync>
    where
        GPD: 'a,
        SIH: 'a,
        D: 'a + Sync,
        QLinear: Send,
        SIH::CoordinateQuant: Send,
    {
        use rayon::prelude::*;

        let helper = self.get_indexing_helper();
        let total = helper.get_dimensions().spatial_element_count();
        let stride = *helper._get_stride();
        let dimensions = *helper.get_dimensions();
        let data = self.get_data_view().as_slice();

        (0..total).into_par_iter().map(move |index| {
            let linear = QLinear::quant_from_usize_unchecked(index);
            let coordinate = stride.linear_to_coordinate::<QLinear, SIH::CoordinateQuant, SIH::DimensionsQuant>(linear, &dimensions);
            (coordinate, &data[index])
        })
    }

    //endregion
}

/// Mutable access to spatial par data elements only
pub trait GenericSpatialParDataMut<QLinear, GPD, SIH, D, const NUM_DIMS: usize, const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier>:
    GenericSpatialParData<QLinear, GPD, SIH, D, NUM_DIMS, AXIS_ORDER_IDENTIFIER>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    GPD: GenericParDataMut<QLinear, D>,
    SIH: SpatialIndexingHelper<QLinear, NUM_DIMS, AXIS_ORDER_IDENTIFIER>,
    D: core::fmt::Debug,
{
    fn get_data_view_mut<'a>(&'a mut self) -> &'a mut GPD
    where
        GPD: 'a;

    //region Default Impls

    /// Mutably borrows the element at the given coordinate, or `None` if out of bounds.
    fn get_coord_mut<'a>(&'a mut self, coordinate: SpatialCoordinate<SIH::CoordinateQuant, NUM_DIMS>) -> Option<&'a mut D>
    where
        GPD: 'a,
        D: 'a,
    {
        let linear = GenericSpatialParData::get_indexing_helper(&*self).coordinate_to_linear(&coordinate);
        self.get_data_view_mut().get_mut(linear)
    }

    /// Iterates each coordinate with its corresponding mutable data element, in linear index order.
    fn iter_coord_data_mut<'a>(&'a mut self) -> impl Iterator<Item = (SpatialCoordinate<SIH::CoordinateQuant, NUM_DIMS>, &'a mut D)> + 'a
    where
        GPD: 'a,
        D: 'a,
    {
        let helper = GenericSpatialParData::get_indexing_helper(&*self);
        let stride = *helper._get_stride();
        let dimensions = *helper.get_dimensions();
        self.get_data_view_mut().iter_mut().enumerate().map(move |(index, value)| {
            let linear = QLinear::quant_from_usize_unchecked(index);
            let coordinate = stride.linear_to_coordinate::<QLinear, SIH::CoordinateQuant, SIH::DimensionsQuant>(linear, &dimensions);
            (coordinate, value)
        })
    }

    /// Parallel iterator over each coordinate and its corresponding mutable data element.
    #[cfg(feature = "expose_rayon")]
    fn rayon_iter_coord_data_mut<'a>(
        &'a mut self,
    ) -> rayon::iter::Map<
        rayon::iter::Enumerate<rayon::slice::IterMut<'a, D>>,
        impl Fn((usize, &'a mut D)) -> (SpatialCoordinate<SIH::CoordinateQuant, NUM_DIMS>, &'a mut D) + Send,
    >
    where
        GPD: 'a,
        D: 'a + Send + Sync,
        QLinear: Send,
        SIH::CoordinateQuant: Send,
    {
        use rayon::prelude::*;

        let helper = GenericSpatialParData::get_indexing_helper(&*self);
        let stride = *helper._get_stride();
        let dimensions = *helper.get_dimensions();
        self.get_data_view_mut()
            .as_mut_slice()
            .par_iter_mut()
            .enumerate()
            .map(move |(index, value)| {
                let linear = QLinear::quant_from_usize_unchecked(index);
                let coordinate = stride.linear_to_coordinate::<QLinear, SIH::CoordinateQuant, SIH::DimensionsQuant>(linear, &dimensions);
                (coordinate, value)
            })
    }

    //endregion
}

//region Implementations

fn validate_data_len_matches_dimensions<QDim: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>(
    data_len: usize,
    dimensions: &SpatialDimensions<QDim, NUM_DIMS>,
) -> Result<(), FeagiDataCollectionError> {
    let expected = dimensions.spatial_element_count();
    if data_len != expected {
        return Err(FeagiFailInvalidDimensions::new("data element count does not match spatial dimensions volume").into());
    }
    Ok(())
}

//region Vector

/// Owned heap-backed spatial par data with an embedded [`OwningSpatialIndexingHelper`].
#[cfg(feature = "alloc")]
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "ParDataVector<QLinear, D>: Serialize, OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>: Serialize",
    deserialize = "ParDataVector<QLinear, D>: Deserialize<'de>, OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>: Deserialize<'de>"
))]
pub struct SpatialParDataVector<
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    D: core::fmt::Debug,
    const NUM_DIMS: usize,
    const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
> {
    pub(crate) data: ParDataVector<QLinear, D>,
    pub(crate) indexing: OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>,
}

#[cfg(feature = "alloc")]
impl<
        QLinear: QuantizedUnsignedIntegerTrait,
        QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        D: core::fmt::Debug,
        const NUM_DIMS: usize,
        const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
    > SpatialParDataVector<QLinear, QCoord, QDim, D, NUM_DIMS, AXIS_ORDER_IDENTIFIER>
{
    /// Creates storage sized to the product of `dimensions`, filled with `initial_value`.
    pub fn new_uniform(dimensions: SpatialDimensions<QDim, NUM_DIMS>, initial_value: D) -> Result<Self, FeagiDataCollectionError>
    where
        D: Clone,
    {
        let element_count = dimensions.spatial_element_count();
        Ok(Self {
            data: ParDataVector::new_uniform(QLinear::quant_from_usize_unchecked(element_count), initial_value),
            indexing: OwningSpatialIndexingHelper::<QLinear, QCoord, QDim, NUM_DIMS>::from_dimensions(dimensions, AXIS_ORDER_IDENTIFIER),
        })
    }

    /// Wraps `data` after verifying its length matches the spatial volume of `dimensions`.
    pub fn try_from_vec(dimensions: SpatialDimensions<QDim, NUM_DIMS>, data: Vec<D>) -> Result<Self, FeagiDataCollectionError> {
        validate_data_len_matches_dimensions(data.len(), &dimensions)?;
        Ok(Self {
            data: ParDataVector::from_vec(data),
            indexing: OwningSpatialIndexingHelper::<QLinear, QCoord, QDim, NUM_DIMS>::from_dimensions(dimensions, AXIS_ORDER_IDENTIFIER),
        })
    }
}

#[cfg(feature = "alloc")]
impl<
        QLinear: QuantizedUnsignedIntegerTrait,
        QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        D: core::fmt::Debug,
        const NUM_DIMS: usize,
        const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
    >
    GenericSpatialParData<
        QLinear,
        ParDataVector<QLinear, D>,
        OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>,
        D,
        NUM_DIMS,
        AXIS_ORDER_IDENTIFIER,
    > for SpatialParDataVector<QLinear, QCoord, QDim, D, NUM_DIMS, AXIS_ORDER_IDENTIFIER>
{
    fn get_data_view<'a>(&'a self) -> &'a ParDataVector<QLinear, D>
    where
        ParDataVector<QLinear, D>: 'a,
    {
        &self.data
    }

    fn get_indexing_helper<'a>(&'a self) -> &'a OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>
    where
        OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>: 'a,
    {
        &self.indexing
    }
}

#[cfg(feature = "alloc")]
impl<
        QLinear: QuantizedUnsignedIntegerTrait,
        QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        D: core::fmt::Debug,
        const NUM_DIMS: usize,
        const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
    >
    GenericSpatialParDataMut<
        QLinear,
        ParDataVector<QLinear, D>,
        OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>,
        D,
        NUM_DIMS,
        AXIS_ORDER_IDENTIFIER,
    > for SpatialParDataVector<QLinear, QCoord, QDim, D, NUM_DIMS, AXIS_ORDER_IDENTIFIER>
{
    fn get_data_view_mut<'a>(&'a mut self) -> &'a mut ParDataVector<QLinear, D>
    where
        ParDataVector<QLinear, D>: 'a,
    {
        &mut self.data
    }
}

//endregion

//region Array

/// Owned fixed-size spatial par data with an embedded [`OwningSpatialIndexingHelper`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "ParDataArray<QLinear, D, N>: Serialize, OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>: Serialize",
    deserialize = "ParDataArray<QLinear, D, N>: Deserialize<'de>, OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>: Deserialize<'de>"
))]
pub struct SpatialParDataArray<
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    D: core::fmt::Debug,
    const NUM_DIMS: usize,
    const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
    const N: usize,
> {
    pub(crate) data: ParDataArray<QLinear, D, N>,
    pub(crate) indexing: OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>,
}

impl<
        QLinear: QuantizedUnsignedIntegerTrait,
        QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        D: core::fmt::Debug,
        const NUM_DIMS: usize,
        const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
        const N: usize,
    > SpatialParDataArray<QLinear, QCoord, QDim, D, NUM_DIMS, AXIS_ORDER_IDENTIFIER, N>
{
    /// Creates an array sized to `N`, filled with `initial_value`, when the spatial volume is `N`.
    pub fn new_uniform(dimensions: SpatialDimensions<QDim, NUM_DIMS>, initial_value: D) -> Result<Self, FeagiDataCollectionError>
    where
        D: Clone,
    {
        validate_data_len_matches_dimensions(N, &dimensions)?;
        Ok(Self {
            data: ParDataArray::new_uniform(initial_value),
            indexing: OwningSpatialIndexingHelper::<QLinear, QCoord, QDim, NUM_DIMS>::from_dimensions(dimensions, AXIS_ORDER_IDENTIFIER),
        })
    }

    /// Wraps `data` after verifying `N` matches the spatial volume of `dimensions`.
    pub fn try_from_array(dimensions: SpatialDimensions<QDim, NUM_DIMS>, data: [D; N]) -> Result<Self, FeagiDataCollectionError> {
        validate_data_len_matches_dimensions(N, &dimensions)?;
        Ok(Self {
            data: ParDataArray::from_array(data),
            indexing: OwningSpatialIndexingHelper::<QLinear, QCoord, QDim, NUM_DIMS>::from_dimensions(dimensions, AXIS_ORDER_IDENTIFIER),
        })
    }
}

impl<
        QLinear: QuantizedUnsignedIntegerTrait,
        QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        D: core::fmt::Debug,
        const NUM_DIMS: usize,
        const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
        const N: usize,
    >
    GenericSpatialParData<
        QLinear,
        ParDataArray<QLinear, D, N>,
        OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>,
        D,
        NUM_DIMS,
        AXIS_ORDER_IDENTIFIER,
    > for SpatialParDataArray<QLinear, QCoord, QDim, D, NUM_DIMS, AXIS_ORDER_IDENTIFIER, N>
{
    fn get_data_view<'a>(&'a self) -> &'a ParDataArray<QLinear, D, N>
    where
        ParDataArray<QLinear, D, N>: 'a,
    {
        &self.data
    }

    fn get_indexing_helper<'a>(&'a self) -> &'a OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>
    where
        OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>: 'a,
    {
        &self.indexing
    }
}

impl<
        QLinear: QuantizedUnsignedIntegerTrait,
        QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        D: core::fmt::Debug,
        const NUM_DIMS: usize,
        const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
        const N: usize,
    >
    GenericSpatialParDataMut<
        QLinear,
        ParDataArray<QLinear, D, N>,
        OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>,
        D,
        NUM_DIMS,
        AXIS_ORDER_IDENTIFIER,
    > for SpatialParDataArray<QLinear, QCoord, QDim, D, NUM_DIMS, AXIS_ORDER_IDENTIFIER, N>
{
    fn get_data_view_mut<'a>(&'a mut self) -> &'a mut ParDataArray<QLinear, D, N>
    where
        ParDataArray<QLinear, D, N>: 'a,
    {
        &mut self.data
    }
}

//endregion

//region Heapless Vector

#[cfg(feature = "heapless")]
/// Owned stack-capacity spatial par data with an embedded [`OwningSpatialIndexingHelper`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "ParDataHeaplessVec<QLinear, D, N>: Serialize, OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>: Serialize",
    deserialize = "ParDataHeaplessVec<QLinear, D, N>: Deserialize<'de>, OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>: Deserialize<'de>"
))]
pub struct SpatialParDataHeaplessVec<
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    D: core::fmt::Debug,
    const NUM_DIMS: usize,
    const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
    const N: usize,
> {
    pub(crate) data: ParDataHeaplessVec<QLinear, D, N>,
    pub(crate) indexing: OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>,
}

#[cfg(feature = "heapless")]
impl<
        QLinear: QuantizedUnsignedIntegerTrait,
        QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        D: core::fmt::Debug,
        const NUM_DIMS: usize,
        const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
        const N: usize,
    > SpatialParDataHeaplessVec<QLinear, QCoord, QDim, D, NUM_DIMS, AXIS_ORDER_IDENTIFIER, N>
{
    /// Creates heapless storage with length equal to the spatial volume, filled with `initial_value`.
    pub fn new_uniform(dimensions: SpatialDimensions<QDim, NUM_DIMS>, initial_value: D) -> Result<Self, FeagiDataCollectionError>
    where
        D: Clone,
    {
        let element_count = dimensions.spatial_element_count();
        if element_count > N {
            return Err(FeagiFailInvalidDimensions::new("spatial volume exceeds heapless vector capacity").into());
        }
        let mut data = ::heapless::Vec::<D, N>::new();
        for _ in 0..element_count {
            data.push(initial_value.clone())
                .map_err(|_| FeagiFailInvalidDimensions::new("spatial volume exceeds heapless vector capacity").into())?;
        }
        Ok(Self {
            data: ParDataHeaplessVec { data, _marker: PhantomData },
            indexing: OwningSpatialIndexingHelper::<QLinear, QCoord, QDim, NUM_DIMS>::from_dimensions(dimensions, AXIS_ORDER_IDENTIFIER),
        })
    }

    /// Wraps fixed array storage when its length `N` matches the spatial volume.
    pub fn try_from_array(dimensions: SpatialDimensions<QDim, NUM_DIMS>, data: [D; N]) -> Result<Self, FeagiDataCollectionError> {
        validate_data_len_matches_dimensions(N, &dimensions)?;
        Ok(Self {
            data: ParDataHeaplessVec::from_array(data),
            indexing: OwningSpatialIndexingHelper::<QLinear, QCoord, QDim, NUM_DIMS>::from_dimensions(dimensions, AXIS_ORDER_IDENTIFIER),
        })
    }
}

#[cfg(feature = "heapless")]
impl<
        QLinear: QuantizedUnsignedIntegerTrait,
        QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        D: core::fmt::Debug,
        const NUM_DIMS: usize,
        const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
        const N: usize,
    >
    GenericSpatialParData<
        QLinear,
        ParDataHeaplessVec<QLinear, D, N>,
        OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>,
        D,
        NUM_DIMS,
        AXIS_ORDER_IDENTIFIER,
    > for SpatialParDataHeaplessVec<QLinear, QCoord, QDim, D, NUM_DIMS, AXIS_ORDER_IDENTIFIER, N>
{
    fn get_data_view<'a>(&'a self) -> &'a ParDataHeaplessVec<QLinear, D, N>
    where
        ParDataHeaplessVec<QLinear, D, N>: 'a,
    {
        &self.data
    }

    fn get_indexing_helper<'a>(&'a self) -> &'a OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>
    where
        OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>: 'a,
    {
        &self.indexing
    }
}

#[cfg(feature = "heapless")]
impl<
        QLinear: QuantizedUnsignedIntegerTrait,
        QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        QDim: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
        D: core::fmt::Debug,
        const NUM_DIMS: usize,
        const AXIS_ORDER_IDENTIFIER: AxisOrderIdentifier,
        const N: usize,
    >
    GenericSpatialParDataMut<
        QLinear,
        ParDataHeaplessVec<QLinear, D, N>,
        OwningSpatialIndexingHelper<QLinear, QCoord, QDim, NUM_DIMS>,
        D,
        NUM_DIMS,
        AXIS_ORDER_IDENTIFIER,
    > for SpatialParDataHeaplessVec<QLinear, QCoord, QDim, D, NUM_DIMS, AXIS_ORDER_IDENTIFIER, N>
{
    fn get_data_view_mut<'a>(&'a mut self) -> &'a mut ParDataHeaplessVec<QLinear, D, N>
    where
        ParDataHeaplessVec<QLinear, D, N>: 'a,
    {
        &mut self.data
    }
}

//endregion

//endregion
