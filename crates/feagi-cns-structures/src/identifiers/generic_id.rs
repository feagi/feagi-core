


pub trait GenericID<Backing>: core::fmt::Debug + Clone + PartialEq + Eq + core::hash::Hash +
Send + Sync + 'static + core::fmt::Display + serde::Serialize + serde::de::DeserializeOwned
where
    Backing: core::fmt::Debug + Clone + PartialEq + Eq + core::hash::Hash +
    Send + Sync + 'static + core::fmt::Display + serde::Serialize + serde::de::DeserializeOwned
{
    fn from_backing(backing: &Backing) -> Self;

    fn get_backing(&self) -> &Backing;

    // TODO to / try from URL parameter


}