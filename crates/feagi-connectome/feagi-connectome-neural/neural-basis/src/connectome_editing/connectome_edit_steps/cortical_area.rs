use crate::connectome_editing::connectome_edit_steps::{ConnectomeEditResultantEnum, ConnectomeEditStepEnum, ConnectomeEditStepType};
use crate::connectome_editing::ConnectomeVisitorDispatchWrapper;
use crate::neuron_model::cortical_indexing::{CorticalAreaEngineIndex};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::visualization::NeuronVisualizationWordIndex;

pub enum ConnectomeEditStepCorticalAreaEnum<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> {
    Add(ConnectomeEditStepCorticalAreaAdd<FIQ, CVDW>)
}

pub enum ConnectomeEditResultantCorticalAreaEnum<FIQ: FeagiIndexQuantization> {
    Add(ConnectomeEditResultantCorticalAreaAdd<FIQ>)
}



//region Add

//region Step

pub struct ConnectomeEditStepCorticalAreaAdd<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> {
    pub cortical_area_spec_dispatch: CVDW::CorticalAreaSpecDispatch,
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> ConnectomeEditStepCorticalAreaAdd<FIQ, CVDW>
{
    pub fn new(cortical_area_spec_dispatch: CVDW::CorticalAreaSpecDispatch) -> Self {
        Self { cortical_area_spec_dispatch, _p: core::marker::PhantomData }
    }
}

impl<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> ConnectomeEditStepType<FIQ, CVDW>
for ConnectomeEditStepCorticalAreaAdd<FIQ, CVDW> {
    type ConnectomeEditResultant = ConnectomeEditResultantCorticalAreaAdd<FIQ>;
}

impl<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> From<ConnectomeEditStepCorticalAreaAdd<FIQ, CVDW>>
for ConnectomeEditStepEnum<FIQ, CVDW> {
    fn from(value: ConnectomeEditStepCorticalAreaAdd<FIQ, CVDW>) -> Self {
        ConnectomeEditStepEnum::CorticalArea(
            ConnectomeEditStepCorticalAreaEnum::Add(value)
        )
    }
}

//endregion

//region Resultant

pub struct ConnectomeEditResultantCorticalAreaAdd<FIQ: FeagiIndexQuantization>
{
    pub new_cortical_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub visualization_start_index: NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>
}

impl<FIQ: FeagiIndexQuantization> From<ConnectomeEditResultantCorticalAreaAdd<FIQ>> for ConnectomeEditResultantEnum<FIQ>
{
    fn from(value: ConnectomeEditResultantCorticalAreaAdd<FIQ>) -> Self {
        ConnectomeEditResultantEnum::CorticalArea(
            ConnectomeEditResultantCorticalAreaEnum::Add(value)
        )
    }
}

//endregion


//endregion
