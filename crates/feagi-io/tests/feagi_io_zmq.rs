// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for feagi-io ZMQ transports.

use feagi_io::io_api::implementations::zmq::{
    FEAGIZMQClientPusher, FEAGIZMQClientRequester, FEAGIZMQClientSubscriber,
    FEAGIZMQServerPuller, FEAGIZMQServerPublisher, FEAGIZMQServerRouter,
};
use feagi_io::io_api::traits_and_enums::client::{
    FeagiClient, FeagiClientPusher, FeagiClientRequester,
};
use feagi_io::io_api::traits_and_enums::server::{
    FeagiServer, FeagiServerPuller, FeagiServerPublisher, FeagiServerRouter,
};
use feagi_io::transports::core::common::config::{ClientConfig, ServerConfig, TransportConfig};
use feagi_io::transports::core::common::error::TransportError;
use feagi_io::transports::core::traits::{
    Publisher, Pull, Push, RequestReplyClient, RequestReplyServer, Subscriber, Transport,
};
use feagi_io::transports::core::zmq::client::{dealer::ZmqDealer, push::ZmqPush, sub::ZmqSub};
use feagi_io::transports::core::zmq::server::{ZmqPub, ZmqPull, ZmqRouter};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

/// Read the ZMQ host for tests from environment variables.
fn test_zmq_host() -> String {
    std::env::var("FEAGI_TEST_ZMQ_HOST").expect(
        "FEAGI_TEST_ZMQ_HOST is required (set to a connectable host, e.g. 127.0.0.1)",
    )
}

fn format_tcp_endpoint(host: &str, port: u16) -> String {
    if host.contains(':') {
        format!("tcp://[{host}]:{port}")
    } else {
        format!("tcp://{host}:{port}")
    }
}

fn bind_address(host: &str) -> String {
    if host.contains(':') {
        format!("[{host}]:0")
    } else {
        format!("{host}:0")
    }
}

/// Build a unique TCP endpoint for test isolation.
fn tcp_endpoint() -> String {
    let host = test_zmq_host();
    let listener = TcpListener::bind(bind_address(&host)).unwrap_or_else(|e| {
        panic!("Failed to reserve a TCP port for ZMQ tests on host {host}: {e}");
    });
    let port = listener
        .local_addr()
        .unwrap_or_else(|e| panic!("Failed to read reserved port for ZMQ tests: {e}"))
        .port();
    drop(listener);
    format_tcp_endpoint(&host, port)
}

/// Read the default transport timeout for consistent test waits.
fn default_timeout() -> Duration {
    TransportConfig::default()
        .timeout
        .expect("TransportConfig::default must include a timeout for tests")
}

/// Derive a polling interval from a timeout window.
fn poll_interval(timeout: Duration) -> Duration {
    let millis = timeout.as_millis().max(1);
    let slice = (millis / 50).max(1) as u64;
    Duration::from_millis(slice)
}

/// Poll until data is available or timeout elapses.
fn wait_for<T, F>(timeout: Duration, mut poll: F) -> T
where
    F: FnMut() -> Option<T>,
{
    let deadline = Instant::now() + timeout;
    let interval = poll_interval(timeout);
    loop {
        if let Some(value) = poll() {
            return value;
        }
        if Instant::now() >= deadline {
            panic!("Timed out waiting for ZMQ message");
        }
        std::thread::sleep(interval);
    }
}

/// Validate core PUB/SUB roundtrip over ZMQ.
#[test]
fn core_zmq_pub_sub_roundtrip() {
    let runtime = Arc::new(Runtime::new().expect("Failed to create runtime"));
    let endpoint = tcp_endpoint();
    let timeout = default_timeout();
    let poll_timeout_ms = (timeout.as_millis().max(1) / 10).max(1) as u64;

    let mut publisher =
        ZmqPub::new(runtime.clone(), ServerConfig::new(endpoint.clone()))
            .expect("Failed to create ZmqPub");
    publisher.start().expect("Failed to start ZmqPub");

    let mut subscriber =
        ZmqSub::new(runtime.clone(), ClientConfig::new(endpoint.clone()))
            .expect("Failed to create ZmqSub");
    subscriber.start().expect("Failed to start ZmqSub");
    subscriber
        .subscribe(b"topic")
        .expect("Failed to subscribe");

    let payload = b"payload";
    publisher
        .publish(b"topic", payload)
        .expect("Failed to publish");

    let mut attempts = 0;
    let (recv_topic, recv_payload) = wait_for(timeout, || {
        match subscriber.receive_timeout(poll_timeout_ms) {
            Ok(value) => Some(value),
            Err(TransportError::Timeout) => {
                if attempts < 3 {
                    publisher
                        .publish(b"topic", payload)
                        .expect("Retry publish failed");
                }
                attempts += 1;
                None
            }
            Err(err) => panic!("Subscriber receive failed: {err}"),
        }
    });

    assert_eq!(recv_topic, b"topic");
    assert_eq!(recv_payload, payload);

    subscriber.stop().expect("Failed to stop ZmqSub");
    publisher.stop().expect("Failed to stop ZmqPub");
}

/// Validate core PUSH/PULL roundtrip over ZMQ.
#[test]
fn core_zmq_push_pull_roundtrip() {
    let runtime = Arc::new(Runtime::new().expect("Failed to create runtime"));
    let endpoint = tcp_endpoint();
    let timeout = default_timeout();
    let poll_timeout_ms = (timeout.as_millis().max(1) / 10).max(1) as u64;

    let mut puller =
        ZmqPull::new(runtime.clone(), ServerConfig::new(endpoint.clone()))
            .expect("Failed to create ZmqPull");
    puller.start().expect("Failed to start ZmqPull");

    let mut pusher =
        ZmqPush::new(runtime.clone(), ClientConfig::new(endpoint.clone()))
            .expect("Failed to create ZmqPush");
    pusher.start().expect("Failed to start ZmqPush");

    let payload = b"work-item";
    pusher.push(payload).expect("Failed to push payload");

    let mut attempts = 0;
    let recv_payload = wait_for(timeout, || {
        match puller.pull_timeout(poll_timeout_ms) {
            Ok(value) => Some(value),
            Err(TransportError::Timeout) => {
                if attempts < 3 {
                    pusher.push(payload).expect("Retry push failed");
                }
                attempts += 1;
                None
            }
            Err(err) => panic!("Puller receive failed: {err}"),
        }
    });

    assert_eq!(recv_payload, payload);

    pusher.stop().expect("Failed to stop ZmqPush");
    puller.stop().expect("Failed to stop ZmqPull");
}

/// Validate core ROUTER/DEALER request-reply over ZMQ.
#[test]
fn core_zmq_router_dealer_roundtrip() {
    let runtime = Arc::new(Runtime::new().expect("Failed to create runtime"));
    let endpoint = tcp_endpoint();
    let timeout = default_timeout();
    let timeout_ms = timeout.as_millis().max(1) as u64;

    let mut router =
        ZmqRouter::new(runtime.clone(), ServerConfig::new(endpoint.clone()))
            .expect("Failed to create ZmqRouter");
    router.start().expect("Failed to start ZmqRouter");

    let mut dealer =
        ZmqDealer::new(runtime.clone(), ClientConfig::new(endpoint.clone()))
            .expect("Failed to create ZmqDealer");
    dealer.start().expect("Failed to start ZmqDealer");

    let request = b"request";
    let response = b"response";

    let dealer_thread = std::thread::spawn(move || {
        let response = dealer
            .request_timeout(request, timeout_ms)
            .expect("Dealer request failed");
        dealer.stop().expect("Failed to stop ZmqDealer");
        response
    });

    let (received, reply_handle) = router
        .receive_timeout(timeout_ms)
        .expect("Router receive failed");
    assert_eq!(received, request);
    reply_handle
        .send(response)
        .expect("Router response failed");

    let dealer_response = dealer_thread
        .join()
        .expect("Dealer thread panicked");
    assert_eq!(dealer_response, response);

    router.stop().expect("Failed to stop ZmqRouter");
}

/// Validate io_api PUB/SUB roundtrip over ZMQ.
#[test]
fn io_api_zmq_pub_sub_roundtrip() {
    let endpoint = tcp_endpoint();
    let timeout = default_timeout();

    let mut publisher = FEAGIZMQServerPublisher::new(
        endpoint.clone(),
        Box::new(|_| {}),
    )
    .expect("Failed to create FEAGIZMQServerPublisher");
    publisher.start().expect("Failed to start publisher");

    let mut subscriber = FEAGIZMQClientSubscriber::new(
        endpoint.clone(),
        Box::new(|_| {}),
    )
    .expect("Failed to create FEAGIZMQClientSubscriber");
    subscriber
        .connect(&endpoint)
        .expect("Failed to connect subscriber");

    let payload = b"io-api-pub-sub";
    publisher
        .publish(payload)
        .expect("Failed to publish data");

    let mut attempts = 0;
    let recv_payload = wait_for(timeout, || {
        match subscriber.try_poll_receive() {
            Ok(Some(value)) => Some(value.to_vec()),
            Ok(None) => {
                if attempts < 3 {
                    publisher
                        .publish(payload)
                        .expect("Retry publish failed");
                }
                attempts += 1;
                None
            }
            Err(err) => panic!("Subscriber receive failed: {err}"),
        }
    });

    assert_eq!(recv_payload, payload);

    subscriber
        .disconnect()
        .expect("Failed to disconnect subscriber");
    publisher.stop().expect("Failed to stop publisher");
}

/// Validate io_api PUSH/PULL roundtrip over ZMQ.
#[test]
fn io_api_zmq_push_pull_roundtrip() {
    let endpoint = tcp_endpoint();
    let timeout = default_timeout();

    let mut puller = FEAGIZMQServerPuller::new(
        endpoint.clone(),
        Box::new(|_| {}),
    )
    .expect("Failed to create FEAGIZMQServerPuller");
    puller.start().expect("Failed to start puller");

    let mut pusher = FEAGIZMQClientPusher::new(
        endpoint.clone(),
        Box::new(|_| {}),
    )
    .expect("Failed to create FEAGIZMQClientPusher");
    pusher
        .connect(&endpoint)
        .expect("Failed to connect pusher");

    let payload = b"io-api-push-pull";
    pusher.push_data(payload);

    let mut attempts = 0;
    let recv_payload = wait_for(timeout, || {
        match puller.try_poll_receive() {
            Ok(Some(value)) => Some(value.to_vec()),
            Ok(None) => {
                if attempts < 3 {
                    pusher.push_data(payload);
                }
                attempts += 1;
                None
            }
            Err(err) => panic!("Puller receive failed: {err}"),
        }
    });

    assert_eq!(recv_payload, payload);

    pusher.disconnect().expect("Failed to disconnect pusher");
    puller.stop().expect("Failed to stop puller");
}

/// Validate io_api ROUTER/REQUESTER roundtrip over ZMQ.
#[test]
fn io_api_zmq_router_requester_roundtrip() {
    let endpoint = tcp_endpoint();
    let timeout = default_timeout();

    let mut router = FEAGIZMQServerRouter::new(
        endpoint.clone(),
        Box::new(|_| {}),
    )
    .expect("Failed to create FEAGIZMQServerRouter");
    router.start().expect("Failed to start router");

    let mut requester = FEAGIZMQClientRequester::new(
        endpoint.clone(),
        Box::new(|_| {}),
    )
    .expect("Failed to create FEAGIZMQClientRequester");
    requester
        .connect(&endpoint)
        .expect("Failed to connect requester");

    let request = b"io-api-request";
    let response = b"io-api-response";
    requester
        .send_request(request)
        .expect("Requester send failed");

    let (client_id, received) = wait_for(timeout, || match router.try_poll_receive() {
        Ok(Some((client_id, value))) => Some((client_id, value.to_vec())),
        Ok(None) => None,
        Err(err) => panic!("Router receive failed: {err}"),
    });
    assert_eq!(received, request);

    router
        .send_response(client_id, response)
        .expect("Router response failed");

    let recv_response = wait_for(timeout, || match requester.try_poll_receive() {
        Ok(Some(value)) => Some(value.to_vec()),
        Ok(None) => None,
        Err(err) => panic!("Requester receive failed: {err}"),
    });

    assert_eq!(recv_response, response);

    requester
        .disconnect()
        .expect("Failed to disconnect requester");
    router.stop().expect("Failed to stop router");
}
