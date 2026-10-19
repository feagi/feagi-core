use feagi_interop_proc::generate_built_feagi_message_path_impls;
use core::str::FromStr;

#[cfg(feature = "picoserve")]
use picoserve::routing::ParsePathSegment;
#[cfg(feature = "picoserve")]
use picoserve::routing::parse_path_segment;

/// Wrapper marker struct. Intended to be extended via impls for various tuple combinations on the
/// implementing end to target different use cases
pub struct FeagiMessagePathTupleConverter<PathTuple>(core::marker::PhantomData<PathTuple>);

impl<PathTuple> FeagiMessagePathTupleConverter<PathTuple> {
    pub fn new() -> FeagiMessagePathTupleConverter<PathTuple> {
        FeagiMessagePathTupleConverter(core::marker::PhantomData)
    }
}

// Yes this is stupid. This generates 1k lines of impls
generate_built_feagi_message_path_impls!();
