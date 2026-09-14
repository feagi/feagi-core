feagi_basis_quantization::create_wrapped_quantized_unsigned_integer!(pub TestAxis);
feagi_basis_quantization::create_wrapped_quantized_unsigned_integer!(pub TestLinearIndex);
feagi_basis_quantization::create_wrapped_quantized_unsigned_integer!(pub TestLinearCount);
feagi_basis_quantization::create_wrapped_quantized_signed_integer!(pub TestSignedAxis);

feagi_basis_quantization::create_wrapped_unsigned_integer_spatial_coordinate!(
    pub TestCoord3,
    3,
    (0, x, TestAxis),
    (1, y, TestAxis),
    (2, z, TestAxis),
);

feagi_basis_quantization::create_wrapped_unsigned_integer_spatial_data!(
    pub TestData3,
    3,
    (0, x, TestAxis),
    (1, y, TestAxis),
    (2, z, TestAxis),
);

feagi_basis_quantization::create_wrapped_unsigned_integer_spatial_dimensions!(
    pub TestDims3,
    TestCoord3,
    TestLinearIndex,
    TestLinearCount,
    3,
    (0, x, TestAxis),
    (1, y, TestAxis),
    (2, z, TestAxis),
);

feagi_basis_quantization::create_wrapped_signed_integer_spatial!(
    pub TestSignedSpatial3,
    3,
    (0, x, TestSignedAxis),
    (1, y, TestSignedAxis),
    (2, z, TestSignedAxis),
);

#[test]
fn unsigned_spatial_dimension_macro_maps_coordinates_to_linear_indices_and_back() {
    let dims = TestDims3::<u16>::try_new_from_usizes(4, 3, 2).unwrap();
    let coord = TestCoord3::<u16>::new_from_usize_array([2, 1, 1]).unwrap();

    let linear = dims.coordinate_to_linear_index(coord).unwrap();
    assert_eq!(linear.deref(), 18);

    let roundtrip = dims.linear_index_to_coordinate(linear).unwrap();
    assert_eq!(roundtrip.as_slice(), coord.as_slice());

    let all_coords: Vec<[u16; 3]> = dims
        .iter_coordinates()
        .map(|c| [c.get_x().deref(), c.get_y().deref(), c.get_z().deref()])
        .collect();
    assert_eq!(all_coords.len(), 24);
    assert_eq!(all_coords[0], [0, 0, 0]);
    assert_eq!(all_coords[1], [1, 0, 0]);
    assert_eq!(all_coords[2], [2, 0, 0]);
}

#[test]
fn unsigned_spatial_data_macro_handles_quantization_transforms_and_axis_accessors() {
    let mut data = TestData3::<u16>::new_from_array([5, 7, 300]);
    assert_eq!(data.get_x().deref(), 5);
    assert_eq!(data.get_y().deref(), 7);

    *data.get_y_mut() = TestAxis::new(9);
    assert_eq!(data.get_y().deref(), 9);

    assert!(data.try_to_quantization::<u8>().is_err());
    let clamped = data.to_quantization_clamped::<u8>();
    assert_eq!(clamped.get_z().deref(), 255);
}

#[test]
fn signed_spatial_macro_converts_between_quantizations_with_clamping() {
    let signed = TestSignedSpatial3::<i16>::new_from_isizes_unchecked(-200, 12, 300);
    let clamped = signed.to_quantization_clamped::<i8>();
    assert_eq!(clamped.get_x().deref(), -128);
    assert_eq!(clamped.get_y().deref(), 12);
    assert_eq!(clamped.get_z().deref(), 127);

    let unchecked = signed.to_quantization_unchecked::<i32>();
    assert_eq!(unchecked.get_x().deref(), -200);
    assert_eq!(unchecked.get_z().deref(), 300);
}
