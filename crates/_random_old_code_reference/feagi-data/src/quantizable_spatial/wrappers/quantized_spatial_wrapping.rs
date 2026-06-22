#[macro_export]
macro_rules! create_quantized_spatial_index_coordinate_2d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_2d_wrapper!(@impl [pub] $struct_name, $x_wrapper, $y_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_2d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_2d_wrapper!(@impl [] $struct_name, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_2d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_2d_wrapper!(@impl [$visibility] $struct_name, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_2d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $x_wrapper, $y_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_2d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<QE>,
            $x_wrapper,
            $y_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_2d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate2D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<$quant_element>,
            $x_wrapper,
            $y_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_index_coordinate_3d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_3d_wrapper!(@impl [pub] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_3d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_3d_wrapper!(@impl [] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_3d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_3d_wrapper!(@impl [$visibility] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_3d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_3d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<QE>,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_3d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate3D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<$quant_element>,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_index_coordinate_4d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_4d_wrapper!(@impl [pub] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_4d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_4d_wrapper!(@impl [] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_4d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_4d_wrapper!(@impl [$visibility] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_coordinate_4d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_4d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<QE>,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_4d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate4D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<$quant_element>,
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
        $crate::create_quantized_spatial_unsigned_coordinate_2d_wrapper!(@impl [pub] $struct_name, $x_wrapper, $y_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_2d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_2d_wrapper!(@impl [] $struct_name, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_2d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_2d_wrapper!(@impl [$visibility] $struct_name, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_2d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $x_wrapper, $y_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_2d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<QE>,
            $x_wrapper,
            $y_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_2d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate2D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<$quant_element>,
            $x_wrapper,
            $y_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_unsigned_coordinate_3d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_3d_wrapper!(@impl [pub] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_3d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_3d_wrapper!(@impl [] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_3d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_3d_wrapper!(@impl [$visibility] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_3d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_3d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<QE>,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_3d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate3D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<$quant_element>,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_unsigned_coordinate_4d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_4d_wrapper!(@impl [pub] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_4d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_4d_wrapper!(@impl [] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_4d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_4d_wrapper!(@impl [$visibility] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_coordinate_4d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_4d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<QE>,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_4d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate4D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<$quant_element>,
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
        $crate::create_quantized_spatial_signed_coordinate_2d_wrapper!(@impl [pub] $struct_name, $x_wrapper, $y_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_2d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_2d_wrapper!(@impl [] $struct_name, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_2d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_2d_wrapper!(@impl [$visibility] $struct_name, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_2d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $x_wrapper, $y_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_2d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<QE>,
            $x_wrapper,
            $y_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_2d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate2D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<$quant_element>,
            $x_wrapper,
            $y_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_signed_coordinate_3d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_3d_wrapper!(@impl [pub] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_3d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_3d_wrapper!(@impl [] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_3d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_3d_wrapper!(@impl [$visibility] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_3d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_3d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<QE>,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_3d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate3D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<$quant_element>,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_signed_coordinate_4d_wrapper {
    ($struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_4d_wrapper!(@impl [pub] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_4d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_4d_wrapper!(@impl [] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_4d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_4d_wrapper!(@impl [$visibility] $struct_name, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_coordinate_4d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_4d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<QE>,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_coordinate_4d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate4D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<$quant_element>,
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
        $crate::create_quantized_spatial_index_dimensions_2d_wrapper!(@impl [pub] $struct_name, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_2d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_2d_wrapper!(@impl [] $struct_name, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_2d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_2d_wrapper!(@impl [$visibility] $struct_name, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_2d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_2d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexDimensions2D<QE>,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<QE>,
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
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_2d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::index::SpatialIndexDimensions2D<$quant_element>,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate2D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<$quant_element>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper
        );
        $crate::__impl_quantized_spatial_index_dimensions_2d_methods!(
            concrete $struct_name,
            $quant_element,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate2D<$quant_element>,
            $coordinate_wrapper,
            $linear_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_index_dimensions_3d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_3d_wrapper!(@impl [pub] $struct_name, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_3d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_3d_wrapper!(@impl [] $struct_name, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_3d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_3d_wrapper!(@impl [$visibility] $struct_name, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_3d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_3d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexDimensions3D<QE>,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<QE>,
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
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_3d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::index::SpatialIndexDimensions3D<$quant_element>,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate3D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<$quant_element>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
        $crate::__impl_quantized_spatial_index_dimensions_3d_methods!(
            concrete $struct_name,
            $quant_element,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate3D<$quant_element>,
            $coordinate_wrapper,
            $linear_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_index_dimensions_4d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_4d_wrapper!(@impl [pub] $struct_name, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_4d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_4d_wrapper!(@impl [] $struct_name, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_4d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_4d_wrapper!(@impl [$visibility] $struct_name, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_index_dimensions_4d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $coordinate_wrapper, $linear_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_4d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::index::SpatialIndexDimensions4D<QE>,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<QE>,
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
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $linear_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_4d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::index::SpatialIndexDimensions4D<$quant_element>,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate4D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<$quant_element>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
        $crate::__impl_quantized_spatial_index_dimensions_4d_methods!(
            concrete $struct_name,
            $quant_element,
            $crate::quantizable_spatial::index::SpatialIndexCoordinate4D<$quant_element>,
            $coordinate_wrapper,
            $linear_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_unsigned_dimensions_2d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_2d_wrapper!(@impl [pub] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_2d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_2d_wrapper!(@impl [] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_2d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_2d_wrapper!(@impl [$visibility] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_2d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_2d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintDimensions2D<QE>,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<QE>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_2d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintDimensions2D<$quant_element>,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate2D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<$quant_element>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_unsigned_dimensions_3d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_3d_wrapper!(@impl [pub] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_3d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_3d_wrapper!(@impl [] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_3d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_3d_wrapper!(@impl [$visibility] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_3d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_3d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintDimensions3D<QE>,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<QE>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_3d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintDimensions3D<$quant_element>,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate3D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<$quant_element>,
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
        $crate::create_quantized_spatial_unsigned_dimensions_4d_wrapper!(@impl [pub] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_4d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_4d_wrapper!(@impl [] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_4d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_4d_wrapper!(@impl [$visibility] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_unsigned_dimensions_4d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_4d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::unsigned_integer::SpatialDimensions4D<QE>,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<QE>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_4d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::unsigned_integer::SpatialDimensions4D<$quant_element>,
            $crate::quantizable_spatial::unsigned_integer::SpatialUintCoordinate4D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedUnsignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperUnsignedInteger<$quant_element>,
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
        $crate::create_quantized_spatial_signed_dimensions_2d_wrapper!(@impl [pub] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_2d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_2d_wrapper!(@impl [] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_2d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_2d_wrapper!(@impl [$visibility] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_2d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_2d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialSintDimensions2D<QE>,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate2D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<QE>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_2d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::signed_integer::SpatialSintDimensions2D<$quant_element>,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate2D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<$quant_element>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper
        );
    };
}

#[macro_export]
macro_rules! create_quantized_spatial_signed_dimensions_3d_wrapper {
    ($struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_3d_wrapper!(@impl [pub] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_3d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_3d_wrapper!(@impl [] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_3d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_3d_wrapper!(@impl [$visibility] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_3d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_3d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialSintDimensions3D<QE>,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate3D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<QE>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_3d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::signed_integer::SpatialSintDimensions3D<$quant_element>,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate3D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<$quant_element>,
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
        $crate::create_quantized_spatial_signed_dimensions_4d_wrapper!(@impl [pub] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_4d_wrapper!(@impl_concrete [pub] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_4d_wrapper!(@impl [] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (private $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_4d_wrapper!(@impl_concrete [] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_4d_wrapper!(@impl [$visibility] $struct_name, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    ($visibility:vis $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::create_quantized_spatial_signed_dimensions_4d_wrapper!(@impl_concrete [$visibility] $struct_name, $quant_element, $coordinate_wrapper, $x_wrapper, $y_wrapper, $z_wrapper, $w_wrapper);
    };
    (@impl [$($visibility:tt)*] $struct_name:ident, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_4d_wrapper!(
            [$($visibility)*],
            $struct_name,
            $crate::quantizable_spatial::signed_integer::SpatialDimensions4D<QE>,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate4D<QE>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<QE>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
    (@impl_concrete [$($visibility:tt)*] $struct_name:ident, $quant_element:ty, $coordinate_wrapper:ident, $x_wrapper:ident, $y_wrapper:ident, $z_wrapper:ident, $w_wrapper:ident) => {
        $crate::__create_quantized_spatial_dimensions_4d_wrapper!(
            concrete [$($visibility)*],
            $struct_name,
            $quant_element,
            $crate::quantizable_spatial::signed_integer::SpatialDimensions4D<$quant_element>,
            $crate::quantizable_spatial::signed_integer::SpatialSintCoordinate4D<$quant_element>,
            $crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait,
            $crate::quantizable_linear::wrappers::QuantizedElementWrapperSignedInteger<$quant_element>,
            $coordinate_wrapper,
            $x_wrapper,
            $y_wrapper,
            $z_wrapper,
            $w_wrapper
        );
    };
}
