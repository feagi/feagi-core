use feagi_data::{
    create_quantized_index_count_wrapper, create_quantized_signed_integer_wrapper,
    create_quantized_spatial_index_coordinate_2d_wrapper,
    create_quantized_spatial_index_coordinate_3d_wrapper,
    create_quantized_spatial_index_coordinate_4d_wrapper,
    create_quantized_spatial_index_dimensions_2d_wrapper,
    create_quantized_spatial_index_dimensions_3d_wrapper,
    create_quantized_spatial_index_dimensions_4d_wrapper,
    create_quantized_spatial_signed_coordinate_2d_wrapper,
    create_quantized_spatial_signed_coordinate_3d_wrapper,
    create_quantized_spatial_signed_coordinate_4d_wrapper,
    create_quantized_spatial_signed_dimensions_2d_wrapper,
    create_quantized_spatial_signed_dimensions_3d_wrapper,
    create_quantized_spatial_signed_dimensions_4d_wrapper,
    create_quantized_spatial_unsigned_coordinate_2d_wrapper,
    create_quantized_spatial_unsigned_coordinate_3d_wrapper,
    create_quantized_spatial_unsigned_coordinate_4d_wrapper,
    create_quantized_spatial_unsigned_dimensions_2d_wrapper,
    create_quantized_spatial_unsigned_dimensions_3d_wrapper,
    create_quantized_spatial_unsigned_dimensions_4d_wrapper,
    create_quantized_unsigned_integer_wrapper,
};
use feagi_data::quantizable_linear::base_types::QuantizedIndexCountTrait;

create_quantized_index_count_wrapper!(IndexAxis);
create_quantized_unsigned_integer_wrapper!(UnsignedAxis);
create_quantized_signed_integer_wrapper!(SignedAxis);

create_quantized_spatial_index_coordinate_2d_wrapper!(IndexCoord2D, IndexAxis, IndexAxis);
create_quantized_spatial_index_coordinate_3d_wrapper!(pub(crate) IndexCoord3D, IndexAxis, IndexAxis, IndexAxis);
create_quantized_spatial_index_coordinate_4d_wrapper!(private PrivateIndexCoord4D, IndexAxis, IndexAxis, IndexAxis, IndexAxis);
create_quantized_spatial_index_dimensions_2d_wrapper!(IndexDims2D, IndexCoord2D, IndexAxis, IndexAxis, IndexAxis);
create_quantized_spatial_index_dimensions_3d_wrapper!(pub(crate) IndexDims3D, IndexCoord3D, IndexAxis, IndexAxis, IndexAxis, IndexAxis);
create_quantized_spatial_index_dimensions_4d_wrapper!(
    private PrivateIndexDims4D,
    PrivateIndexCoord4D,
    IndexAxis,
    IndexAxis,
    IndexAxis,
    IndexAxis,
    IndexAxis
);

create_quantized_spatial_unsigned_coordinate_2d_wrapper!(UnsignedCoord2D, UnsignedAxis, UnsignedAxis);
create_quantized_spatial_unsigned_coordinate_3d_wrapper!(pub(crate) UnsignedCoord3D, UnsignedAxis, UnsignedAxis, UnsignedAxis);
create_quantized_spatial_unsigned_coordinate_4d_wrapper!(
    private PrivateUnsignedCoord4D,
    UnsignedAxis,
    UnsignedAxis,
    UnsignedAxis,
    UnsignedAxis
);
create_quantized_spatial_unsigned_dimensions_2d_wrapper!(UnsignedDims2D, UnsignedCoord2D, UnsignedAxis, UnsignedAxis);
create_quantized_spatial_unsigned_dimensions_3d_wrapper!(
    pub(crate) UnsignedDims3D,
    UnsignedCoord3D,
    UnsignedAxis,
    UnsignedAxis,
    UnsignedAxis
);
create_quantized_spatial_unsigned_dimensions_4d_wrapper!(
    private PrivateUnsignedDims4D,
    PrivateUnsignedCoord4D,
    UnsignedAxis,
    UnsignedAxis,
    UnsignedAxis,
    UnsignedAxis
);

create_quantized_spatial_signed_coordinate_2d_wrapper!(SignedCoord2D, SignedAxis, SignedAxis);
create_quantized_spatial_signed_coordinate_3d_wrapper!(pub(crate) SignedCoord3D, SignedAxis, SignedAxis, SignedAxis);
create_quantized_spatial_signed_coordinate_4d_wrapper!(
    private PrivateSignedCoord4D,
    SignedAxis,
    SignedAxis,
    SignedAxis,
    SignedAxis
);
create_quantized_spatial_signed_dimensions_2d_wrapper!(SignedDims2D, SignedCoord2D, SignedAxis, SignedAxis);
create_quantized_spatial_signed_dimensions_3d_wrapper!(
    pub(crate) SignedDims3D,
    SignedCoord3D,
    SignedAxis,
    SignedAxis,
    SignedAxis
);
create_quantized_spatial_signed_dimensions_4d_wrapper!(
    private PrivateSignedDims4D,
    PrivateSignedCoord4D,
    SignedAxis,
    SignedAxis,
    SignedAxis,
    SignedAxis
);

fn index_axis(value: u32) -> IndexAxis<u16> {
    IndexAxis::from_u32(value)
}

fn unsigned_axis(value: u16) -> UnsignedAxis<u16> {
    <UnsignedAxis<u16> as feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase<u16>>::wrap(value)
}

fn signed_axis(value: i16) -> SignedAxis<i16> {
    <SignedAxis<i16> as feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase<i16>>::wrap(value)
}

#[test]
fn generated_index_spatial_wrappers_construct_and_convert_indices() {
    let coord = IndexCoord3D::<u16>::new(index_axis(1), index_axis(2), index_axis(3));
    let dims = IndexDims3D::<u16>::new_checked(index_axis(4), index_axis(5), index_axis(6))
        .expect("non-zero dimensions should be valid");

    assert_eq!(coord.get_x().to_u32(), 1);
    assert_eq!(coord.get_y().to_u32(), 2);
    assert_eq!(coord.get_z().to_u32(), 3);
    assert!(dims.does_coordinate_fit(coord));

    let linear = dims.coordinate_to_linear_index(IndexCoord3D::new(index_axis(1), index_axis(2), index_axis(3)));
    let reconstructed = dims.linear_index_to_coordinate(linear);

    assert_eq!(linear.to_u32(), 69);
    assert_eq!(reconstructed.get_x().to_u32(), 1);
    assert_eq!(reconstructed.get_y().to_u32(), 2);
    assert_eq!(reconstructed.get_z().to_u32(), 3);
}

#[test]
fn generated_spatial_wrappers_cover_all_dimensions_and_visibility_forms() {
    let _index_2d = IndexCoord2D::<u16>::new(index_axis(1), index_axis(2));
    let _index_4d = PrivateIndexCoord4D::<u16>::new(index_axis(1), index_axis(2), index_axis(3), index_axis(4));
    let _index_dims_2d = IndexDims2D::<u16>::new_checked(index_axis(2), index_axis(3))
        .expect("non-zero index dimensions should be valid");
    let _index_dims_4d = PrivateIndexDims4D::<u16>::new_checked(
        index_axis(2),
        index_axis(3),
        index_axis(4),
        index_axis(5),
    )
    .expect("non-zero index dimensions should be valid");

    let unsigned_coord = UnsignedCoord3D::<u16>::new(unsigned_axis(1), unsigned_axis(2), unsigned_axis(3));
    let unsigned_dims = UnsignedDims3D::<u16>::new_checked(unsigned_axis(4), unsigned_axis(5), unsigned_axis(6))
        .expect("non-zero unsigned dimensions should be valid");
    assert!(unsigned_dims.does_coordinate_fit(unsigned_coord));

    let _unsigned_2d = UnsignedCoord2D::<u16>::new(unsigned_axis(1), unsigned_axis(2));
    let _unsigned_4d = PrivateUnsignedCoord4D::<u16>::new(
        unsigned_axis(1),
        unsigned_axis(2),
        unsigned_axis(3),
        unsigned_axis(4),
    );
    let _unsigned_dims_2d = UnsignedDims2D::<u16>::new_checked(unsigned_axis(2), unsigned_axis(3))
        .expect("non-zero unsigned dimensions should be valid");
    let _unsigned_dims_4d = PrivateUnsignedDims4D::<u16>::new_checked(
        unsigned_axis(2),
        unsigned_axis(3),
        unsigned_axis(4),
        unsigned_axis(5),
    )
    .expect("non-zero unsigned dimensions should be valid");

    let signed_coord = SignedCoord3D::<i16>::new(signed_axis(1), signed_axis(2), signed_axis(3));
    let signed_dims = SignedDims3D::<i16>::new_checked(signed_axis(4), signed_axis(5), signed_axis(6))
        .expect("positive signed dimensions should be valid");
    assert!(signed_dims.does_coordinate_fit(signed_coord));

    let _signed_2d = SignedCoord2D::<i16>::new(signed_axis(1), signed_axis(2));
    let _signed_4d = PrivateSignedCoord4D::<i16>::new(
        signed_axis(1),
        signed_axis(2),
        signed_axis(3),
        signed_axis(4),
    );
    let _signed_dims_2d = SignedDims2D::<i16>::new_checked(signed_axis(2), signed_axis(3))
        .expect("positive signed dimensions should be valid");
    let _signed_dims_4d = PrivateSignedDims4D::<i16>::new_checked(
        signed_axis(2),
        signed_axis(3),
        signed_axis(4),
        signed_axis(5),
    )
    .expect("positive signed dimensions should be valid");
}
