//! JSON5 output of a configured Zenoh node.

#![cfg(feature = "serde_json")]

use feagi_interop::transport::zenoh::zenoh_std::zenoh_configuration_builder::configuration_builder::ZenohSTDConfigurationBuilder;
use feagi_interop::transport::zenoh::zenoh_std::zenoh_configuration_builder::connect::ZenohTimeoutMs;
use feagi_interop::transport::zenoh::zenoh_std::zenoh_configuration_builder::error::ZenohConfigBuildError;
use feagi_interop::transport::zenoh::zenoh_std::zenoh_configuration_builder::id_generation_mode::ZenohIDGenerationMode;
use feagi_interop::transport::zenoh::zenoh_std::zenoh_configuration_builder::namespace::ZenohNamespace;

#[test]
fn connect_timeout_rejection_keeps_the_previous_value() {
    let mut node = ZenohSTDConfigurationBuilder::default().into_client();
    let error = node.set_connect_timeout(ZenohTimeoutMs::Millis(0)).expect_err("zero milliseconds");
    assert_eq!(error, ZenohConfigBuildError::ZeroMillisecondTimeout);
    assert_eq!(node.connect().timeout_ms, ZenohTimeoutMs::NoRetry);
}

#[test]
fn client_json5_encodes_the_chosen_id_namespace_and_client_connect() {
    let mut builder = ZenohSTDConfigurationBuilder::default();
    builder
        .set_id_mode(ZenohIDGenerationMode::Specific(0x10ab))
        .set_namespace(ZenohNamespace::new("brain/1").expect("namespace"))
        .expect("set namespace");
    let json = builder.into_client().build_json5().expect("json5");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse");
    assert_eq!(value["id"], "10ab");
    assert_eq!(value["mode"], "client");
    assert_eq!(value["namespace"], "brain/1");
    assert_eq!(value["queries_default_timeout"], 10_000);
    assert_eq!(value["connect"]["timeout_ms"], 0);
    assert_eq!(value["connect"]["exit_on_failure"], true);
    assert_eq!(value["connect"]["endpoints"], serde_json::json!([]));
    assert_eq!(value["listen"]["endpoints"], serde_json::json!([]));
    assert_eq!(value["transport"]["link"]["tx"]["sequence_number_resolution"], "32bit");
    assert_eq!(value["transport"]["unicast"]["qos"]["enabled"], true);
}

#[cfg(feature = "std")]
#[test]
fn peer_json5_waits_forever_and_listens_on_an_ephemeral_port() {
    let json = ZenohSTDConfigurationBuilder::default().into_peer().build_json5().expect("json5");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse");
    assert!(value.get("id").is_none());
    assert_eq!(value["mode"], "peer");
    assert_eq!(value["connect"]["timeout_ms"], -1);
    assert_eq!(value["connect"]["exit_on_failure"], false);
    assert_eq!(value["listen"]["endpoints"][0], "tcp/[::]:0");
}
