use crate::define_quantized_index_count_wrapper_cpu;

#[macro_export]
macro_rules! define_unsigned_spatial_4d_cpu_wrappers {
    (
        $coordinate_vis:vis struct $coordinate_struct_name:ident,
        $dimension_vis:vis struct $dimension_struct_name:ident,
        $linear_wrapper_type:ty,
        $x_axis_wrapper_type:ty,
        $y_axis_wrapper_type:ty,
        $z_axis_wrapper_type:ty,
        $t_axis_wrapper_type:ty
    ) => {
        #[repr(C)]
        #[derive(Copy, Clone)]
        $coordinate_vis struct $coordinate_struct_name<
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
        >
        where
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            x: $x_axis_wrapper_type,
            y: $y_axis_wrapper_type,
            z: $z_axis_wrapper_type,
            t: $t_axis_wrapper_type,
            _p: core::marker::PhantomData<($linear_wrapper_type, QuantIndex)>,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        $dimension_vis struct $dimension_struct_name<
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
        >
        where
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            x: $x_axis_wrapper_type,
            y: $y_axis_wrapper_type,
            z: $z_axis_wrapper_type,
            t: $t_axis_wrapper_type,
            _p: core::marker::PhantomData<($linear_wrapper_type, QuantIndex, CoordinateType)>,
        }

        impl<QuantIndex> $coordinate_struct_name<QuantIndex>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            #[inline(always)]
            pub fn new_checked(
                x: $x_axis_wrapper_type,
                y: $y_axis_wrapper_type,
                z: $z_axis_wrapper_type,
                t: $t_axis_wrapper_type,
            ) -> Result<Self, $crate::feagi_quantized_hardware_error::FeagiQuantizedHardwareError> {
                $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::verify_linear_index_within_unsigned_4d_bounds(
                    *<$x_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&x),
                    *<$y_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&y),
                    *<$z_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&z),
                    *<$t_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&t),
                )?;
                Ok(Self {
                    x,
                    y,
                    z,
                    t,
                    _p: core::marker::PhantomData,
                })
            }
        }

        impl<QuantIndex> $crate::quantizable_spatial::shared_spatial_traits::SpatialUnsignedBaseXDTrait<QuantIndex, $linear_wrapper_type>
            for $coordinate_struct_name<QuantIndex>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
        }

        impl<QuantIndex> $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedBase4DTrait<QuantIndex, $linear_wrapper_type>
            for $coordinate_struct_name<QuantIndex>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            type AxisXIndexWrapperType = $x_axis_wrapper_type;
            type AxisYIndexWrapperType = $y_axis_wrapper_type;
            type AxisZIndexWrapperType = $z_axis_wrapper_type;
            type AxisTIndexWrapperType = $t_axis_wrapper_type;

            #[inline(always)]
            fn get_x(&self) -> Self::AxisXIndexWrapperType {
                self.x
            }

            #[inline(always)]
            fn get_y(&self) -> Self::AxisYIndexWrapperType {
                self.y
            }

            #[inline(always)]
            fn get_z(&self) -> Self::AxisZIndexWrapperType {
                self.z
            }

            #[inline(always)]
            fn get_t(&self) -> Self::AxisTIndexWrapperType {
                self.t
            }

            #[inline(always)]
            fn get_x_mut(&mut self) -> &mut Self::AxisXIndexWrapperType {
                &mut self.x
            }

            #[inline(always)]
            fn get_y_mut(&mut self) -> &mut Self::AxisYIndexWrapperType {
                &mut self.y
            }

            #[inline(always)]
            fn get_z_mut(&mut self) -> &mut Self::AxisZIndexWrapperType {
                &mut self.z
            }

            #[inline(always)]
            fn get_t_mut(&mut self) -> &mut Self::AxisTIndexWrapperType {
                &mut self.t
            }

            #[inline(always)]
            fn set_x(&mut self, new_value: Self::AxisXIndexWrapperType) {
                self.x = new_value;
            }

            #[inline(always)]
            fn set_y(&mut self, new_value: Self::AxisYIndexWrapperType) {
                self.y = new_value;
            }

            #[inline(always)]
            fn set_z(&mut self, new_value: Self::AxisZIndexWrapperType) {
                self.z = new_value;
            }

            #[inline(always)]
            fn set_t(&mut self, new_value: Self::AxisTIndexWrapperType) {
                self.t = new_value;
            }

            #[inline(always)]
            fn new_unchecked(x: QuantIndex, y: QuantIndex, z: QuantIndex, t: QuantIndex) -> Self {
                Self {
                    x: <$x_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::wrap_quant(x),
                    y: <$y_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::wrap_quant(y),
                    z: <$z_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::wrap_quant(z),
                    t: <$t_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::wrap_quant(t),
                    _p: core::marker::PhantomData,
                }
            }
        }

        impl<QuantIndex> $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>
            for $coordinate_struct_name<QuantIndex>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
        }

        impl<QuantIndex> core::ops::Add for $coordinate_struct_name<QuantIndex>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                    z: self.z + rhs.z,
                    t: self.t + rhs.t,
                    _p: core::marker::PhantomData,
                }
            }
        }

        impl<QuantIndex> core::ops::Sub for $coordinate_struct_name<QuantIndex>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                    z: self.z - rhs.z,
                    t: self.t - rhs.t,
                    _p: core::marker::PhantomData,
                }
            }
        }

        impl<QuantIndex> core::ops::AddAssign for $coordinate_struct_name<QuantIndex>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.x += rhs.x;
                self.y += rhs.y;
                self.z += rhs.z;
                self.t += rhs.t;
            }
        }

        impl<QuantIndex> core::ops::SubAssign for $coordinate_struct_name<QuantIndex>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.x -= rhs.x;
                self.y -= rhs.y;
                self.z -= rhs.z;
                self.t -= rhs.t;
            }
        }

        #[cfg(feature = "alloc")]
        impl<QuantIndex> core::fmt::Debug for $coordinate_struct_name<QuantIndex>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter
                    .debug_struct(stringify!($coordinate_struct_name))
                    .field("x", <$x_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.x))
                    .field("y", <$y_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.y))
                    .field("z", <$z_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.z))
                    .field("t", <$t_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.t))
                    .finish()
            }
        }

        #[cfg(feature = "alloc")]
        impl<QuantIndex> core::fmt::Display for $coordinate_struct_name<QuantIndex>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(
                    formatter,
                    "({}, {}, {}, {})",
                    <$x_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.x),
                    <$y_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.y),
                    <$z_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.z),
                    <$t_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.t)
                )
            }
        }

        impl<QuantIndex, CoordinateType> $dimension_struct_name<QuantIndex, CoordinateType>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            #[inline(always)]
            pub fn new_checked(
                x: $x_axis_wrapper_type,
                y: $y_axis_wrapper_type,
                z: $z_axis_wrapper_type,
                t: $t_axis_wrapper_type,
            ) -> Result<Self, $crate::feagi_quantized_hardware_error::FeagiQuantizedHardwareError> {
                $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::verify_linear_index_within_unsigned_4d_bounds(
                    *<$x_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&x),
                    *<$y_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&y),
                    *<$z_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&z),
                    *<$t_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&t),
                )?;
                Ok(Self {
                    x,
                    y,
                    z,
                    t,
                    _p: core::marker::PhantomData,
                })
            }
        }

        impl<QuantIndex, CoordinateType> $crate::quantizable_spatial::shared_spatial_traits::SpatialUnsignedBaseXDTrait<QuantIndex, $linear_wrapper_type>
            for $dimension_struct_name<QuantIndex, CoordinateType>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
        }

        impl<QuantIndex, CoordinateType> $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedBase4DTrait<QuantIndex, $linear_wrapper_type>
            for $dimension_struct_name<QuantIndex, CoordinateType>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            type AxisXIndexWrapperType = $x_axis_wrapper_type;
            type AxisYIndexWrapperType = $y_axis_wrapper_type;
            type AxisZIndexWrapperType = $z_axis_wrapper_type;
            type AxisTIndexWrapperType = $t_axis_wrapper_type;

            #[inline(always)]
            fn get_x(&self) -> Self::AxisXIndexWrapperType {
                self.x
            }

            #[inline(always)]
            fn get_y(&self) -> Self::AxisYIndexWrapperType {
                self.y
            }

            #[inline(always)]
            fn get_z(&self) -> Self::AxisZIndexWrapperType {
                self.z
            }

            #[inline(always)]
            fn get_t(&self) -> Self::AxisTIndexWrapperType {
                self.t
            }

            #[inline(always)]
            fn get_x_mut(&mut self) -> &mut Self::AxisXIndexWrapperType {
                &mut self.x
            }

            #[inline(always)]
            fn get_y_mut(&mut self) -> &mut Self::AxisYIndexWrapperType {
                &mut self.y
            }

            #[inline(always)]
            fn get_z_mut(&mut self) -> &mut Self::AxisZIndexWrapperType {
                &mut self.z
            }

            #[inline(always)]
            fn get_t_mut(&mut self) -> &mut Self::AxisTIndexWrapperType {
                &mut self.t
            }

            #[inline(always)]
            fn set_x(&mut self, new_value: Self::AxisXIndexWrapperType) {
                self.x = new_value;
            }

            #[inline(always)]
            fn set_y(&mut self, new_value: Self::AxisYIndexWrapperType) {
                self.y = new_value;
            }

            #[inline(always)]
            fn set_z(&mut self, new_value: Self::AxisZIndexWrapperType) {
                self.z = new_value;
            }

            #[inline(always)]
            fn set_t(&mut self, new_value: Self::AxisTIndexWrapperType) {
                self.t = new_value;
            }

            #[inline(always)]
            fn new_unchecked(x: QuantIndex, y: QuantIndex, z: QuantIndex, t: QuantIndex) -> Self {
                Self {
                    x: <$x_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::wrap_quant(x),
                    y: <$y_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::wrap_quant(y),
                    z: <$z_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::wrap_quant(z),
                    t: <$t_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::wrap_quant(t),
                    _p: core::marker::PhantomData,
                }
            }
        }

        impl<QuantIndex, CoordinateType> $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialDimension4DTrait<QuantIndex, $linear_wrapper_type, CoordinateType>
            for $dimension_struct_name<QuantIndex, CoordinateType>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
        }

        impl<QuantIndex, CoordinateType> core::ops::Add for $dimension_struct_name<QuantIndex, CoordinateType>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                    z: self.z + rhs.z,
                    t: self.t + rhs.t,
                    _p: core::marker::PhantomData,
                }
            }
        }

        impl<QuantIndex, CoordinateType> core::ops::Sub for $dimension_struct_name<QuantIndex, CoordinateType>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                    z: self.z - rhs.z,
                    t: self.t - rhs.t,
                    _p: core::marker::PhantomData,
                }
            }
        }

        impl<QuantIndex, CoordinateType> core::ops::AddAssign for $dimension_struct_name<QuantIndex, CoordinateType>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.x += rhs.x;
                self.y += rhs.y;
                self.z += rhs.z;
                self.t += rhs.t;
            }
        }

        impl<QuantIndex, CoordinateType> core::ops::SubAssign for $dimension_struct_name<QuantIndex, CoordinateType>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.x -= rhs.x;
                self.y -= rhs.y;
                self.z -= rhs.z;
                self.t -= rhs.t;
            }
        }

        #[cfg(feature = "alloc")]
        impl<QuantIndex, CoordinateType> core::fmt::Debug for $dimension_struct_name<QuantIndex, CoordinateType>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter
                    .debug_struct(stringify!($dimension_struct_name))
                    .field("x", <$x_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.x))
                    .field("y", <$y_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.y))
                    .field("z", <$z_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.z))
                    .field("t", <$t_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.t))
                    .finish()
            }
        }

        #[cfg(feature = "alloc")]
        impl<QuantIndex, CoordinateType> core::fmt::Display for $dimension_struct_name<QuantIndex, CoordinateType>
        where
            QuantIndex: $crate::quantizable_base::index_count::QuantizedIndexCountTrait,
            CoordinateType: $crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::SpatialUnsignedCoordinate4DTrait<QuantIndex, $linear_wrapper_type>,
            $linear_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $x_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $y_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $z_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
            $t_axis_wrapper_type: $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>,
        {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(
                    formatter,
                    "({}, {}, {}, {})",
                    <$x_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.x),
                    <$y_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.y),
                    <$z_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.z),
                    <$t_axis_wrapper_type as $crate::quantizable_base::index_count::QuantizedIndexCountWrapperTrait<QuantIndex>>::quant_ref(&self.t)
                )
            }
        }
    };
}

/*
define_quantized_index_count_wrapper_cpu!(pub struct TestA);
define_quantized_index_count_wrapper_cpu!(pub struct TestB);
define_quantized_index_count_wrapper_cpu!(pub struct TestC);
define_quantized_index_count_wrapper_cpu!(pub struct TestD);
define_quantized_index_count_wrapper_cpu!(pub struct TestE);

define_unsigned_spatial_4d_cpu_wrappers!(
    pub struct SpatialUnsignedCoordinate4D,
    pub struct SpatialUnsignedDimension4D,
    TestA<QuantIndex>,
    TestB<QuantIndex>,
    TestC<QuantIndex>,
    TestD<QuantIndex>,
    TestE<QuantIndex>
);

 */
