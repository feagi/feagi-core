#[macro_export]
macro_rules! define_ref_access_trait_methods {
    ($property:ident, $type:ty) => {
        ::paste::paste! {
            fn [<get_ $property>](&self) -> &$type;
            fn [<get_ $property _mut>](&mut self) -> &mut $type;
        }
    };
}

#[macro_export]
macro_rules! define_ref_access_concrete_methods {
    ($property_name:ident, $type:ty, $member_name:ident) => {
        ::paste::paste! {
            #[inline(always)]
            fn [<get_ $property_name>](&self) -> &$type {
                &self.$member_name
            }
            #[inline(always)]
            fn [<get_ $property_name _mut>](&mut self) -> &mut $type {
                &mut self.$member_name
            }
        }
    };
}