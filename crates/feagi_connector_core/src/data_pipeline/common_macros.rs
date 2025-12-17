
/// Macro to quickly define a new pipeline stage properties struct with all required implementations.
///
/// This macro generates:
/// - The struct definition with `#[derive(Debug, Clone)]`
/// - `PipelineStageProperties` trait implementation
/// - `new()` and `new_box()` constructors
/// - `Display` trait implementation
/// - `TryFrom` implementations for downcasting from boxed trait objects
///
/// # Usage
/// ```ignore
/// define_stage_properties! {
///     /// Optional documentation for the struct
///     name: MyStageProperties,
///     
///     fields: {
///         pub field1: Type1,
///         pub field2: Type2,
///     },
///     
///     input_type: |s| WrappedIOType::SomeType(s.field1),
///     output_type: |s| WrappedIOType::SomeType(s.field2),
///     
///     create_stage: |s| {
///         MyStage::new_box(s.field1.clone(), s.field2.clone()).unwrap()
///     },
///     
///     display: ("MyStageProperties(field1: {:?}, field2: {:?})", field1, field2),
/// }
/// ```
#[macro_export]
macro_rules! define_stage_properties {
    (
        $(#[$meta:meta])*
        name: $name:ident,
        
        fields: {
            $(
                $(#[$field_meta:meta])*
                $field_name:ident : $field_type:ty
            ),* $(,)?
        },
        
        input_type: |$self_in:ident| $input_expr:expr,
        output_type: |$self_out:ident| $output_expr:expr,
        
        create_stage: |$self_stage:ident| $create_stage_expr:expr,
        
        display: ($display_format:expr, $($display_field:ident),* $(,)?),
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field_name: $field_type,
            )*
        }
        
        impl $crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties for $name {
            fn get_input_data_type(&self) -> $crate::wrapped_io_data::WrappedIOType {
                let $self_in = self;
                $input_expr
            }
            
            fn get_output_data_type(&self) -> $crate::wrapped_io_data::WrappedIOType {
                let $self_out = self;
                $output_expr
            }
            
            fn clone_box(&self) -> Box<dyn $crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties + Sync + Send> {
                Box::new(self.clone())
            }
            
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn create_stage(&self) -> Box<dyn $crate::data_pipeline::PipelineStage> {
                let $self_stage = self;
                $create_stage_expr
            }
        }
        
        impl $name {
            /// Creates a new instance of the stage properties.
            pub fn new($($field_name: $field_type),*) -> Self {
                Self {
                    $($field_name,)*
                }
            }
            
            /// Creates a new boxed instance of the stage properties.
            pub fn new_box($($field_name: $field_type),*) -> Box<dyn $crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties + Send + Sync> {
                Box::new(Self::new($($field_name),*))
            }
        }
        
        // Display implementation
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, $display_format, $(self.$display_field),*)
            }
        }
        
        // TryFrom for reference downcasting
        impl<'a> TryFrom<&'a Box<dyn $crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties>> for &'a $name {
            type Error = feagi_data_structures::FeagiDataError;
            fn try_from(value: &'a Box<dyn $crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties>) -> Result<Self, Self::Error> {
                match value.as_any().downcast_ref::<$name>() {
                    Some(p) => Ok(p),
                    None => Err(feagi_data_structures::FeagiDataError::InternalError(
                        concat!("Given stage attempted to be cast as '&", stringify!($name), "' when it isn't!").into()
                    ))
                }
            }
        }
        
        // TryFrom for owned downcasting
        impl TryFrom<Box<dyn $crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties>> for $name {
            type Error = feagi_data_structures::FeagiDataError;
            fn try_from(value: Box<dyn $crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties>) -> Result<Self, Self::Error> {
                match value.as_any().downcast_ref::<$name>() {
                    Some(p) => Ok(p.clone()),
                    None => Err(feagi_data_structures::FeagiDataError::InternalError(
                        concat!("Given stage attempted to be cast as '", stringify!($name), "' when it isn't!").into()
                    ))
                }
            }
        }
    };
}