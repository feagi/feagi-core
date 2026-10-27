use feagi_brain_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_area_container::CorticalAreaContainer;
use crate::graphs;
use crate::graphs::components::element_identifiers::{ConcreteGraphElementIdentifiers, GraphElementIdentifiers};
use crate::graphs::components::npu_context::GraphNPUContext;

/// Represents the layout and (a variable amount of) data of a FEAGI brain
pub trait FeagiGraph<FIQ: FeagiIndexQuantization>
{
    // Note: To be extended by `GraphConnectomeDataAsync` or `GraphConnectomeDataSync`
    /// What identifiers are used
    type ElementIdentifiers: GraphElementIdentifiers;

    /// The types of data contained that is relevant to NPU communication / translation
    type NPUContext: GraphNPUContext;


    //region Reading Properties

    fn get_cortical_area(&self, cortical_id: &<ConcreteGraphElementIdentifiers as GraphElementIdentifiers>::CorticalIdentifier) -> Option<&CorticalAreaContainer<FIQ, CacheData>>;



    //region Default Impls

    // TODO Multis

    //endregion


    //endregion
}








// Storage (stored per data or writer), Storage Light (writer only), runtime (from npu), amalgamation (stored per or writer, faux identifiers)

