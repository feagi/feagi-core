/// Common base trait for quantized spatial wrappers.
///
/// Spatial wrappers are transparent newtypes around one of the generic spatial
/// coordinate or dimension structs. Their public constructors and accessors use
/// linear quantized wrappers per axis, while this trait gives access to the
/// enclosed spatial value.
pub trait QuantizedSpatialWrapperBase<Spatial>: Sized {
    fn wrap(spatial: Spatial) -> Self;

    fn unwrap(self) -> Spatial;

    fn spatial_ref(&self) -> &Spatial;

    fn spatial_ref_mut(&mut self) -> &mut Spatial;
}
