
#[doc(hidden)]
/// Common base trait for quantized spatial wrappers. DO not implement directly as we make
/// use of unsafe functions internally, only make wrappers through the macros!
pub trait QuantizedSpatialWrapperBase<Spatial>: Sized {
    fn wrap(spatial: Spatial) -> Self;

    fn wrap_ref(spatial: &Spatial) -> &Self; // scary!

    fn unwrap(self) -> Spatial;

    fn spatial_ref(&self) -> &Spatial;

    fn spatial_ref_mut(&mut self) -> &mut Spatial;
}
