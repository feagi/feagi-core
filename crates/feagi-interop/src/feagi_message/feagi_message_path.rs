use crate::feagi_message::feagi_request_message::message_components::MessageQueryables;

/// TODO make network builders external functions that act on these

/// Represents the path (and the configurations of it) for a feagi message to take. Zero Sized,
/// ergo has no internal fields
pub trait FeagiMessagePath {
    /// A tuple of elements from `path_element_type`
    type TypesTuple;

    /// Just the Configurables as a tuple, needed to make the path instantiation functions inputs
    type ConfigurablesTuple;

    fn create_key_expr_for_zenoh(configurables: &Self::ConfigurablesTuple) -> ();
}


/// Request Response paths may be used by pico, and thus needs some of its own logic
pub trait FeagiRequestMessagePath: FeagiMessagePath {
    /// Adds a path to the picoserver router. Needs the queryables specifically
    fn add_path_to_pico_serve_router<Q: MessageQueryables>(configurables: &Self::ConfigurablesTuple) -> (); // TODO add to router func

    // TODO add paths to zenoh router specifically?
}

// TODO add data paths for zenoh router (FeagiPublishingMessagePath?)




/// The allowed types for the tuple of FeagiMessagePath
pub mod path_element_type {
    use crate::feagi_message::feagi_request_message::message_components::FeagiMessageParameterFieldSerializable;

    /// Constant parts of a path that are constant (at least by compile time)
    pub type PathStaticElement = &'static str;

    /// A part of a path that is configurable only at the path instantiation
    pub type PathConfigurableElement<'a> = &'a str;


    pub type PathParameterElement<T: FeagiMessageParameterFieldSerializable> = T;
}




