use crate::basis::specifications::brain_regions::BrainRegionID;

pub trait BrainRegionContainer
{
    
}

pub trait BrainRegionContainerNonRoot: BrainRegionContainer
{
    
}

pub trait BrainRegionContainerRoot: BrainRegionContainer
{

}


pub mod designer 
{
    use std::collections::HashSet;
    use crate::basis::cortical_area::cortical_identifier::CorticalIdentifier;
    use crate::basis::specifications::brain_regions::BrainRegionID;

    
    pub struct BrainRegionContextDesignerRoot
    {
        cache_contained_device_area_ids: HashSet<CorticalIdentifier>
    }




    pub struct BrainRegionContextDesignerNonRoot
    {
        pub parent_region_id: BrainRegionID,
        
        cache_contained_device_area_ids: HashSet<CorticalIdentifier>
    }
    
    
    
    
    

}
