use feagi_basis::FeagiBasisError;
use crate::legacy_code::sensio_motor::feagi_interfaces::feagi_connection_enums::FeagiInterfaceStatus;
use crate::legacy_code::sensio_motor::feagi_interfaces::feagi_connector_interface_definition::FeagiConnectionInterfaceDefinition;

#[allow(dead_code)]
pub trait FeagiConnectorInterface {
    fn get_connection_status(&self) -> FeagiInterfaceStatus;

    fn attempt_start_connection_to_feagi(
        &mut self,
        connection_definition: Box<dyn FeagiConnectionInterfaceDefinition>,
    ) -> Result<(), FeagiBasisError>;
}
