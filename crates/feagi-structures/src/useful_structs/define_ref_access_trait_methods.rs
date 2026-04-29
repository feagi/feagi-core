#[macro_export]
macro_rules! define_ref_access_trait_methods {
    ($property:ident, $type:ty) => {
        ::paste::paste! {
            fn [<get_ $property>](&self) -> &$type;
            fn [<get_ $property _mut>](&mut self) -> &mut $type;
        }
    };
}