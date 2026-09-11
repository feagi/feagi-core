use crate::feagi_interfaces::feagi_connection_enums::FeagiInterfaceStatus;
use crate::feagi_interfaces::feagi_connector_interface_definition::FeagiConnectionInterfaceDefinition;
use feagi_data::feagi_data_error::FeagiDataError;

#[allow(dead_code)]
pub trait FeagiConnectorInterface {
    fn get_connection_status(&self) -> FeagiInterfaceStatus;

    fn attempt_start_connection_to_feagi(
        &mut self,
        connection_definition: Box<dyn FeagiConnectionInterfaceDefinition>,
    ) -> Result<(), FeagiDataError>;
}
