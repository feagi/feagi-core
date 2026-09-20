use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::generic_data::par_data::GenericParData;
use crate::spatial_indexing_structs::axis_order::AxisOrderIdentifier;
use crate::spatial_indexing_structs::coordinate::SpatialCoordinate;
use crate::spatial_indexing_structs::dimensions::SpatialDimensions;
use crate::spatial_indexing_structs::spatial_indexing_helpers::SpatialIndexingHelper;

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
    fn get_coord<'a>(
        &'a self,
        coordinate: SpatialCoordinate<SIH::CoordinateQuant, NUM_DIMS>,
    ) -> Option<&'a D>
    where
        GPD: 'a,
    {
        let linear = self.get_indexing_helper().coordinate_to_linear(&coordinate);
        self.get_data_view().get(linear)
    }








    //endregion

}