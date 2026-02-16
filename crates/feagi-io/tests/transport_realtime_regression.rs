//! Integration regressions for real-time transport behavior.
//!
//! These tests run real client/server sockets (no mocks) to guard against
//! startup backlog replay and malformed-frame interference regressions.

#![cfg(feature = "zmq-transport")]

use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use feagi_io::protocol_implementations::zmq::{
    FeagiZmqClientPusherProperties, FeagiZmqServerPullerProperties,
};
use feagi_io::traits_and_enums::client::FeagiClientPusherProperties;
use feagi_io::traits_and_enums::server::FeagiServerPullerProperties;
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_serialization::FeagiByteContainer;

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
    let container = FeagiByteContainer::new_empty();
    let mut bytes = container.get_byte_ref().to_vec();
    // Header layout: [version, increment_lo, increment_hi, struct_count]
    // Keep it valid, but encode marker in increment counter for assertions.
    bytes[1] = marker;
    bytes
}

fn make_valid_frame_with_counter(counter: u16) -> Vec<u8> {
    let container = FeagiByteContainer::new_empty();
    let mut bytes = container.get_byte_ref().to_vec();
    bytes[1] = (counter & 0x00FF) as u8;
    bytes[2] = ((counter >> 8) & 0x00FF) as u8;
    bytes
}

fn counter_from_frame(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[1], bytes[2]])
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

