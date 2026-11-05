use feagi_connectome::quantization_levels::FeagiIndexQuantization;
use crate::genome::genome_editing::genome_edit_steps::brain_region::GenomeEditStepBrainRegionEnum;
use crate::genome::genome_editing::GenomeEditSequence;

pub mod cortical_area;
pub mod brain_region;
pub mod connectivity_rule;
pub mod cortical_mapping;


pub trait GenomeEditStepType<FIQ: FeagiIndexQuantization>: Sized
{
    /// The sub category of edits
    type SuperEditEnum: Into<GenomeEditStepEnum<FIQ>>;
    
    type ConnectomeEditStep;

    /// wrap into a sub category enum
    fn wrap_to_sub_category_enum(self) -> Self::SuperEditEnum;
    
    /// First All genome edit steps are checked in sequence to ensure plausibility
    fn genome_validity_check(&self, ) -> Result<(),()>;
    
    /// Construct the Connectome Edit Step (May be a null which does nothing)
    fn construct_connectome_edit_step(&self) -> Self::ConnectomeEditStep;

    fn apply_edit_to_genome(&self, connectome_edit_response: (), genome: ()) -> Result<(),()>;

    /// wrap into an enum and inject into the sequence
    fn wrap_to_enum_and_inject(self, sequence: &mut GenomeEditSequence<FIQ>) {

        let sub_enum = self.wrap_to_sub_category_enum();
        let wrapped_enum = sub_enum.into();
        
        // TODO from ConnectomeEditStep
        const EDITABLE: bool = true;
        
        sequence.add_genome_edit_enum(wrapped_enum, EDITABLE);
    }
}


pub(crate) enum GenomeEditStepEnum<FIQ: FeagiIndexQuantization> {
    BrainRegion(GenomeEditStepBrainRegionEnum),
    CorticalArea(core::marker::PhantomData<FIQ>), // TODO cortical area edits, which will use FIQ
    ConnectivityRule(),
    CorticalMapping(),
}

impl<FIQ: FeagiIndexQuantization> From<GenomeEditStepBrainRegionEnum> for GenomeEditStepEnum<FIQ> {
    fn from(value: GenomeEditStepBrainRegionEnum) -> Self {
        GenomeEditStepEnum::BrainRegion(value)
    }
}
