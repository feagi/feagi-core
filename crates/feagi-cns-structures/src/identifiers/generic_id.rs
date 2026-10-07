use crate::identifiers::genome_identifier_error::GenomeIdentifierError;

pub trait GenericID<Backing>: core::fmt::Debug + Clone + PartialEq + Eq + core::hash::Hash +
Send + Sync + 'static + core::fmt::Display + serde::Serialize + serde::de::DeserializeOwned
where
    Backing: core::fmt::Debug + Clone + PartialEq + Eq + core::hash::Hash +
    Send + Sync + 'static + core::fmt::Display + serde::Serialize + serde::de::DeserializeOwned
{
    const MAX_URL_PARAMETER_LENGTH: usize;
    
    // NOTE do not implement a general new() as some IDs are UUIDs which have some unique requirements

    /// Create from an existing raw data source
    fn from_backing(backing: &Backing) -> Self;

    /// Get the internal data
    fn get_backing(&self) -> &Backing;
    
    
    // TODO URL parameters may need to be a specific data type
    /*
    
    /// Create a new instance given a URL parameter string slice
    #[cfg(feature = "alloc")]
    fn try_new_from_url_parameter(parameter_section: &str) -> Result<Self, GenomeIdentifierError>;
    
    //fn try_update_from_url_parameter(&mut self,parameter_section: &str) -> Result<(), GenomeIdentifierError>;
    
     */

}