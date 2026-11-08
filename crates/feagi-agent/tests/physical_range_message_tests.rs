//! A `physical_range` inside `AgentConfigurationDetails` goes through the same decode the server
//! uses for every command/control message (`FeagiMessage::try_from(&FeagiByteContainer)`).
//!
//! An invalid range must make that decode fail; a valid range must survive the decode and the
//! re-serialization the server stores (`serde_json::to_value(device_def)`).

use feagi_agent::command_and_control::agent_embodiment_configuration_message::AgentEmbodimentConfigurationMessage;
use feagi_agent::command_and_control::FeagiMessage;
use feagi_agent::FeagiAgentError;
use feagi_sensorimotor::configuration::jsonable::JSONInputOutputDefinition;
use feagi_sensorimotor::ConnectorCache;
use feagi_serialization::FeagiByteContainer;
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalChannelCount, CorticalUnitIndex, NeuronDepth,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::FeagiJSON;
use serde_json::{json, Value};

/// Registration JSON of one two-channel sensory unit, exported by the real connector cache.
fn exported_registration() -> Value {
    let cache = ConnectorCache::new();
    cache
        .get_sensor_cache()
        .infrared_register(
            CorticalUnitIndex::from(0u16),
            CorticalChannelCount::new(2).unwrap(),
            FrameChangeHandling::Absolute,
            NeuronDepth::new(8).unwrap(),
            PercentageNeuronPositioning::Linear,
        )
        .unwrap();
    cache.export_device_registrations_as_config_json().unwrap()
}

/// First channel of the first sensory unit.
fn first_device(registration: &mut Value) -> &mut Value {
    let units = registration["input_units_and_encoder_properties"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap();
    &mut units[0][0]["device_grouping"][0]
}

/// The wire form of `AgentConfigurationDetails(registration)` as a message JSON value.
fn message_json(registration: Value) -> Value {
    let definition: JSONInputOutputDefinition = serde_json::from_value(registration).unwrap();
    serde_json::to_value(FeagiMessage::AgentConfiguration(
        AgentEmbodimentConfigurationMessage::AgentConfigurationDetails(definition),
    ))
    .unwrap()
}

/// Server-side decode of a message JSON value, exactly as received bytes are decoded.
fn server_decode(message: &Value) -> Result<FeagiMessage, FeagiAgentError> {
    let mut container = FeagiByteContainer::new_empty();
    container
        .overwrite_byte_data_with_single_struct_data(
            &FeagiJSON::from_json_value(message.clone()),
            0,
        )
        .unwrap();
    FeagiMessage::try_from(&container)
}

#[test]
fn valid_range_survives_server_decode_and_stored_reserialization() {
    let mut registration = exported_registration();
    first_device(&mut registration)["physical_range"] =
        json!({"value_at_0": -1.5, "value_at_1": 1.5, "unit": "rad"});

    let decoded = server_decode(&message_json(registration)).expect("valid range must decode");
    let FeagiMessage::AgentConfiguration(
        AgentEmbodimentConfigurationMessage::AgentConfigurationDetails(device_def),
    ) = decoded
    else {
        panic!("expected AgentConfigurationDetails");
    };

    // The server stores serde_json::to_value(device_def) (feagi_agent_handler.rs).
    let mut stored = serde_json::to_value(device_def).unwrap();
    assert_eq!(
        first_device(&mut stored)["physical_range"],
        json!({"value_at_0": -1.5, "value_at_1": 1.5, "unit": "rad"})
    );
    // The second channel declared none, so none is stored.
    let units = stored["input_units_and_encoder_properties"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    assert!(units[0][0]["device_grouping"][1]
        .get("physical_range")
        .is_none());
}

#[test]
fn invalid_range_makes_the_server_decode_fail() {
    for bad in [
        json!({"value_at_0": 2.0, "value_at_1": 2.0, "unit": "rad"}),
        json!({"value_at_0": 0.0, "value_at_1": 1.0, "unit": ""}),
        json!({"value_at_0": 0.0, "value_at_1": 1.0}),
    ] {
        // Build the message with a valid range, then corrupt it on the wire form.
        let mut registration = exported_registration();
        first_device(&mut registration)["physical_range"] =
            json!({"value_at_0": 0.0, "value_at_1": 1.0, "unit": "rad"});
        let mut message = message_json(registration);
        *first_device(&mut message["agent_configuration"]["AgentConfigurationDetails"])
            .get_mut("physical_range")
            .unwrap() = bad.clone();

        let err = server_decode(&message).expect_err("invalid range must not decode");
        assert!(
            matches!(err, FeagiAgentError::UnableToDecodeReceivedData(_)),
            "{bad}: {err:?}"
        );
    }
}
