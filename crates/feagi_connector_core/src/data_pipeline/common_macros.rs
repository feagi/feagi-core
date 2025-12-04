
#[macro_export]
macro_rules! pipeline_stage_property_implementations {
    ($struct_type:ty, $display_format:expr, $($field:ident),* $(,)?) => {
        impl std::fmt::Display for $struct_type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, $display_format, $(self.$field),*)
            }
        }

        impl<'a> TryFrom<&'a Box<dyn PipelineStageProperties>> for &'a $struct_type {
            type Error = FeagiDataError;
            fn try_from(value: &'a Box<dyn PipelineStageProperties>) -> Result<Self, Self::Error> {
                match value.as_any().downcast_ref::<$struct_type>() {
                    Some(p) => Ok(p),
                    None => Err(FeagiDataError::InternalError(
                        concat!("Given stage attempted to be cast as '&", stringify!($struct_type), "' when it isn't!").into()
                    ))
                }
            }
        }

        impl TryFrom<Box<dyn PipelineStageProperties>> for $struct_type {
            type Error = FeagiDataError;
            fn try_from(value: Box<dyn PipelineStageProperties>) -> Result<Self, Self::Error> {
                match value.as_any().downcast_ref::<$struct_type>() {
                    Some(p) => Ok(p.clone()),
                    None => Err(FeagiDataError::InternalError(
                        concat!("Given stage attempted to be cast as '", stringify!($struct_type), "' when it isn't!").into()
                    ))
                }
            }
        }
    };
}