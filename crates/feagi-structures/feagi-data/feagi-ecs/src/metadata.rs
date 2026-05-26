//! Some collections may have some metadata that can be used by CPU functions to quickly gather
//! context about the nature of the actual data without needing to process or access the actual
//! data


pub trait FeagiECSCollectionMetadata<Metadata> {
    fn get_metadata(&self) -> &Metadata;
}