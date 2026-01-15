
pub trait ConnectionDefinition {
    fn as_box(&self) -> Box<dyn ConnectionDefinition>;

    fn as_any(&self) -> &dyn std::any::Any;
}