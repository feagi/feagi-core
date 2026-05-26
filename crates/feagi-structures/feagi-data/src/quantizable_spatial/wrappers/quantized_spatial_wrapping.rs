#[macro_export]
macro_rules! create_quantized_spatial_index_coordinate_2d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_2d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount,
            $x_wrapper,
            $y_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_index_coordinate_3d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_3d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_index_coordinate_4d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_4d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_unsigned_coordinate_2d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_2d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger,
            $x_wrapper,
            $y_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_unsigned_coordinate_3d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_3d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_unsigned_coordinate_4d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_4d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_signed_coordinate_2d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_2d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger,
            $x_wrapper,
            $y_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_signed_coordinate_3d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_3d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_signed_coordinate_4d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_4d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_index_dimensions_2d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_2d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexDimensions2D<QE>,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper
        );
        $crate::__impl_quantized_spatial_index_dimensions_2d_methods!(
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate2D<QE>,
            $coordinate_wrapper,
            $linear_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_index_dimensions_3d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_3d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexDimensions3D<QE>,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
        $crate::__impl_quantized_spatial_index_dimensions_3d_methods!(
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate3D<QE>,
            $coordinate_wrapper,
            $linear_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_index_dimensions_4d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_4d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexDimensions4D<QE>,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
        $crate::__impl_quantized_spatial_index_dimensions_4d_methods!(
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate4D<QE>,
            $coordinate_wrapper,
            $linear_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_unsigned_dimensions_2d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_2d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintDimensions2D<QE>,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_unsigned_dimensions_3d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_3d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintDimensions3D<QE>,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_unsigned_dimensions_4d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_4d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialDimensions4D<QE>,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_signed_dimensions_2d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_2d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialSintDimensions2D<QE>,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_signed_dimensions_3d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_3d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialSintDimensions3D<QE>,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_signed_dimensions_4d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_4d_wrapper!(
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialDimensions4D<QE>,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
}
