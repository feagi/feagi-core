use crate::feagi_interfaces::feagi_connection_enums::FeagiInterfaceStatus;
use crate::feagi_interfaces::feagi_connector_interface_definition::FeagiConnectionInterfaceDefinition;
use feagi_data_structures::FeagiDataError;

pub trait FeagiConnectorInterface {
    fn get_connection_status(&self) -> FeagiInterfaceStatus;

    fn attempt_start_connection_to_feagi(
        &mut self,
        connection_definition: Box<dyn FeagiConnectionInterfaceDefinition>,
    ) -> Result<(), FeagiDataError>;
}
