//! The zenoh worker over websocket on localhost: announcing itself, and being reached.

#![cfg(all(feature = "std", feature = "transport_ws"))]

use std::net::SocketAddr;

use feagi_basis::parallelism::block_on_thread;
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_interop::node::feagi_node_flags::FeagiNodeFlags;
use feagi_interop::node::feagi_role::{FeagiRole, FeagiRoles};
use feagi_interop::transport::zenoh::zenoh_configuration::{ZenohConfigurationBuilder, ZenohConfigurationError};
use feagi_interop::transport::zenoh::zenoh_liveliness_path::{FeagiNodeLivelinessPath, FeagiNodeLivelinessPathConfigurables};
use feagi_interop::transport::zenoh::zenoh_worker_builder::ZenohWorkerBuilder;
use zenoh::Wait;

/// A localhost address nothing is listening on
fn free_address() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
    listener.local_addr().expect("ephemeral port address")
}

fn flags(role: FeagiRole) -> FeagiNodeFlags {
    let mut roles = FeagiRoles::new();
    roles.set_role(role, true);
    let mut flags = FeagiNodeFlags::new();
    flags.set_roles(roles);
    flags
}

/// A plain zenoh client connected to `address` over websocket, with scouting off
fn plain_client(address: SocketAddr) -> zenoh::Session {
    let mut config = zenoh::Config::default();
    config.insert_json5("mode", "\"client\"").unwrap();
    config.insert_json5("connect/endpoints", &format!("[\"ws/{address}\"]")).unwrap();
    config.insert_json5("scouting/multicast/enabled", "false").unwrap();
    zenoh::open(config).wait().expect("plain client connects to the worker")
}

/// Every liveliness key under `feagi` the session can see
fn liveliness_keys(session: &zenoh::Session) -> Vec<String> {
    let replies = session.liveliness().get("feagi/**").wait().expect("liveliness query");
    let mut keys = Vec::new();
    while let Ok(reply) = replies.recv() {
        if let Ok(sample) = reply.result() {
            keys.push(sample.key_expr().as_str().to_string());
        }
    }
    keys
}

#[test]
fn nothing_enabled_is_rejected() {
    let error = ZenohConfigurationBuilder::new().build().expect_err("nothing enabled");
    assert!(matches!(error, ZenohConfigurationError::NothingEnabled));
}

#[test]
fn server_announces_itself_and_clients_can_reach_it() {
    let address = free_address();

    let mut server_configuration = ZenohConfigurationBuilder::new();
    server_configuration.set_ws_server_bind(address);
    let server_builder = ZenohWorkerBuilder::new(server_configuration.build().unwrap(), flags(FeagiRole::FeagiBrainDesignerMode));
    let server_id = server_builder.feagi_node_id();
    let (mut server, ()) = server_builder.build_worker(&FeagiStdThreadSpawner, ()).expect("server worker starts");

    // A plain client sees the server's token, with the flags it was built with
    let observer = plain_client(address);
    let server_path = FeagiNodeLivelinessPathConfigurables::new(server_id);
    let keys = liveliness_keys(&observer);
    let server_flags: Vec<_> = keys.iter().filter_map(|key| FeagiNodeLivelinessPath::parse_key(&server_path, key)).collect();
    assert_eq!(server_flags, vec![flags(FeagiRole::FeagiBrainDesignerMode)], "keys seen: {keys:?}");

    // A client worker connects through the server, and its token is seen too
    let mut client_configuration = ZenohConfigurationBuilder::new();
    client_configuration.set_ws_client_target(address);
    let client_builder = ZenohWorkerBuilder::new(client_configuration.build().unwrap(), flags(FeagiRole::FeagiViewerEditor));
    let client_path = FeagiNodeLivelinessPathConfigurables::new(client_builder.feagi_node_id());
    let (mut client, ()) = client_builder.build_worker(&FeagiStdThreadSpawner, ()).expect("client worker starts");
    let keys = liveliness_keys(&observer);
    assert!(
        keys.iter().any(|key| FeagiNodeLivelinessPath::parse_key(&client_path, key).is_some()),
        "keys seen: {keys:?}"
    );

    // Stopping the client withdraws its token
    block_on_thread(client.stop()).expect("client worker stops");
    let keys = liveliness_keys(&observer);
    assert!(
        !keys.iter().any(|key| FeagiNodeLivelinessPath::parse_key(&client_path, key).is_some()),
        "keys seen: {keys:?}"
    );

    drop(observer);
    block_on_thread(server.stop()).expect("server worker stops");
    server.join().expect("server worker exits");
    client.join().expect("client worker exits");
}

#[test]
fn client_without_a_server_fails_to_start() {
    let mut configuration = ZenohConfigurationBuilder::new();
    configuration.set_ws_client_target(free_address());
    let result = ZenohWorkerBuilder::new(configuration.build().unwrap(), FeagiNodeFlags::new()).build_worker(&FeagiStdThreadSpawner, ());
    assert!(result.is_err());
}
