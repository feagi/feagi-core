//! WebSocket transport integration tests
//!
//! Tests the WebSocket transport implementations end-to-end.

#[cfg(all(feature = "websocket-server", feature = "websocket-client"))]
mod websocket_tests {
    use feagi_transports::prelude::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_pub_sub_pattern() {
        // Start publisher
        let mut publisher = WsPub::with_address("127.0.0.1:40001").await.unwrap();
        publisher.start_async().await.unwrap();
        
        // Give server time to start
        sleep(Duration::from_millis(100)).await;
        
        // Start subscriber
        let mut subscriber = WsSub::with_address("ws://127.0.0.1:40001").await.unwrap();
        subscriber.start_async().await.unwrap();
        subscriber.subscribe(b"test").unwrap();
        
        // Give connection time to establish
        sleep(Duration::from_millis(100)).await;
        
        // Publish a message
        publisher.publish(b"test", b"Hello WebSocket!").unwrap();
        
        // Receive the message
        let (topic, data) = subscriber.receive_timeout(2000).unwrap();
        assert_eq!(topic, b"test");
        assert_eq!(data, b"Hello WebSocket!");
        
        // Cleanup
        publisher.stop().unwrap();
        subscriber.stop().unwrap();
    }

    #[tokio::test]
    async fn test_push_pull_pattern() {
        // Start pull server
        let mut pull = WsPull::with_address("127.0.0.1:40002").await.unwrap();
        pull.start_async().await.unwrap();
        
        // Give server time to start
        sleep(Duration::from_millis(100)).await;
        
        // Start push client
        let mut push = WsPush::with_address("ws://127.0.0.1:40002").await.unwrap();
        push.start_async().await.unwrap();
        
        // Give connection time to establish
        sleep(Duration::from_millis(100)).await;
        
        // Push a message
        push.push_async(b"Sensory data").await.unwrap();
        
        // Pull the message
        let data = pull.pull_timeout(2000).unwrap();
        assert_eq!(data, b"Sensory data");
        
        // Cleanup
        push.stop().unwrap();
        pull.stop().unwrap();
    }

    // Router/Dealer test would require Clone trait on WsRouter or Arc wrapping
    // Skipping for now - the pattern is functional as shown in examples

    #[tokio::test]
    async fn test_multiple_subscribers() {
        // Start publisher
        let mut publisher = WsPub::with_address("127.0.0.1:40004").await.unwrap();
        publisher.start_async().await.unwrap();
        
        sleep(Duration::from_millis(100)).await;
        
        // Start multiple subscribers
        let mut sub1 = WsSub::with_address("ws://127.0.0.1:40004").await.unwrap();
        sub1.start_async().await.unwrap();
        sub1.subscribe(b"broadcast").unwrap();
        
        let mut sub2 = WsSub::with_address("ws://127.0.0.1:40004").await.unwrap();
        sub2.start_async().await.unwrap();
        sub2.subscribe(b"broadcast").unwrap();
        
        sleep(Duration::from_millis(100)).await;
        
        // Publish a message
        publisher.publish(b"broadcast", b"Message to all").unwrap();
        
        // Both subscribers should receive it
        let (topic1, data1) = sub1.receive_timeout(2000).unwrap();
        assert_eq!(topic1, b"broadcast");
        assert_eq!(data1, b"Message to all");
        
        let (topic2, data2) = sub2.receive_timeout(2000).unwrap();
        assert_eq!(topic2, b"broadcast");
        assert_eq!(data2, b"Message to all");
        
        // Cleanup
        publisher.stop().unwrap();
        sub1.stop().unwrap();
        sub2.stop().unwrap();
    }

    #[tokio::test]
    async fn test_topic_filtering() {
        // Start publisher
        let mut publisher = WsPub::with_address("127.0.0.1:40005").await.unwrap();
        publisher.start_async().await.unwrap();
        
        sleep(Duration::from_millis(100)).await;
        
        // Subscriber only wants "topicA"
        let mut subscriber = WsSub::with_address("ws://127.0.0.1:40005").await.unwrap();
        subscriber.start_async().await.unwrap();
        subscriber.subscribe(b"topicA").unwrap();
        
        sleep(Duration::from_millis(100)).await;
        
        // Publish to both topics
        publisher.publish(b"topicA", b"Message A").unwrap();
        publisher.publish(b"topicB", b"Message B").unwrap();
        
        // Should only receive topicA
        let (topic, data) = subscriber.receive_timeout(2000).unwrap();
        assert_eq!(topic, b"topicA");
        assert_eq!(data, b"Message A");
        
        // topicB should not be received (timeout expected)
        let result = subscriber.receive_timeout(500);
        assert!(matches!(result, Err(TransportError::Timeout)));
        
        // Cleanup
        publisher.stop().unwrap();
        subscriber.stop().unwrap();
    }
}

