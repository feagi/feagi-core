#[doc(hidden)]
#[macro_export]
macro_rules! __impl_quantized_spatial_wrapper_base {
    ($wrapper_type:ident, $quant_element:ident, $inner_type:ty, $($quant_bound:tt)+) => {
        impl<$quant_element> $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$inner_type>
            for $wrapper_type<$quant_element>
        where
            $quant_element: $($quant_bound)+,
        {
            #[inline(always)]
            fn wrap(spatial: $inner_type) -> Self {
                Self(spatial)
            }

            #[inline(always)]
            fn wrap_ref(spatial: &$inner_type) -> &Self {
                // The generated wrapper is #[repr(transparent)] over exactly one spatial value.
                unsafe { &*(spatial as *const $inner_type as *const Self) }
            }

            #[inline(always)]
            fn unwrap(self) -> $inner_type {
                self.0
            }

            #[inline(always)]
            fn spatial_ref(&self) -> &$inner_type {
                &self.0
            }

            #[inline(always)]
            fn spatial_ref_mut(&mut self) -> &mut $inner_type {
                &mut self.0
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_quantized_spatial_wrapper_base_concrete {
    ($wrapper_type:ident, $quant_element:ty, $inner_type:ty, $($quant_bound:tt)+) => {
        impl $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$inner_type>
            for $wrapper_type
        where
            $quant_element: $($quant_bound)+,
        {
            #[inline(always)]
            fn wrap(spatial: $inner_type) -> Self {
                Self(spatial)
            }

            #[inline(always)]
            fn wrap_ref(spatial: &$inner_type) -> &Self {
                // The generated wrapper is #[repr(transparent)] over exactly one spatial value.
                unsafe { &*(spatial as *const $inner_type as *const Self) }
            }

            #[inline(always)]
            fn unwrap(self) -> $inner_type {
                self.0
            }

            #[inline(always)]
            fn spatial_ref(&self) -> &$inner_type {
                &self.0
            }

            #[inline(always)]
            fn spatial_ref_mut(&mut self) -> &mut $inner_type {
                &mut self.0
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __create_quantized_spatial_coordinate_2d_wrapper {
    (
        [$($visibility:tt)*],
        $struct_name:ident,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $x_wrapper:ident,
        $y_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name<QE: $quant_bound>($base_coordinate);

        impl<QE> $struct_name<QE>
        where
            QE: $quant_bound,
            $x_wrapper<QE>: $axis_wrapper_bound,
            $y_wrapper<QE>: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new(x: $x_wrapper<QE>, y: $y_wrapper<QE>) -> Self {
                Self(<$base_coordinate>::new(
                    <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(x),
                    <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(y),
                ))
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper<QE> {
                <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper<QE> {
                <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_coordinate {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base!(
            $struct_name,
            QE,
            $base_coordinate,
            $quant_bound
        );
    };
    (
        concrete [$($visibility:tt)*],
        $struct_name:ident,
        $quant_element:ty,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $x_wrapper:ident,
        $y_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name($base_coordinate);

        impl $struct_name
        where
            $quant_element: $quant_bound,
            $x_wrapper: $axis_wrapper_bound,
            $y_wrapper: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new(x: $x_wrapper, y: $y_wrapper) -> Self {
                Self(<$base_coordinate>::new(
                    <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(x),
                    <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(y),
                ))
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper {
                <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper {
                <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_coordinate {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base_concrete!(
            $struct_name,
            $quant_element,
            $base_coordinate,
            $quant_bound
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __create_quantized_spatial_coordinate_3d_wrapper {
    (
        [$($visibility:tt)*],
        $struct_name:ident,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $x_wrapper:ident,
        $y_wrapper:ident,
        $z_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name<QE: $quant_bound>($base_coordinate);

        impl<QE> $struct_name<QE>
        where
            QE: $quant_bound,
            $x_wrapper<QE>: $axis_wrapper_bound,
            $y_wrapper<QE>: $axis_wrapper_bound,
            $z_wrapper<QE>: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new(x: $x_wrapper<QE>, y: $y_wrapper<QE>, z: $z_wrapper<QE>) -> Self {
                Self(<$base_coordinate>::new(
                    <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(x),
                    <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(y),
                    <$z_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(z),
                ))
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper<QE> {
                <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper<QE> {
                <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub fn get_z(&self) -> $z_wrapper<QE> {
                <$z_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_z())
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_coordinate {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base!(
            $struct_name,
            QE,
            $base_coordinate,
            $quant_bound
        );
    };
    (
        concrete [$($visibility:tt)*],
        $struct_name:ident,
        $quant_element:ty,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $x_wrapper:ident,
        $y_wrapper:ident,
        $z_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name($base_coordinate);

        impl $struct_name
        where
            $quant_element: $quant_bound,
            $x_wrapper: $axis_wrapper_bound,
            $y_wrapper: $axis_wrapper_bound,
            $z_wrapper: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new(x: $x_wrapper, y: $y_wrapper, z: $z_wrapper) -> Self {
                Self(<$base_coordinate>::new(
                    <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(x),
                    <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(y),
                    <$z_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(z),
                ))
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper {
                <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper {
                <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub fn get_z(&self) -> $z_wrapper {
                <$z_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_z())
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_coordinate {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base_concrete!(
            $struct_name,
            $quant_element,
            $base_coordinate,
            $quant_bound
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __create_quantized_spatial_coordinate_4d_wrapper {
    (
        [$($visibility:tt)*],
        $struct_name:ident,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $x_wrapper:ident,
        $y_wrapper:ident,
        $z_wrapper:ident,
        $w_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name<QE: $quant_bound>($base_coordinate);

        impl<QE> $struct_name<QE>
        where
            QE: $quant_bound,
            $x_wrapper<QE>: $axis_wrapper_bound,
            $y_wrapper<QE>: $axis_wrapper_bound,
            $z_wrapper<QE>: $axis_wrapper_bound,
            $w_wrapper<QE>: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new(x: $x_wrapper<QE>, y: $y_wrapper<QE>, z: $z_wrapper<QE>, w: $w_wrapper<QE>) -> Self {
                Self(<$base_coordinate>::new(
                    <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(x),
                    <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(y),
                    <$z_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(z),
                    <$w_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(w),
                ))
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper<QE> {
                <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper<QE> {
                <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub fn get_z(&self) -> $z_wrapper<QE> {
                <$z_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_z())
            }

            #[inline(always)]
            pub fn get_w(&self) -> $w_wrapper<QE> {
                <$w_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_w())
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_coordinate {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base!(
            $struct_name,
            QE,
            $base_coordinate,
            $quant_bound
        );
    };
    (
        concrete [$($visibility:tt)*],
        $struct_name:ident,
        $quant_element:ty,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $x_wrapper:ident,
        $y_wrapper:ident,
        $z_wrapper:ident,
        $w_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name($base_coordinate);

        impl $struct_name
        where
            $quant_element: $quant_bound,
            $x_wrapper: $axis_wrapper_bound,
            $y_wrapper: $axis_wrapper_bound,
            $z_wrapper: $axis_wrapper_bound,
            $w_wrapper: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new(x: $x_wrapper, y: $y_wrapper, z: $z_wrapper, w: $w_wrapper) -> Self {
                Self(<$base_coordinate>::new(
                    <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(x),
                    <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(y),
                    <$z_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(z),
                    <$w_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(w),
                ))
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper {
                <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper {
                <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub fn get_z(&self) -> $z_wrapper {
                <$z_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_z())
            }

            #[inline(always)]
            pub fn get_w(&self) -> $w_wrapper {
                <$w_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_w())
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_coordinate {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base_concrete!(
            $struct_name,
            $quant_element,
            $base_coordinate,
            $quant_bound
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __create_quantized_spatial_dimensions_2d_wrapper {
    (
        [$($visibility:tt)*],
        $struct_name:ident,
        $base_dimensions:ty,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $coordinate_wrapper:ident,
        $x_wrapper:ident,
        $y_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name<QE: $quant_bound>($base_dimensions);

        impl<QE> $struct_name<QE>
        where
            QE: $quant_bound,
            $coordinate_wrapper<QE>: $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>,
            $x_wrapper<QE>: $axis_wrapper_bound,
            $y_wrapper<QE>: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new_unchecked(x: $x_wrapper<QE>, y: $y_wrapper<QE>) -> Self {
                Self(<$base_dimensions>::new_unchecked(
                    <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(x),
                    <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(y),
                ))
            }

            #[inline(always)]
            pub fn new_checked(x: $x_wrapper<QE>, y: $y_wrapper<QE>) -> Result<Self, $crate::quantizable_spatial::FeagiSpatialError> {
                <$base_dimensions>::new_checked(
                    <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(x),
                    <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(y),
                ).map(Self)
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper<QE> {
                <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper<QE> {
                <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub fn does_coordinate_fit(&self, coordinate: $coordinate_wrapper<QE>) -> bool {
                self.0.does_coordinate_fit(
                    <$coordinate_wrapper<QE> as $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>>::unwrap(coordinate)
                )
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_dimensions {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base!(
            $struct_name,
            QE,
            $base_dimensions,
            $quant_bound
        );
    };
    (
        concrete [$($visibility:tt)*],
        $struct_name:ident,
        $quant_element:ty,
        $base_dimensions:ty,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $coordinate_wrapper:ident,
        $x_wrapper:ident,
        $y_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name($base_dimensions);

        impl $struct_name
        where
            $quant_element: $quant_bound,
            $coordinate_wrapper: $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>,
            $x_wrapper: $axis_wrapper_bound,
            $y_wrapper: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new_unchecked(x: $x_wrapper, y: $y_wrapper) -> Self {
                Self(<$base_dimensions>::new_unchecked(
                    <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(x),
                    <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(y),
                ))
            }

            #[inline(always)]
            pub fn new_checked(x: $x_wrapper, y: $y_wrapper) -> Result<Self, $crate::quantizable_spatial::FeagiSpatialError> {
                <$base_dimensions>::new_checked(
                    <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(x),
                    <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(y),
                ).map(Self)
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper {
                <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper {
                <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub fn does_coordinate_fit(&self, coordinate: $coordinate_wrapper) -> bool {
                self.0.does_coordinate_fit(
                    <$coordinate_wrapper as $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>>::unwrap(coordinate)
                )
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_dimensions {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base_concrete!(
            $struct_name,
            $quant_element,
            $base_dimensions,
            $quant_bound
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __create_quantized_spatial_dimensions_3d_wrapper {
    (
        [$($visibility:tt)*],
        $struct_name:ident,
        $base_dimensions:ty,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $coordinate_wrapper:ident,
        $x_wrapper:ident,
        $y_wrapper:ident,
        $z_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name<QE: $quant_bound>($base_dimensions);

        impl<QE> $struct_name<QE>
        where
            QE: $quant_bound,
            $coordinate_wrapper<QE>: $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>,
            $x_wrapper<QE>: $axis_wrapper_bound,
            $y_wrapper<QE>: $axis_wrapper_bound,
            $z_wrapper<QE>: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new_unchecked(x: $x_wrapper<QE>, y: $y_wrapper<QE>, z: $z_wrapper<QE>) -> Self {
                Self(<$base_dimensions>::new_unchecked(
                    <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(x),
                    <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(y),
                    <$z_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(z),
                ))
            }

            #[inline(always)]
            pub fn new_checked(x: $x_wrapper<QE>, y: $y_wrapper<QE>, z: $z_wrapper<QE>) -> Result<Self, $crate::quantizable_spatial::FeagiSpatialError> {
                <$base_dimensions>::new_checked(
                    <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(x),
                    <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(y),
                    <$z_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(z),
                ).map(Self)
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper<QE> {
                <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper<QE> {
                <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub fn get_z(&self) -> $z_wrapper<QE> {
                <$z_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_z())
            }

            #[inline(always)]
            pub fn does_coordinate_fit(&self, coordinate: $coordinate_wrapper<QE>) -> bool {
                self.0.does_coordinate_fit(
                    <$coordinate_wrapper<QE> as $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>>::unwrap(coordinate)
                )
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_dimensions {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base!(
            $struct_name,
            QE,
            $base_dimensions,
            $quant_bound
        );
    };
    (
        concrete [$($visibility:tt)*],
        $struct_name:ident,
        $quant_element:ty,
        $base_dimensions:ty,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $coordinate_wrapper:ident,
        $x_wrapper:ident,
        $y_wrapper:ident,
        $z_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name($base_dimensions);

        impl $struct_name
        where
            $quant_element: $quant_bound,
            $coordinate_wrapper: $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>,
            $x_wrapper: $axis_wrapper_bound,
            $y_wrapper: $axis_wrapper_bound,
            $z_wrapper: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new_unchecked(x: $x_wrapper, y: $y_wrapper, z: $z_wrapper) -> Self {
                Self(<$base_dimensions>::new_unchecked(
                    <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(x),
                    <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(y),
                    <$z_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(z),
                ))
            }

            #[inline(always)]
            pub fn new_checked(x: $x_wrapper, y: $y_wrapper, z: $z_wrapper) -> Result<Self, $crate::quantizable_spatial::FeagiSpatialError> {
                <$base_dimensions>::new_checked(
                    <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(x),
                    <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(y),
                    <$z_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(z),
                ).map(Self)
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper {
                <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper {
                <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub fn get_z(&self) -> $z_wrapper {
                <$z_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_z())
            }

            #[inline(always)]
            pub fn does_coordinate_fit(&self, coordinate: $coordinate_wrapper) -> bool {
                self.0.does_coordinate_fit(
                    <$coordinate_wrapper as $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>>::unwrap(coordinate)
                )
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_dimensions {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base_concrete!(
            $struct_name,
            $quant_element,
            $base_dimensions,
            $quant_bound
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __create_quantized_spatial_dimensions_4d_wrapper {
    (
        [$($visibility:tt)*],
        $struct_name:ident,
        $base_dimensions:ty,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $coordinate_wrapper:ident,
        $x_wrapper:ident,
        $y_wrapper:ident,
        $z_wrapper:ident,
        $w_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name<QE: $quant_bound>($base_dimensions);

        impl<QE> $struct_name<QE>
        where
            QE: $quant_bound,
            $coordinate_wrapper<QE>: $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>,
            $x_wrapper<QE>: $axis_wrapper_bound,
            $y_wrapper<QE>: $axis_wrapper_bound,
            $z_wrapper<QE>: $axis_wrapper_bound,
            $w_wrapper<QE>: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new_unchecked(x: $x_wrapper<QE>, y: $y_wrapper<QE>, z: $z_wrapper<QE>, w: $w_wrapper<QE>) -> Self {
                Self(<$base_dimensions>::new_unchecked(
                    <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(x),
                    <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(y),
                    <$z_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(z),
                    <$w_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(w),
                ))
            }

            #[inline(always)]
            pub fn new_checked(x: $x_wrapper<QE>, y: $y_wrapper<QE>, z: $z_wrapper<QE>, w: $w_wrapper<QE>) -> Result<Self, $crate::quantizable_spatial::FeagiSpatialError> {
                <$base_dimensions>::new_checked(
                    <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(x),
                    <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(y),
                    <$z_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(z),
                    <$w_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(w),
                ).map(Self)
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper<QE> {
                <$x_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper<QE> {
                <$y_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub fn get_z(&self) -> $z_wrapper<QE> {
                <$z_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_z())
            }

            #[inline(always)]
            pub fn get_w(&self) -> $w_wrapper<QE> {
                <$w_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(*self.0.get_w())
            }

            #[inline(always)]
            pub fn does_coordinate_fit(&self, coordinate: $coordinate_wrapper<QE>) -> bool {
                self.0.does_coordinate_fit(
                    <$coordinate_wrapper<QE> as $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>>::unwrap(coordinate)
                )
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_dimensions {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base!(
            $struct_name,
            QE,
            $base_dimensions,
            $quant_bound
        );
    };
    (
        concrete [$($visibility:tt)*],
        $struct_name:ident,
        $quant_element:ty,
        $base_dimensions:ty,
        $base_coordinate:ty,
        $quant_bound:path,
        $axis_wrapper_bound:path,
        $coordinate_wrapper:ident,
        $x_wrapper:ident,
        $y_wrapper:ident,
        $z_wrapper:ident,
        $w_wrapper:ident
    ) => {
        #[repr(transparent)]
        $($visibility)* struct $struct_name($base_dimensions);

        impl $struct_name
        where
            $quant_element: $quant_bound,
            $coordinate_wrapper: $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>,
            $x_wrapper: $axis_wrapper_bound,
            $y_wrapper: $axis_wrapper_bound,
            $z_wrapper: $axis_wrapper_bound,
            $w_wrapper: $axis_wrapper_bound,
        {
            #[inline(always)]
            pub fn new_unchecked(x: $x_wrapper, y: $y_wrapper, z: $z_wrapper, w: $w_wrapper) -> Self {
                Self(<$base_dimensions>::new_unchecked(
                    <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(x),
                    <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(y),
                    <$z_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(z),
                    <$w_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(w),
                ))
            }

            #[inline(always)]
            pub fn new_checked(x: $x_wrapper, y: $y_wrapper, z: $z_wrapper, w: $w_wrapper) -> Result<Self, $crate::quantizable_spatial::FeagiSpatialError> {
                <$base_dimensions>::new_checked(
                    <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(x),
                    <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(y),
                    <$z_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(z),
                    <$w_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(w),
                ).map(Self)
            }

            #[inline(always)]
            pub fn get_x(&self) -> $x_wrapper {
                <$x_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_x())
            }

            #[inline(always)]
            pub fn get_y(&self) -> $y_wrapper {
                <$y_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_y())
            }

            #[inline(always)]
            pub fn get_z(&self) -> $z_wrapper {
                <$z_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_z())
            }

            #[inline(always)]
            pub fn get_w(&self) -> $w_wrapper {
                <$w_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(*self.0.get_w())
            }

            #[inline(always)]
            pub fn does_coordinate_fit(&self, coordinate: $coordinate_wrapper) -> bool {
                self.0.does_coordinate_fit(
                    <$coordinate_wrapper as $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>>::unwrap(coordinate)
                )
            }

            #[inline(always)]
            pub const fn const_unwrap(self) -> $base_dimensions {
                self.0
            }
        }

        $crate::__impl_quantized_spatial_wrapper_base_concrete!(
            $struct_name,
            $quant_element,
            $base_dimensions,
            $quant_bound
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_quantized_spatial_index_dimensions_2d_methods {
    ($struct_name:ident, $base_coordinate:ty, $coordinate_wrapper:ident, $linear_wrapper:ident) => {
        impl<QE> $struct_name<QE>
        where
            QE: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $coordinate_wrapper<QE>: $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>,
            $linear_wrapper<QE>: $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<QE>,
        {
            #[inline(always)]
            pub fn max_linear_index(&self) -> $linear_wrapper<QE> {
                <$linear_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(
                    self.0.max_linear_index()
                )
            }

            #[inline(always)]
            pub fn coordinate_to_linear_index(&self, coordinate: $coordinate_wrapper<QE>) -> $linear_wrapper<QE> {
                <$linear_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::wrap(
                    self.0.coordinate_to_linear_index(
                        <$coordinate_wrapper<QE> as $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>>::unwrap(coordinate)
                    )
                )
            }

            #[inline(always)]
            pub fn linear_index_to_coordinate(&self, linear_index: $linear_wrapper<QE>) -> $coordinate_wrapper<QE> {
                <$coordinate_wrapper<QE> as $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>>::wrap(
                    self.0.linear_index_to_coordinate(
                        <$linear_wrapper<QE> as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<QE>>::unwrap(linear_index)
                    )
                )
            }
        }
    };
    (
        concrete $struct_name:ident,
        $quant_element:ty,
        $base_coordinate:ty,
        $coordinate_wrapper:ident,
        $linear_wrapper:ident
    ) => {
        impl $struct_name
        where
            $quant_element: $crate::quantizable_linear::base_types::QuantizedIndexCountTrait,
            $coordinate_wrapper: $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>,
            $linear_wrapper: $crate::quantizable_linear::wrappers::QuantizedElementWrapperIndexCount<$quant_element>,
        {
            #[inline(always)]
            pub fn max_linear_index(&self) -> $linear_wrapper {
                <$linear_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(
                    self.0.max_linear_index()
                )
            }

            #[inline(always)]
            pub fn coordinate_to_linear_index(&self, coordinate: $coordinate_wrapper) -> $linear_wrapper {
                <$linear_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::wrap(
                    self.0.coordinate_to_linear_index(
                        <$coordinate_wrapper as $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>>::unwrap(coordinate)
                    )
                )
            }

            #[inline(always)]
            pub fn linear_index_to_coordinate(&self, linear_index: $linear_wrapper) -> $coordinate_wrapper {
                <$coordinate_wrapper as $crate::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase<$base_coordinate>>::wrap(
                    self.0.linear_index_to_coordinate(
                        <$linear_wrapper as $crate::quantizable_linear::wrappers::QuantizedElementWrapperBase<$quant_element>>::unwrap(linear_index)
                    )
                )
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_quantized_spatial_index_dimensions_3d_methods {
    ($struct_name:ident, $base_coordinate:ty, $coordinate_wrapper:ident, $linear_wrapper:ident) => {
        $crate::__impl_quantized_spatial_index_dimensions_2d_methods!(
            $struct_name,
            $base_coordinate,
            $coordinate_wrapper,
            $linear_wrapper
        );
    };
    (concrete $struct_name:ident, $quant_element:ty, $base_coordinate:ty, $coordinate_wrapper:ident, $linear_wrapper:ident) => {
        $crate::__impl_quantized_spatial_index_dimensions_2d_methods!(
            concrete $struct_name,
            $quant_element,
            $base_coordinate,
            $coordinate_wrapper,
            $linear_wrapper
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_quantized_spatial_index_dimensions_4d_methods {
    ($struct_name:ident, $base_coordinate:ty, $coordinate_wrapper:ident, $linear_wrapper:ident) => {
        $crate::__impl_quantized_spatial_index_dimensions_2d_methods!(
            $struct_name,
            $base_coordinate,
            $coordinate_wrapper,
            $linear_wrapper
        );
    };
    (concrete $struct_name:ident, $quant_element:ty, $base_coordinate:ty, $coordinate_wrapper:ident, $linear_wrapper:ident) => {
        $crate::__impl_quantized_spatial_index_dimensions_2d_methods!(
            concrete $struct_name,
            $quant_element,
            $base_coordinate,
            $coordinate_wrapper,
            $linear_wrapper
        );
    };
}

