//! Macros to make quick to add methods to traits (and concrete implementations) to enforce
//! the existence of members in structs

//region Trait methods
#[macro_export]
macro_rules! define_ref_immut_mut_access_trait_methods {
    ($property:ident, $type:ty) => {
        ::paste::paste! {
            fn [<get_ $property>](&self) -> &$type;
            fn [<get_ $property _mut>](&mut self) -> &mut $type;
        }
    };
}
#[macro_export]
macro_rules! define_ref_immut_access_trait_methods {
    ($property:ident, $type:ty) => {
        ::paste::paste! {
            fn [<get_ $property>](&self) -> &$type;
        }
    };
}
#[macro_export]
macro_rules! define_ref_mut_access_trait_methods {
    ($property:ident, $type:ty) => {
        ::paste::paste! {
            fn [<get_ $property _mut>](&mut self) -> &mut $type;
        }
    };
}

//endregion

//region Concrete Trait Implementations

#[macro_export]
macro_rules! define_ref_immut_mut_access_concrete_methods {
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

#[macro_export]
macro_rules! define_ref_immut_access_concrete_methods {
    ($property_name:ident, $type:ty, $member_name:ident) => {
        ::paste::paste! {
            #[inline(always)]
            fn [<get_ $property_name>](&self) -> &$type {
                &self.$member_name
            }
        }
    };
}

#[macro_export]
macro_rules! define_ref_mut_access_concrete_methods {
    ($property_name:ident, $type:ty, $member_name:ident) => {
        ::paste::paste! {
            #[inline(always)]
            fn [<get_ $property_name _mut>](&mut self) -> &mut $type {
                &mut self.$member_name
            }
        }
    };
}

//endregion