


// NOTE: all below are internal implementations, do not call them!

// TODO we can make a separate mortality macro by just asking for the field name

//region Base Linear

/// Creates Neuron Structs and Implements base neuron traits
macro_rules! __internal_neuron_generate_base_neuron_structs_and_traits{
    (
        $model_neuron_name:ident,
        $model_collection_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {

            pub struct [<$model_neuron_name Neuron>]<CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> {
                membrane_potential: $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant>,
                $(
                    $field : $ty,
                )*
            }

            pub struct [<$model_neuron_name NeuronRef>]<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization + 'a> {
                membrane_potential: &'a $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant>,
                $(
                    $field : &'a $ty,
                )*
            }

            pub struct [<$model_neuron_name NeuronRefMut>]<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization + 'a> {
                membrane_potential: &'a mut $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant>,
                $(
                    $field : &'a mut $ty,
                )*
            }



            pub trait [<$model_neuron_name NeuronTrait>]<CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization>:
            $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronTrait<CANQ>
            {
                $(
                    fn [<get_ $field>](&self) -> &$ty;
                    fn [<get_ $field _mut>](&mut self) -> &mut $ty;
                )*
            }

            pub trait [<$model_neuron_name NeuronRefTrait>]<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization>:
            $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronRefTrait<'a, CANQ>
            {
                $(
                    fn [<get_ $field>](&self) -> &'a $ty;
                )*
            }

            pub trait [<$model_neuron_name NeuronMutRefTrait>]<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization>:
            [<$model_neuron_name NeuronRefTrait>]<'a, CANQ> +
            $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronMutRefTrait<'a, CANQ>
            {
                $(
                    fn [<get_ $field _mut>](&mut self) -> &mut $ty;
                )*
            }


            impl<CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> [<$model_neuron_name Neuron>]<CANQ> {
                #[inline(always)]
                pub fn new(
                    membrane_potential: $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant>,
                    $(
                        $field: $ty,
                    )*
                ) -> Self {
                    Self {
                        membrane_potential,
                        $(
                            $field,
                        )*
                    }
                }
            }

            impl<CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronTrait<CANQ> for [<$model_neuron_name Neuron>]<CANQ> {
                $crate::define_ref_immut_mut_access_concrete_methods!(
                    membrane_potential,
                    $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant>,
                    membrane_potential
                );
            }

            impl<CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> [<$model_neuron_name NeuronTrait>]<CANQ> for [<$model_neuron_name Neuron>]<CANQ> {
                $(
                    #[inline(always)]
                    fn [<get_ $field>](&self) -> &$ty {
                        &self.$field
                    }

                    #[inline(always)]
                    fn [<get_ $field _mut>](&mut self) -> &mut $ty {
                        &mut self.$field
                    }
                )*
            }

            impl<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronRefTrait<'a, CANQ> for [<$model_neuron_name NeuronRef>]<'a, CANQ> {
                #[inline(always)]
                fn get_membrane_potential(&self) -> &'a $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant> {
                    self.membrane_potential
                }
            }

            impl<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronRefTrait<'a, CANQ> for [<$model_neuron_name NeuronRefMut>]<'a, CANQ> {
                #[inline(always)]
                fn get_membrane_potential(&self) -> &'a $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant> {
                    &*self.membrane_potential
                }
            }

            impl<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> $crate::neuron_dynamics::code_definitions::neurons::base_neuron_model_fields::NeuronModelNeuronMutRefTrait<'a, CANQ> for [<$model_neuron_name NeuronRefMut>]<'a, CANQ> {
                #[inline(always)]
                fn get_membrane_potential_mut(&mut self) -> &mut $crate::neuron_dynamics::code_definitions::neurons::common_linear_neuron_structs::NeuronMembranePotential<CANQ::NeuronValueQuant> {
                    self.membrane_potential
                }
            }

            impl<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> [<$model_neuron_name NeuronRefTrait>]<'a, CANQ> for [<$model_neuron_name NeuronRef>]<'a, CANQ> {
                $(
                    #[inline(always)]
                    fn [<get_ $field>](&self) -> &'a $ty {
                        self.$field
                    }
                )*
            }

            impl<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> [<$model_neuron_name NeuronRefTrait>]<'a, CANQ> for [<$model_neuron_name NeuronRefMut>]<'a, CANQ> {
                $(
                    #[inline(always)]
                    fn [<get_ $field>](&self) -> &'a $ty {
                        &*self.$field
                    }
                )*
            }

            impl<'a, CANQ: $crate::quantization_level::CorticalAreaNeuronQuantization> [<$model_neuron_name NeuronMutRefTrait>]<'a, CANQ> for [<$model_neuron_name NeuronRefMut>]<'a, CANQ> {
                $(
                    #[inline(always)]
                    fn [<get_ $field _mut>](&mut self) -> &mut $ty {
                        self.$field
                    }
                )*
            }
        }
    };
}

//region Packed Collections



/// Creates Neuron Slice structs and implements their traits, for
macro_rules! __internal_neuron_generate_base_neuron_packed_shared_implementations {
    (
        $model_neuron_name:ident,
        $model_collection_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {




        }
    };
}

/// Creates Packed Resizable Vector Collection for neuron and implements their traits
macro_rules! __internal_neuron_generate_base_neuron_packed_vector_structs_and_traits{
    (
        $model_neuron_name:ident,
        $model_collection_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {


        }
    };
}

// TODO Array Implementation

//endregion

//region Single Neuron Sparse Collections

/// Creates Indexed Resizable Vector Collection for neuron and implements their traits
macro_rules! __internal_neuron_generate_base_neuron_single_neuron_indexed_vector_structs_and_traits{
    (
        $model_neuron_name:ident,
        $model_collection_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {


        }
    };
    
}

// TODO Hashmap Implementation

//endregion

//region Multi Neuron Sparse Collections

/// Creates Indexed Resizable Vector Collection for neuron and implements their traits
macro_rules! __internal_neuron_generate_base_neuron_multi_neuron_indexed_vector_structs_and_traits{
    (
        $model_neuron_name:ident,
        $model_collection_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {


        }
    };
}

macro_rules! __internal_neuron_implement_single_neuron_multi_neuron_conversion_trait {
    (
        $model_neuron_name:ident,
        $model_collection_name:ident,
        {
            $( $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        ::paste::paste! {


        }
    };
}

// TODO Hashmap Implementation

//endregion


//endregion


//region Dimensional

// TODO Shared

//region Single Neuron

//endregion

//region MultiNeuron

//endregion


//endregion