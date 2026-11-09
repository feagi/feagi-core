pub mod designer;
pub mod storage;






#[derive(Debug, thiserror::Error)]
pub enum GenomeError {
    #[error("Unable to find brain region of given ID!")]
    InvalidParentRegionID,
    #[error("Given region ID already exists!")]
    DuplicateParentRegionID,
    #[error("This operation cannot be applied to the root region!")]
    InvalidRootIDOperation,
}










// TODO review and implement


/*


/// Common Genome Reading Properties
pub trait FeagiGenomeReading<FIQ: FeagiIndexQuantization>
{
    fn get_cortical_areas<'a>(&self, cortical_ids: &[CorticalIdentifier]) -> impl ExactSizeIterator<Item=Result<&'a DesignerCorticalAreaContainer<FIQ>, ()>>;

    
    
    
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


 */