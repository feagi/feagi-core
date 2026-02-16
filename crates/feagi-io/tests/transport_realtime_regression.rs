//! Integration regressions for real-time transport behavior.
//!
//! These tests run real client/server sockets (no mocks) to guard against
//! startup backlog replay and malformed-frame interference regressions.

#![cfg(feature = "zmq-transport")]

use std::net::TcpListener;
use std::env;
use std::thread;
use std::time::{Duration, Instant};

use feagi_io::protocol_implementations::zmq::{
    FeagiZmqClientPusherProperties, FeagiZmqServerPullerProperties,
};
use feagi_io::traits_and_enums::client::FeagiClientPusherProperties;
use feagi_io::traits_and_enums::server::FeagiServerPullerProperties;
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_serialization::FeagiByteContainer;
use feagi_structures::FeagiJSON;

fn reserve_free_tcp_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to reserve free TCP port");
    listener
        .local_addr()
        .expect("Failed to read local socket address")
        .port()
}

fn wait_until(timeout: Duration, mut predicate: impl FnMut() -> bool) {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if predicate() {
            return;
        }
        thread::sleep(Duration::from_millis(1));
    }
    panic!("Timed out waiting for condition");
}

fn make_valid_frame_with_marker(marker: u8) -> Vec<u8> {
    let mut container = FeagiByteContainer::new_empty();
    let payload = FeagiJSON::from_json_value(serde_json::json!({
        "kind": "sensory",
        "marker": marker
    }));
    container
        .overwrite_byte_data_with_single_struct_data(&payload, marker as u16)
        .expect("Failed to build non-empty FEAGI frame");
    container.get_byte_ref().to_vec()
}

fn make_valid_frame_with_counter(counter: u16) -> Vec<u8> {
    let mut container = FeagiByteContainer::new_empty();
    let payload = FeagiJSON::from_json_value(serde_json::json!({
        "kind": "sensory",
        "counter": counter
    }));
    container
        .overwrite_byte_data_with_single_struct_data(&payload, counter)
        .expect("Failed to build non-empty FEAGI frame");
    container.get_byte_ref().to_vec()
}

fn counter_from_frame(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[1], bytes[2]])
}

fn make_empty_valid_container() -> Vec<u8> {
    FeagiByteContainer::new_empty().get_byte_ref().to_vec()
}

fn read_soak_secs_from_env_or_default(default_secs: u64) -> u64 {
    match env::var("FEAGI_IO_SOAK_SECS") {
        Ok(raw) => raw.parse::<u64>().unwrap_or(default_secs).max(1),
        Err(_) => default_secs,
    }
}

#[test]
fn zmq_puller_keeps_latest_valid_frame_in_burst_with_noise() {
    let port = reserve_free_tcp_port();
    let endpoint = format!("tcp://127.0.0.1:{port}");

    let server_props = FeagiZmqServerPullerProperties::new(&endpoint, &endpoint)
        .expect("Failed to create ZMQ server puller properties");
    let mut server = server_props.as_boxed_server_puller();
    server.request_start().expect("Failed to start server puller");

    let client_props = FeagiZmqClientPusherProperties::new(&endpoint)
        .expect("Failed to create ZMQ client pusher properties");
    let mut client = client_props.as_boxed_client_pusher();
    client
        .request_connect()
        .expect("Failed to request client connect");

    wait_until(Duration::from_secs(2), || {
        matches!(
            client.poll(),
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData
        )
    });

    // Give ZMQ connect handshake a short moment before burst.
    thread::sleep(Duration::from_millis(50));

    let mut last_sent_valid_marker: Option<u8> = None;
    for marker in 0u8..80u8 {
        let valid = make_valid_frame_with_marker(marker);
        if client.publish_data(&valid).is_ok() {
            last_sent_valid_marker = Some(marker);
        }
        // Interleave malformed short frames to emulate transport noise.
        let _ = client.publish_data(&[0xAA, 0xBB, marker]);
    }

    let expected_marker = last_sent_valid_marker.expect("No valid frame was sent");

    wait_until(Duration::from_secs(2), || {
        matches!(server.poll(), FeagiEndpointState::ActiveHasData)
    });

    let consumed = server
        .consume_retrieved_data()
        .expect("Server failed to consume retrieved data");

    assert!(
        consumed.len() >= 12,
        "Expected FEAGI frame length >= 12, got {} bytes",
        consumed.len()
    );
    assert_eq!(
        consumed[1], expected_marker,
        "Expected latest valid marker {}, got {}",
        expected_marker, consumed[1]
    );
}

#[test]
fn zmq_stream_stays_responsive_and_fresh_under_sustained_noise() {
    let port = reserve_free_tcp_port();
    let endpoint = format!("tcp://127.0.0.1:{port}");

    let server_props = FeagiZmqServerPullerProperties::new(&endpoint, &endpoint)
        .expect("Failed to create ZMQ server puller properties");
    let mut server = server_props.as_boxed_server_puller();
    server.request_start().expect("Failed to start server puller");

    let client_props = FeagiZmqClientPusherProperties::new(&endpoint)
        .expect("Failed to create ZMQ client pusher properties");
    let mut client = client_props.as_boxed_client_pusher();
    client
        .request_connect()
        .expect("Failed to request client connect");

    wait_until(Duration::from_secs(2), || {
        matches!(
            client.poll(),
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData
        )
    });

    // Allow connection setup to settle before sustained send.
    thread::sleep(Duration::from_millis(50));

    let sender = thread::spawn(move || {
        let start = Instant::now();
        let run_for = Duration::from_millis(900);
        let mut counter: u16 = 0;
        while start.elapsed() < run_for {
            let valid = make_valid_frame_with_counter(counter);
            let _ = client.publish_data(&valid);

            // Interleave malformed short payloads (noise) frequently.
            if counter % 3 == 0 {
                let _ = client.publish_data(&[0xEE, 0x01, (counter & 0x00FF) as u8]);
            }

            counter = counter.wrapping_add(1);
            thread::sleep(Duration::from_millis(2));
        }
        counter.wrapping_sub(1)
    });

    let recv_start = Instant::now();
    let observe_for = Duration::from_millis(1200);
    let mut first_valid_at: Option<Instant> = None;
    let mut last_valid_at: Option<Instant> = None;
    let mut max_inter_frame_gap = Duration::ZERO;
    let mut valid_frame_count: usize = 0;
    let mut last_received_counter: Option<u16> = None;

    while recv_start.elapsed() < observe_for {
        match server.poll() {
            FeagiEndpointState::ActiveHasData => {
                let data = server
                    .consume_retrieved_data()
                    .expect("Server failed to consume retrieved data");
                if data.len() >= 12 {
                    let now = Instant::now();
                    if first_valid_at.is_none() {
                        first_valid_at = Some(now);
                    }
                    if let Some(prev) = last_valid_at {
                        let gap = now.saturating_duration_since(prev);
                        if gap > max_inter_frame_gap {
                            max_inter_frame_gap = gap;
                        }
                    }
                    last_valid_at = Some(now);
                    valid_frame_count += 1;
                    last_received_counter = Some(counter_from_frame(data));
                }
            }
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::Pending | FeagiEndpointState::Inactive => {
                thread::sleep(Duration::from_millis(1));
            }
            FeagiEndpointState::Errored(err) => {
                panic!("Server entered error state during sustained stream: {err}");
            }
        }
    }

    let last_sent_counter = sender.join().expect("Sender thread panicked");

    let first_valid_at = first_valid_at.expect("No valid sensory frame received");
    let startup_latency = first_valid_at.saturating_duration_since(recv_start);
    assert!(
        startup_latency <= Duration::from_millis(500),
        "Startup responsiveness regression: first valid frame took {:?}",
        startup_latency
    );

    assert!(
        valid_frame_count >= 20,
        "Expected sustained valid frame flow, got only {} valid frames",
        valid_frame_count
    );

    assert!(
        max_inter_frame_gap <= Duration::from_millis(250),
        "Detected large frame-gap/flicker window: {:?}",
        max_inter_frame_gap
    );

    let last_received_counter = last_received_counter.expect("No last received counter");
    let freshness_delta = last_sent_counter.saturating_sub(last_received_counter);
    assert!(
        freshness_delta <= 80,
        "Latest frame freshness regression: last_sent={} last_received={} delta={}",
        last_sent_counter,
        last_received_counter,
        freshness_delta
    );
}

#[test]
fn zmq_puller_prefers_non_empty_sensory_frame_over_empty_container_in_same_drain() {
    let port = reserve_free_tcp_port();
    let endpoint = format!("tcp://127.0.0.1:{port}");

    let server_props = FeagiZmqServerPullerProperties::new(&endpoint, &endpoint)
        .expect("Failed to create ZMQ server puller properties");
    let mut server = server_props.as_boxed_server_puller();
    server.request_start().expect("Failed to start server puller");

    let client_props = FeagiZmqClientPusherProperties::new(&endpoint)
        .expect("Failed to create ZMQ client pusher properties");
    let mut client = client_props.as_boxed_client_pusher();
    client
        .request_connect()
        .expect("Failed to request client connect");

    wait_until(Duration::from_secs(2), || {
        matches!(
            client.poll(),
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData
        )
    });

    thread::sleep(Duration::from_millis(50));

    let meaningful = make_valid_frame_with_counter(0x1337);
    let empty = make_empty_valid_container();

    // Same drain window: meaningful arrives, then empty frame arrives later.
    // Puller should keep meaningful frame to avoid visual flicker from empty overwrite.
    client
        .publish_data(&meaningful)
        .expect("Failed to send meaningful sensory frame");
    client
        .publish_data(&empty)
        .expect("Failed to send empty sensory frame");

    wait_until(Duration::from_secs(2), || {
        matches!(server.poll(), FeagiEndpointState::ActiveHasData)
    });

    let consumed = server
        .consume_retrieved_data()
        .expect("Server failed to consume retrieved data");
    assert!(
        consumed.len() > 12,
        "Expected non-empty sensory payload to be preferred, got empty container len={}",
        consumed.len()
    );
    assert_eq!(
        counter_from_frame(consumed),
        0x1337,
        "Expected meaningful sensory frame counter to be preserved"
    );
}

#[test]
fn zmq_stream_soak_detects_blackout_or_degradation_windows() {
    // Keep this in standard test flow. Allow override for deeper local soak runs.
    let soak_secs = read_soak_secs_from_env_or_default(12);
    let send_interval_ms = 4u64; // ~250Hz sender pace

    let port = reserve_free_tcp_port();
    let endpoint = format!("tcp://127.0.0.1:{port}");

    let server_props = FeagiZmqServerPullerProperties::new(&endpoint, &endpoint)
        .expect("Failed to create ZMQ server puller properties");
    let mut server = server_props.as_boxed_server_puller();
    server.request_start().expect("Failed to start server puller");

    let client_props = FeagiZmqClientPusherProperties::new(&endpoint)
        .expect("Failed to create ZMQ client pusher properties");
    let mut client = client_props.as_boxed_client_pusher();
    client
        .request_connect()
        .expect("Failed to request client connect");

    wait_until(Duration::from_secs(2), || {
        matches!(
            client.poll(),
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData
        )
    });
    thread::sleep(Duration::from_millis(50));

    let sender = thread::spawn(move || {
        let start = Instant::now();
        let run_for = Duration::from_secs(soak_secs);
        let mut counter: u16 = 0;
        while start.elapsed() < run_for {
            let valid = make_valid_frame_with_counter(counter);
            let _ = client.publish_data(&valid);

            // Mixed noise profile to emulate real startup/runtime artifacts.
            if counter % 4 == 0 {
                let _ = client.publish_data(&[0xAB, 0xCD]); // malformed short
            }
            if counter % 7 == 0 {
                let empty = make_empty_valid_container();
                let _ = client.publish_data(&empty); // valid but empty
            }

            counter = counter.wrapping_add(1);
            thread::sleep(Duration::from_millis(send_interval_ms));
        }
        counter.wrapping_sub(1)
    });

    let recv_start = Instant::now();
    let observe_for = Duration::from_secs(soak_secs + 2);
    let mut first_valid_at: Option<Instant> = None;
    let mut last_valid_at: Option<Instant> = None;
    let mut max_inter_frame_gap = Duration::ZERO;
    let mut valid_frame_count: usize = 0;
    let mut last_received_counter: Option<u16> = None;

    while recv_start.elapsed() < observe_for {
        match server.poll() {
            FeagiEndpointState::ActiveHasData => {
                let data = server
                    .consume_retrieved_data()
                    .expect("Server failed to consume retrieved data");
                if data.len() > 12 {
                    let now = Instant::now();
                    if first_valid_at.is_none() {
                        first_valid_at = Some(now);
                    }
                    if let Some(prev) = last_valid_at {
                        let gap = now.saturating_duration_since(prev);
                        if gap > max_inter_frame_gap {
                            max_inter_frame_gap = gap;
                        }
                    }
                    last_valid_at = Some(now);
                    valid_frame_count += 1;
                    last_received_counter = Some(counter_from_frame(data));
                }
            }
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::Pending | FeagiEndpointState::Inactive => {
                thread::sleep(Duration::from_millis(1));
            }
            FeagiEndpointState::Errored(err) => {
                panic!("Server entered error state during soak: {err}");
            }
        }
    }

    let last_sent_counter = sender.join().expect("Sender thread panicked");
    let first_valid_at = first_valid_at.expect("No non-empty valid sensory frame received during soak");

    let startup_latency = first_valid_at.saturating_duration_since(recv_start);
    assert!(
        startup_latency <= Duration::from_millis(700),
        "Startup responsiveness regression during soak: first non-empty frame took {:?}",
        startup_latency
    );

    let min_expected_frames = ((soak_secs * 1000) / 35) as usize; // ~28fps floor under noise
    assert!(
        valid_frame_count >= min_expected_frames,
        "Sustained throughput degraded: got {} valid frames, expected at least {} over {}s",
        valid_frame_count,
        min_expected_frames,
        soak_secs
    );

    assert!(
        max_inter_frame_gap <= Duration::from_millis(350),
        "Blackout/flicker regression: max inter-frame gap was {:?}",
        max_inter_frame_gap
    );

    let last_received_counter = last_received_counter.expect("No last received counter in soak");
    let freshness_delta = last_sent_counter.saturating_sub(last_received_counter);
    assert!(
        freshness_delta <= 140,
        "Freshness degraded during soak: last_sent={} last_received={} delta={}",
        last_sent_counter,
        last_received_counter,
        freshness_delta
    );
}

