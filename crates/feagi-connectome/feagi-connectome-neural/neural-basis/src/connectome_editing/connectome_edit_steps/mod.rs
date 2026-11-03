pub mod cortical_area;

use crate::connectome_editing::connectome_edit_steps::cortical_area::{ConnectomeEditResultantCorticalAreaEnum, ConnectomeEditStepCorticalAreaEnum};
use crate::connectome_editing::ConnectomeVisitorDispatchWrapper;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;


pub trait ConnectomeEditStepType<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>>:
Sized + Into<ConnectomeEditStepEnum<FIQ, CVDW>> {
    type ConnectomeEditResultant: Into<ConnectomeEditResultantEnum<FIQ>>;

}

pub enum ConnectomeEditStepEnum<FIQ: FeagiIndexQuantization, CVDW: ConnectomeVisitorDispatchWrapper<FIQ>> {
    /// Used by GenomeEditStep to skip over edits that dont affect the connectome
    Nothing,
    CorticalArea(ConnectomeEditStepCorticalAreaEnum<FIQ, CVDW>),
}


pub enum ConnectomeEditResultantEnum<FIQ: FeagiIndexQuantization> {
    /// Used by GenomeEditStep to skip over edits that dont affect the connectome
    Nothing,
    CorticalArea(ConnectomeEditResultantCorticalAreaEnum<FIQ>),
}