use crate::basis::cortical_area::cortical_id_legacy::CorticalIDLegacy;
use crate::genome::genomes::designer::designer_cortical_container::DesignerCorticalAreaContainer;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

// TODO for designer only right now, fix this

/// Common Genome Reading Properties
pub trait FeagiGenomeReading<FIQ: FeagiIndexQuantization>
{
    fn get_cortical_areas<'a>(&self, cortical_ids: &[CorticalIDLegacy]) -> impl ExactSizeIterator<Item=Result<&'a DesignerCorticalAreaContainer<FIQ>, ()>>;

    
    
    
    /*
    fn get_cortical_area<'a>(&self, cortical_id: &CorticalID) -> Result<&'a DesignerCorticalAreaContainer<FIQ>, ()> {
        let arr_ref: &[CorticalID; 1] = core::slice::from_ref(cortical_id).try_into().unwrap();
        let out = self.get_cortical_areas(arr_ref.as_slice()).collect();
        out
    }
    
     */
}

#[cfg(feature = "alloc")]
/// Common Genome Writing Properties
pub trait FeagiGenomeWriting<FIQ: FeagiIndexQuantization>
{
    fn add_cortical_areas<'a>(&mut self, cortical_generators: Vec<()>) -> Result<(), ()>;
}