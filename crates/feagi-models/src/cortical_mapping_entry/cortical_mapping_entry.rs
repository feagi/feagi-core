use std::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::cortical_area::cortical_activity::CorticalActivity;
use crate::cortical_area::cortical_area_layout::CorticalAreaLayout;
use crate::cortical_area::fire_ledger::FireLedger;
use crate::cortical_mapping_entry::doublet::doublet_iterator::DoubletIterator;
use crate::cortical_mapping_entry::synapse_model::synapse_model::SynapseModel;
use crate::cortical_mapping_entry::synapse_model::synapse_model_quantization::SynapseModelQuantization;

pub struct CorticalMappingEntry<FIQ, SMQ, SG, SL, DL, DI, FL, SM,>
where
    FIQ: FeagiIndexQuantization,
    SMQ: SynapseModelQuantization,
    SG: CorticalActivity<FIQ, SMQ::JunctionPotentialQuant>, // source gate
    SL: CorticalAreaLayout<FIQ>,
    DL: CorticalAreaLayout<FIQ>,
    DI: DoubletIterator<FIQ, SL, DL>,
    FL: FireLedger<FIQ>,
    SM: SynapseModel<FIQ, SMQ>,
{
    _p: PhantomData<(FIQ, SMQ, SG, SL, DL, DI, DI, FL, SM)>,
}