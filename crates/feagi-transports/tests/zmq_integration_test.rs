// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for ZMQ transport client-server pairs

use feagi_transports::prelude::*;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

/// Test ROUTER-DEALER request-reply pattern
#[test]
fn test_router_dealer_roundtrip() {
    // Synchronization barrier
    let barrier = Arc::new(Barrier::new(2));
    let barrier_server = Arc::clone(&barrier);
    let barrier_client = Arc::clone(&barrier);

    // Server thread
    let server_handle = thread::spawn(move || {
        let context = Arc::new(zmq::Context::new());
        let config = ServerConfig::new("tcp://127.0.0.1:31000");
        let mut server = ZmqRouter::new(context, config).unwrap();
        server.start().unwrap();

        // Signal ready
        barrier_server.wait();

        // Receive request
        let (request, reply_handle) = server.receive_timeout(5000).unwrap();
        assert_eq!(request, b"ping");

        // Send reply
        reply_handle.send(b"pong").unwrap();

        server.stop().unwrap();
    });

    // Client thread
    let client_handle = thread::spawn(move || {
        let context = Arc::new(zmq::Context::new());
        let config = ClientConfig::new("tcp://127.0.0.1:31000");
        let mut client = ZmqDealer::new(context, config).unwrap();
        client.start().unwrap();

        // Wait for server to be ready
        barrier_client.wait();
        thread::sleep(Duration::from_millis(100)); // ZMQ connection setup time

        // Send request
        let response = client.request_timeout(b"ping", 5000).unwrap();
        assert_eq!(response, b"pong");

        client.stop().unwrap();
    });

    server_handle.join().unwrap();
    client_handle.join().unwrap();
}

/// Test PUB-SUB publish-subscribe pattern
#[test]
fn test_pub_sub() {
    let barrier = Arc::new(Barrier::new(2));
    let barrier_server = Arc::clone(&barrier);
    let barrier_client = Arc::clone(&barrier);

    // Publisher thread
    let publisher_handle = thread::spawn(move || {
        let context = Arc::new(zmq::Context::new());
        let config = ServerConfig::new("tcp://127.0.0.1:31001");
        let mut publisher = ZmqPub::new(context, config).unwrap();
        publisher.start().unwrap();

        // Signal ready
        barrier_server.wait();

        // Wait for subscriber to connect and subscribe
        thread::sleep(Duration::from_millis(200));

        // Publish messages
        for i in 0..5 {
            let message = format!("message_{}", i);
            publisher.publish(b"test_topic", message.as_bytes()).unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        thread::sleep(Duration::from_millis(100));
        publisher.stop().unwrap();
    });

    // Subscriber thread
    let subscriber_handle = thread::spawn(move || {
        let context = Arc::new(zmq::Context::new());
        let config = ClientConfig::new("tcp://127.0.0.1:31001");
        let mut subscriber = ZmqSub::new(context, config).unwrap();
        subscriber.start().unwrap();
        subscriber.subscribe(b"test_topic").unwrap();

        // Signal ready
        barrier_client.wait();

        // Receive messages
        let mut received = Vec::new();
        for _ in 0..5 {
            if let Ok((topic, data)) = subscriber.receive_timeout(2000) {
                assert_eq!(topic, b"test_topic");
                received.push(String::from_utf8(data).unwrap());
            }
        }

        assert!(received.contains(&"message_0".to_string()));
        assert!(received.contains(&"message_4".to_string()));

        subscriber.stop().unwrap();
    });

    publisher_handle.join().unwrap();
    subscriber_handle.join().unwrap();
}

/// Test PULL-PUSH pattern
#[test]
fn test_pull_push() {
    let barrier = Arc::new(Barrier::new(2));
    let barrier_server = Arc::clone(&barrier);
    let barrier_client = Arc::clone(&barrier);

    // PULL (receiver) thread
    let pull_handle = thread::spawn(move || {
        let context = Arc::new(zmq::Context::new());
        let config = ServerConfig::new("tcp://127.0.0.1:31002");
        let mut pull = ZmqPull::new(context, config).unwrap();
        pull.start().unwrap();

        // Signal ready
        barrier_server.wait();

        // Receive messages
        let mut received = Vec::new();
        for _ in 0..3 {
            if let Ok(data) = pull.pull_timeout(2000) {
                received.push(String::from_utf8(data).unwrap());
            }
        }

        assert!(received.contains(&"data_0".to_string()));
        assert!(received.contains(&"data_1".to_string()));
        assert!(received.contains(&"data_2".to_string()));

        pull.stop().unwrap();
    });

    // PUSH (sender) thread
    let push_handle = thread::spawn(move || {
        let context = Arc::new(zmq::Context::new());
        let config = ClientConfig::new("tcp://127.0.0.1:31002");
        let mut push = ZmqPush::new(context, config).unwrap();
        push.start().unwrap();

        // Wait for PULL to be ready
        barrier_client.wait();
        thread::sleep(Duration::from_millis(100));

        // Push messages
        for i in 0..3 {
            let message = format!("data_{}", i);
            push.push(message.as_bytes()).unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        thread::sleep(Duration::from_millis(100));
        push.stop().unwrap();
    });

    pull_handle.join().unwrap();
    push_handle.join().unwrap();
}

/// Test error handling - timeout
#[test]
fn test_timeout() {
    let _context = Arc::new(zmq::Context::new());
    let config = ServerConfig::new("tcp://127.0.0.1:31003");
    let mut server = ZmqRouter::with_address("tcp://127.0.0.1:31003").unwrap();
    server.start().unwrap();

    // Try to receive with timeout (should timeout since no client is sending)
    let result = server.receive_timeout(100);
    assert!(result.is_err());
    
    if let Err(e) = result {
        assert!(matches!(e, TransportError::Timeout));
    }

    server.stop().unwrap();
}

/// Test invalid config
#[test]
fn test_invalid_config() {
    let context = Arc::new(zmq::Context::new());
    let mut config = ServerConfig::new("");
    config.base.max_message_size = Some(0);

    let result = config.base.validate();
    assert!(result.is_err());
}

