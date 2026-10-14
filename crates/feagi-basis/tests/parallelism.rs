#![cfg(feature = "std")]

use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll, Waker};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use feagi_basis::parallelism::block_on_thread;
use feagi_basis::parallelism::feagi_channel::channel_error::{FeagiChannelReceiveError, FeagiChannelSendError, FeagiRequestError};
use feagi_basis::parallelism::feagi_channel::feagi_channel::{FeagiChannelDynamicBuilder, FeagiChannelReceiver, FeagiChannelSender};
use feagi_basis::parallelism::feagi_channel::recycling::Recyclable;
use feagi_basis::parallelism::feagi_channel::request_response::feagi_request_response::FeagiRequestResponseDynamicBuilder;
use feagi_basis::parallelism::feagi_worker::feagi_worker::{FeagiWorkerHandleDynamic, FeagiWorkerNotificationsDynamic, FeagiWorkerSpawner};
use feagi_basis::parallelism::feagi_worker::notifications::{FeagiWorkerNotification, FeagiWorkerReply};
use feagi_basis::parallelism::feagi_worker::std_thread_spawner::FeagiStdThreadSpawner;

//region Helpers

/// Simple recyclable value for channel tests
#[derive(Debug, PartialEq)]
struct Number(u64);

impl Recyclable for Number {
    fn new_for_recycler() -> Self {
        Number(0)
    }
}

/// Recyclable buffer that records how often it was recycled, to observe slot reuse
#[derive(Debug, Default)]
struct Buffer {
    values: Vec<u32>,
    times_recycled: u32,
}

impl Recyclable for Buffer {
    fn new_for_recycler() -> Self {
        Buffer::default()
    }

    fn recycle_self(&mut self) {
        self.values.clear();
        self.times_recycled += 1;
    }
}

#[derive(Debug)]
struct TestError;

impl core::fmt::Display for TestError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "test error")
    }
}

impl core::error::Error for TestError {}

/// Polls `future` exactly once, then drops it. Used to cancel futures mid-way
fn poll_once<F: Future>(future: F) -> Poll<F::Output> {
    let mut future = pin!(future);
    future.as_mut().poll(&mut Context::from_waker(Waker::noop()))
}

/// Worker body that doubles custom notifications and stops on `Stop`
async fn doubling_worker(notifications: FeagiWorkerNotificationsDynamic<u32, u32>) {
    let mut running = true;
    while running {
        let result = notifications.async_process_next(async |notification| match notification {
            FeagiWorkerNotification::Stop => {
                running = false;
                FeagiWorkerReply::Acknowledged
            }
            FeagiWorkerNotification::Custom(value) => FeagiWorkerReply::Custom(value * 2),
            FeagiWorkerNotification::Nothing => FeagiWorkerReply::Nothing,
        }).await;
        if result.is_err() {
            break;
        }
    }
}

//endregion

//region Recycling

#[test]
fn recyclable_result_starts_ok_and_recycles_inner_value() {
    let mut result = <Result<Buffer, TestError> as Recyclable>::new_for_recycler();
    assert!(matches!(&result, Ok(buffer) if buffer.values.is_empty()));

    if let Ok(buffer) = &mut result {
        buffer.values.push(1);
    }
    result.recycle_self();
    assert!(matches!(&result, Ok(buffer) if buffer.values.is_empty() && buffer.times_recycled == 1));

    let mut error: Result<Buffer, TestError> = Err(TestError);
    error.recycle_self();
    assert!(error.is_err());
}

//endregion

//region Dynamic Channels

#[test]
fn dynamic_channel_try_send_and_try_recv_in_order() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Number>::new(4).create_channels();
    for value in 0..4 {
        sender.try_send(Number(value)).unwrap();
    }
    for value in 0..4 {
        assert_eq!(receiver.try_recv().unwrap(), Number(value));
    }
}

#[test]
fn dynamic_channel_helpers_track_queue_state() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Number>::new(2).create_channels();
    assert_eq!(sender.capacity(), 2);
    assert_eq!(receiver.capacity(), 2);
    assert!(sender.is_empty() && receiver.is_empty());
    assert!(!sender.is_full() && !receiver.is_full());
    assert_eq!(sender.remaining(), 2);

    sender.try_send(Number(1)).unwrap();
    assert_eq!(sender.len(), 1);
    assert_eq!(receiver.len(), 1);
    assert_eq!(receiver.remaining(), 1);
    assert!(!sender.is_empty());

    sender.try_send(Number(2)).unwrap();
    assert!(sender.is_full() && receiver.is_full());
    assert_eq!(sender.remaining(), 0);

    receiver.try_recv().unwrap();
    assert!(!sender.is_full());
    assert_eq!(receiver.len(), 1);
}

#[test]
fn dynamic_channel_try_send_errors_when_full() {
    let (sender, _receiver) = FeagiChannelDynamicBuilder::<Number>::new(1).create_channels();
    sender.try_send(Number(1)).unwrap();
    assert!(matches!(sender.try_send(Number(2)), Err(FeagiChannelSendError::ChannelFull)));
    assert!(matches!(sender.try_send_ref(), Err(FeagiChannelSendError::ChannelFull)));
}

#[test]
fn dynamic_channel_try_recv_errors_when_empty() {
    let (_sender, receiver) = FeagiChannelDynamicBuilder::<Number>::new(1).create_channels();
    assert!(matches!(receiver.try_recv(), Err(FeagiChannelReceiveError::ChannelEmpty)));
    assert!(matches!(receiver.try_recv_ref(), Err(FeagiChannelReceiveError::ChannelEmpty)));
}

#[test]
fn dynamic_channel_send_errors_once_receiver_dropped() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Number>::new(2).create_channels();
    drop(receiver);
    assert!(matches!(sender.try_send(Number(1)), Err(FeagiChannelSendError::ChannelClosed)));
    assert!(matches!(sender.try_send_ref(), Err(FeagiChannelSendError::ChannelClosed)));
    assert!(matches!(block_on_thread(sender.async_send(Number(1))), Err(FeagiChannelSendError::ChannelClosed)));
}

#[test]
fn dynamic_channel_drains_queue_before_reporting_closed() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Number>::new(2).create_channels();
    sender.try_send(Number(7)).unwrap();
    drop(sender);
    // TODO: Restore with `FeagiChannelReceiver::is_closed`, disabled due to a thingbuf 0.1.6 bug
    // assert!(receiver.is_closed());
    assert_eq!(receiver.try_recv().unwrap(), Number(7));
    assert!(matches!(receiver.try_recv(), Err(FeagiChannelReceiveError::ChannelClosed)));
    assert!(matches!(block_on_thread(receiver.async_recv()), Err(FeagiChannelReceiveError::ChannelClosed)));
}

#[test]
fn dynamic_channel_closes_only_after_every_cloned_sender_dropped() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Number>::new(2).create_channels();
    let second_sender = sender.clone();
    drop(sender);
    // TODO: Restore with `FeagiChannelReceiver::is_closed`, disabled due to a thingbuf 0.1.6 bug.
    // thingbuf reports closed here even though `second_sender` is alive
    // assert!(!receiver.is_closed());
    second_sender.try_send(Number(3)).unwrap();
    assert!(matches!(receiver.try_recv(), Ok(Number(3))));
    assert!(matches!(receiver.try_recv(), Err(FeagiChannelReceiveError::ChannelEmpty)));
    drop(second_sender);
    // assert!(receiver.is_closed());
    assert!(matches!(receiver.try_recv(), Err(FeagiChannelReceiveError::ChannelClosed)));
}

#[test]
fn dynamic_channel_async_send_and_recv_across_threads() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Number>::new(2).create_channels();
    let producers: Vec<_> = (0..4u64).map(|producer| {
        let sender = sender.clone();
        std::thread::spawn(move || block_on_thread(async {
            for value in 0..100u64 {
                sender.async_send(Number(producer * 1000 + value)).await.unwrap();
            }
        }))
    }).collect();
    drop(sender);

    let mut received = block_on_thread(async {
        let mut received = Vec::new();
        while let Ok(Number(value)) = receiver.async_recv().await {
            received.push(value);
        }
        received
    });
    for producer in producers {
        producer.join().unwrap();
    }

    received.sort_unstable();
    let expected: Vec<u64> = (0..4u64).flat_map(|producer| (0..100u64).map(move |value| producer * 1000 + value)).collect();
    assert_eq!(received, expected);
}

#[test]
fn dynamic_builder_creates_independent_channels() {
    let builder = FeagiChannelDynamicBuilder::<Number>::new(1);
    let (first_sender, first_receiver) = builder.create_channels();
    let (second_sender, second_receiver) = builder.create_channels();
    first_sender.try_send(Number(1)).unwrap();
    assert!(matches!(second_receiver.try_recv(), Err(FeagiChannelReceiveError::ChannelEmpty)));
    second_sender.try_send(Number(2)).unwrap();
    assert_eq!(first_receiver.try_recv().unwrap(), Number(1));
    assert_eq!(second_receiver.try_recv().unwrap(), Number(2));
}

//endregion

//region Send / Receive by Reference

#[test]
fn send_ref_and_recv_ref_reuse_the_slot_value() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Buffer>::new(1).create_channels();

    {
        let mut slot = sender.try_send_ref().unwrap();
        slot.values.extend_from_slice(&[1, 2, 3]);
    }
    {
        let slot = receiver.try_recv_ref().unwrap();
        assert_eq!(slot.values, vec![1, 2, 3]);
    }

    // The same slot comes back recycled, keeping its allocation
    let slot = sender.try_send_ref().unwrap();
    assert!(slot.values.is_empty());
    assert!(slot.values.capacity() >= 3);
    assert_eq!(slot.times_recycled, 1);
}

#[test]
fn by_value_recv_replaces_the_slot_value() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Buffer>::new(1).create_channels();
    {
        let mut slot = sender.try_send_ref().unwrap();
        slot.values.extend_from_slice(&[1, 2, 3]);
    }
    let received = receiver.try_recv().unwrap();
    assert_eq!(received.values, vec![1, 2, 3]);

    // The received value was moved out, so the slot holds a fresh value with no allocation
    let slot = sender.try_send_ref().unwrap();
    assert_eq!(slot.values.capacity(), 0);
}

#[test]
fn dropping_unwritten_send_ref_still_publishes_slot() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Buffer>::new(1).create_channels();
    drop(sender.try_send_ref().unwrap());
    let slot = receiver.try_recv_ref().unwrap();
    assert!(slot.values.is_empty());
}

#[test]
fn async_ref_methods_round_trip() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Buffer>::new(1).create_channels();
    let producer = std::thread::spawn(move || block_on_thread(async {
        for value in 0..10u32 {
            let mut slot = sender.async_send_ref().await.unwrap();
            slot.values.push(value);
        }
    }));
    let received = block_on_thread(async {
        let mut received = Vec::new();
        while let Ok(slot) = receiver.async_recv_ref().await {
            received.push(slot.values[0]);
        }
        received
    });
    producer.join().unwrap();
    assert_eq!(received, (0..10u32).collect::<Vec<_>>());
}

//endregion

//region Block On Thread

#[test]
fn block_on_thread_returns_ready_output() {
    assert_eq!(block_on_thread(async { 5 }), 5);
}

#[test]
fn block_on_thread_wakes_when_another_thread_sends() {
    let (sender, receiver) = FeagiChannelDynamicBuilder::<Number>::new(1).create_channels();
    let producer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        sender.try_send(Number(9)).unwrap();
    });
    assert_eq!(block_on_thread(receiver.async_recv()).unwrap(), Number(9));
    producer.join().unwrap();
}

//endregion

//region Request / Response

#[test]
fn many_requesters_get_their_own_responses() {
    let (requester, responder) = FeagiRequestResponseDynamicBuilder::<Number, Number>::new(4).create();
    let server = std::thread::spawn(move || block_on_thread(async {
        while responder.async_process_next(async |request: Number| Number(request.0 * 10)).await.is_ok() {}
    }));
    let clients: Vec<_> = (0..4u64).map(|client| {
        let mut requester = requester.clone();
        std::thread::spawn(move || block_on_thread(async {
            for index in 0..200u64 {
                let value = client * 1000 + index;
                assert_eq!(requester.make_request(Number(value)).await.unwrap(), Number(value * 10));
            }
        }))
    }).collect();
    drop(requester);
    for client in clients {
        client.join().unwrap();
    }
    server.join().unwrap();
}

#[test]
fn in_place_request_and_response() {
    let (mut requester, responder) = FeagiRequestResponseDynamicBuilder::<Number, Number>::new(2).create();
    let server = std::thread::spawn(move || block_on_thread(
        responder.async_process_next_in_place(async |request: &mut Number, response: &mut Number| response.0 = request.0 + 1)
    ));
    let output = block_on_thread(requester.make_request_in_place(|request| request.0 = 41, |response| response.0)).unwrap();
    assert_eq!(output, 42);
    server.join().unwrap().unwrap();
}

#[test]
fn try_process_next_reports_empty_queue() {
    let (_requester, responder) = FeagiRequestResponseDynamicBuilder::<Number, Number>::new(1).create();
    let result = block_on_thread(responder.try_process_next(async |request: Number| request));
    assert!(matches!(result, Err(FeagiChannelReceiveError::ChannelEmpty)));
    let result = block_on_thread(responder.try_process_next_in_place(async |_request: &mut Number, _response: &mut Number| {}));
    assert!(matches!(result, Err(FeagiChannelReceiveError::ChannelEmpty)));
}

#[test]
fn try_process_next_answers_queued_request() {
    let (mut requester, responder) = FeagiRequestResponseDynamicBuilder::<Number, Number>::new(1).create();
    let client = std::thread::spawn(move || block_on_thread(requester.make_request(Number(4))));
    loop {
        match block_on_thread(responder.try_process_next(async |request: Number| Number(request.0 + 1))) {
            Ok(()) => break,
            Err(FeagiChannelReceiveError::ChannelEmpty) => std::thread::yield_now(),
            Err(error) => panic!("unexpected error: {error}"),
        }
    }
    assert_eq!(client.join().unwrap().unwrap(), Number(5));
}

#[test]
fn responder_reports_closed_once_every_requester_dropped() {
    let (requester, responder) = FeagiRequestResponseDynamicBuilder::<Number, Number>::new(1).create();
    let second_requester = requester.clone();
    drop(requester);
    drop(second_requester);
    let result = block_on_thread(responder.async_process_next(async |request: Number| request));
    assert!(matches!(result, Err(FeagiChannelReceiveError::ChannelClosed)));
}

#[test]
fn requests_fail_once_responder_is_gone() {
    let (mut requester, responder) = FeagiRequestResponseDynamicBuilder::<Number, Number>::new(2).create();
    drop(responder);
    assert!(matches!(block_on_thread(requester.make_request(Number(1))), Err(FeagiRequestError::RequestNotSent(_))));
}

#[test]
fn cancelled_request_reply_is_not_delivered_to_next_request() {
    let (mut requester, responder) = FeagiRequestResponseDynamicBuilder::<Number, Number>::new(4).create();
    // Request 1 is sent, then its future is dropped while waiting for the reply
    assert!(poll_once(requester.make_request(Number(1))).is_pending());
    let server = std::thread::spawn(move || block_on_thread(async {
        for _ in 0..2 {
            responder.async_process_next(async |request: Number| Number(request.0 * 10)).await.unwrap();
        }
    }));
    assert_eq!(block_on_thread(requester.make_request(Number(2))).unwrap(), Number(20));
    server.join().unwrap();
}

#[test]
fn cancelled_processing_reports_dropped_request() {
    let (mut requester, responder) = FeagiRequestResponseDynamicBuilder::<Number, Number>::new(2).create();
    let server = std::thread::spawn(move || {
        // Ready means the queue was empty. Pending means processing started, then it is dropped
        loop {
            let started = poll_once(responder.try_process_next(async |_request: Number| {
                core::future::pending::<()>().await;
                Number(0)
            })).is_pending();
            if started {
                break;
            }
            std::thread::yield_now();
        }
    });
    let result = block_on_thread(requester.make_request(Number(5)));
    assert!(matches!(result, Err(FeagiRequestError::ResponderDroppedRequest)));
    server.join().unwrap();
}

#[test]
fn dropping_responder_answers_queued_requests() {
    let (mut requester, responder) = FeagiRequestResponseDynamicBuilder::<Number, Number>::new(2).create();
    let client = std::thread::spawn(move || block_on_thread(requester.make_request(Number(1))));
    // Give the request time to be queued, then drop the responder without processing it
    std::thread::sleep(Duration::from_millis(50));
    drop(responder);
    assert!(matches!(client.join().unwrap(), Err(FeagiRequestError::ResponderDroppedRequest)));
}

#[test]
fn request_from_dropped_requester_is_skipped() {
    let (mut requester, responder) = FeagiRequestResponseDynamicBuilder::<Number, Number>::new(2).create();
    assert!(poll_once(requester.make_request(Number(1))).is_pending());
    drop(requester);

    let processed = AtomicBool::new(false);
    let result = block_on_thread(responder.try_process_next(async |request: Number| {
        processed.store(true, Ordering::SeqCst);
        request
    }));
    assert!(result.is_ok());
    assert!(!processed.load(Ordering::SeqCst));
}

//endregion

//region Workers

#[test]
fn std_thread_spawner_runs_body_on_its_own_thread() {
    let caller_thread = std::thread::current().id();
    let ran_elsewhere = Arc::new(AtomicBool::new(false));
    let ran_elsewhere_in_body = ran_elsewhere.clone();
    let join_handle = FeagiStdThreadSpawner.spawn_worker(move || async move {
        ran_elsewhere_in_body.store(std::thread::current().id() != caller_thread, Ordering::SeqCst);
    }).unwrap();
    join_handle.join().unwrap();
    assert!(ran_elsewhere.load(Ordering::SeqCst));
}

#[test]
fn worker_answers_custom_notifications_then_stops() {
    let mut handle = FeagiWorkerHandleDynamic::<u32, u32, _>::start(&FeagiStdThreadSpawner, 2, |notifications| {
        move || doubling_worker(notifications)
    }).unwrap();

    block_on_thread(async {
        for value in 0..50u32 {
            assert!(matches!(handle.notify(value).await.unwrap(), FeagiWorkerReply::Custom(doubled) if doubled == value * 2));
        }
        handle.stop().await.unwrap();
    });
    handle.join().unwrap();
}

#[test]
fn stop_after_worker_exited_errors() {
    let mut handle = FeagiWorkerHandleDynamic::<u32, u32, _>::start(&FeagiStdThreadSpawner, 1, |notifications| {
        move || async move { drop(notifications) }
    }).unwrap();
    std::thread::sleep(Duration::from_millis(50));
    assert!(matches!(block_on_thread(handle.stop()), Err(FeagiRequestError::RequestNotSent(_))));
    handle.join().unwrap();
}

#[test]
fn join_reports_worker_panic() {
    let handle = FeagiWorkerHandleDynamic::<u32, u32, _>::start(&FeagiStdThreadSpawner, 1, |notifications| {
        move || async move {
            let _notifications = notifications;
            panic!("worker failed");
        }
    }).unwrap();
    assert!(handle.join().is_err());
}

#[test]
fn worker_can_send_to_additional_channels() {
    let (event_sender, event_receiver) = FeagiChannelDynamicBuilder::<Number>::new(4).create_channels();
    let mut handle = FeagiWorkerHandleDynamic::<u32, (), _>::start(&FeagiStdThreadSpawner, 1, |notifications| {
        move || async move {
            let mut running = true;
            while running {
                let result = notifications.async_process_next(async |notification| match notification {
                    FeagiWorkerNotification::Stop => {
                        running = false;
                        FeagiWorkerReply::Acknowledged
                    }
                    FeagiWorkerNotification::Custom(value) => {
                        event_sender.async_send(Number(u64::from(value))).await.unwrap();
                        FeagiWorkerReply::Acknowledged
                    }
                    FeagiWorkerNotification::Nothing => FeagiWorkerReply::Nothing,
                }).await;
                if result.is_err() {
                    break;
                }
            }
        }
    }).unwrap();

    block_on_thread(async {
        handle.notify(3).await.unwrap();
        handle.notify(4).await.unwrap();
        handle.stop().await.unwrap();
    });
    handle.join().unwrap();
    assert_eq!(event_receiver.try_recv().unwrap(), Number(3));
    assert_eq!(event_receiver.try_recv().unwrap(), Number(4));
}

#[test]
fn worker_can_spawn_and_stop_a_child_worker() {
    let child_stopped = Arc::new(AtomicU32::new(0));
    let child_stopped_in_parent = child_stopped.clone();
    let mut parent = FeagiWorkerHandleDynamic::<u32, u32, _>::start(&FeagiStdThreadSpawner, 1, |notifications| {
        move || async move {
            let mut child = FeagiWorkerHandleDynamic::<u32, u32, _>::start(&FeagiStdThreadSpawner, 1, |child_notifications| {
                move || doubling_worker(child_notifications)
            }).unwrap();

            let mut running = true;
            while running {
                // Custom notifications are forwarded to the child, Stop stops the child first
                let result = notifications.async_process_next(async |notification| match notification {
                    FeagiWorkerNotification::Stop => {
                        child.stop().await.unwrap();
                        child_stopped_in_parent.fetch_add(1, Ordering::SeqCst);
                        running = false;
                        FeagiWorkerReply::Acknowledged
                    }
                    FeagiWorkerNotification::Custom(value) => child.notify(value).await.unwrap(),
                    FeagiWorkerNotification::Nothing => FeagiWorkerReply::Nothing,
                }).await;
                if result.is_err() {
                    break;
                }
            }
            child.join().unwrap();
        }
    }).unwrap();

    block_on_thread(async {
        assert!(matches!(parent.notify(21).await.unwrap(), FeagiWorkerReply::Custom(42)));
        parent.stop().await.unwrap();
    });
    parent.join().unwrap();
    assert_eq!(child_stopped.load(Ordering::SeqCst), 1);
}

//endregion
