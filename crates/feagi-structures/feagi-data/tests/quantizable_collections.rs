use feagi_data::quantizable_collections::dim_1d::{
    QuantizableLinearCollection1DHashmapSparse, QuantizableLinearCollection1DVectorDense,
};
use feagi_data::quantizable_collections::dim_2d::QuantizableSpatialCollection2DHashmapSparse;
use feagi_data::quantizable_collections::dim_2d::spatial_shared_traits::{
    QuantizableSpatialCollection2DBase, QuantizableSpatialCollection2DCPUData,
    QuantizableSpatialCollection2DIterWithIndex,
};
use feagi_data::quantizable_collections::dim_2d::QuantizableSpatialCollection2DVectorDense;
use feagi_data::quantizable_collections::dim_3d::QuantizableSpatialCollection3DHashmapSparse;
use feagi_data::quantizable_collections::dim_3d::spatial_shared_traits::{
    QuantizableSpatialCollection3DCPUData, QuantizableSpatialCollection3DIterWithIndex,
};
use feagi_data::quantizable_collections::dim_3d::QuantizableSpatialCollection3DVectorDense;
use feagi_data::quantizable_collections::dim_4d::QuantizableSpatialCollection4DHashmapSparse;
use feagi_data::quantizable_collections::dim_4d::spatial_shared_traits::{
    QuantizableSpatialCollection4DBase, QuantizableSpatialCollection4DCPUData,
};
use feagi_data::quantizable_collections::dim_4d::QuantizableSpatialCollection4DVectorDense;
use feagi_data::quantizable_collections::shared_traits::{
    QuantizableLinearCollectionAsSlice, QuantizableLinearCollectionBase, QuantizableLinearCollectionCPUData,
};
use feagi_data::quantizable_spatial::index::{
    SpatialIndexCoordinate2D, SpatialIndexCoordinate3D, SpatialIndexCoordinate4D, SpatialIndexDimensions2D,
    SpatialIndexDimensions3D, SpatialIndexDimensions4D,
};

#[test]
fn one_dimensional_dense_collection_supports_linear_access_and_slices() {
    let mut collection = QuantizableLinearCollection1DVectorDense::new_uniform(4_u16, 7_i32);

    assert_eq!(QuantizableLinearCollectionBase::max_linear_index(&collection), 4);
    assert_eq!(collection.get_values_slice(), &[7, 7, 7, 7]);
    assert_eq!(collection.try_get_value(2), Some(&7));

    *collection.get_unchecked_value_mut(2) = 11;

    assert_eq!(collection.get_unchecked_value(2), &11);
    assert_eq!(collection.get_values_slice(), &[7, 7, 11, 7]);
}

#[test]
fn one_dimensional_sparse_collection_supports_linear_lookup_and_mutation() {
    let mut collection = QuantizableLinearCollection1DHashmapSparse::new(8_u16);

    collection.internal_get_values_mut().insert(3, "active");

    assert_eq!(QuantizableLinearCollectionBase::max_linear_index(&collection), 8);
    assert_eq!(collection.try_get_value(2), None);
    assert_eq!(collection.try_get_value(3), Some(&"active"));

    *collection.get_unchecked_value_mut(3) = "updated";

    assert_eq!(collection.get_unchecked_value(3), &"updated");
}

#[test]
fn two_dimensional_dense_collection_maps_coordinates_to_linear_values() {
    let dimensions = SpatialIndexDimensions2D::new_checked(3_u16, 2_u16)
        .expect("non-zero dimensions should be valid");
    let collection = QuantizableSpatialCollection2DVectorDense::new_with_iter(dimensions, 0_i32..6);

    let coordinate = SpatialIndexCoordinate2D::new(1, 1);
    let reconstructed = collection.linear_index_to_coordinate(4);

    assert_eq!(QuantizableLinearCollectionBase::max_linear_index(&collection), 6);
    assert_eq!(collection.try_get_value_by_coordinate(coordinate), Some(&4));
    assert_eq!((*reconstructed.get_x(), *reconstructed.get_y()), (1, 1));
    assert_eq!(collection.iter_with_index().map(|(_, value)| *value).sum::<i32>(), 15);
}

#[test]
fn two_dimensional_sparse_collection_uses_linear_indices_for_coordinate_access() {
    let dimensions = SpatialIndexDimensions2D::new_checked(3_u16, 2_u16)
        .expect("non-zero dimensions should be valid");
    let mut collection = QuantizableSpatialCollection2DHashmapSparse::new(dimensions);

    collection.internal_get_values_mut().insert(4, "occupied");

    assert_eq!(
        collection.try_get_value_by_coordinate(SpatialIndexCoordinate2D::new(1, 1)),
        Some(&"occupied")
    );
    assert_eq!(collection.iter_with_index().collect::<Vec<_>>(), vec![(4, &"occupied")]);
}

#[test]
fn three_dimensional_dense_collection_maps_coordinates_to_linear_values() {
    let dimensions = SpatialIndexDimensions3D::new_checked(2_u16, 3_u16, 2_u16)
        .expect("non-zero dimensions should be valid");
    let mut collection = QuantizableSpatialCollection3DVectorDense::new_with_iter(dimensions, 0_i32..12);

    let coordinate = SpatialIndexCoordinate3D::new(1, 2, 1);

    assert_eq!(collection.try_get_value_by_coordinate(coordinate), Some(&11));

    *collection
        .try_get_value_by_coordinate_mut(SpatialIndexCoordinate3D::new(0, 1, 1))
        .expect("coordinate should map to an initialized dense value") = 99;

    assert_eq!(collection.get_unchecked_value(8), &99);
}

#[test]
fn three_dimensional_sparse_collection_supports_index_iteration() {
    let dimensions = SpatialIndexDimensions3D::new_checked(2_u16, 2_u16, 2_u16)
        .expect("non-zero dimensions should be valid");
    let mut collection = QuantizableSpatialCollection3DHashmapSparse::new(dimensions);

    collection.internal_get_values_mut().insert(0, 10);
    collection.internal_get_values_mut().insert(7, 70);

    let mut values: Vec<_> = collection.iter_with_index().map(|(index, value)| (index, *value)).collect();
    values.sort_by_key(|(index, _)| *index);

    assert_eq!(values, vec![(0, 10), (7, 70)]);
}

#[test]
fn four_dimensional_dense_collection_maps_coordinates_to_linear_values() {
    let dimensions = SpatialIndexDimensions4D::new_checked(2_u16, 2_u16, 2_u16, 2_u16)
        .expect("non-zero dimensions should be valid");
    let collection = QuantizableSpatialCollection4DVectorDense::new_with_iter(dimensions, 0_i32..16);

    let coordinate = SpatialIndexCoordinate4D::new(1, 1, 1, 1);
    let reconstructed = collection.linear_index_to_coordinate(15);

    assert_eq!(collection.try_get_value_by_coordinate(coordinate), Some(&15));
    assert_eq!(
        (
            *reconstructed.get_x(),
            *reconstructed.get_y(),
            *reconstructed.get_z(),
            *reconstructed.get_w(),
        ),
        (1, 1, 1, 1)
    );
}

#[test]
fn four_dimensional_sparse_collection_supports_coordinate_mutation() {
    let dimensions = SpatialIndexDimensions4D::new_checked(2_u16, 2_u16, 2_u16, 2_u16)
        .expect("non-zero dimensions should be valid");
    let mut collection = QuantizableSpatialCollection4DHashmapSparse::new(dimensions);

    collection.internal_get_values_mut().insert(15, 1_i32);

    *collection
        .try_get_value_by_coordinate_mut(SpatialIndexCoordinate4D::new(1, 1, 1, 1))
        .expect("coordinate should map to an initialized sparse value") = 5;

    assert_eq!(collection.try_get_value(15), Some(&5));
}
