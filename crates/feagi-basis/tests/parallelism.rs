#![cfg(feature = "std")]

use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll, Waker};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use feagi_basis::parallelism::block_on_thread;
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::{FeagiChannelReceiver, FeagiChannelTransmitter};
use feagi_basis::parallelism::feagi_channel::base_channels::base_errors::{FeagiChannelReceiveError, FeagiChannelTransmitError};
use feagi_basis::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;
use feagi_basis::parallelism::feagi_channel::recycling::{Recyclable, RecyclableRecyclePolicy};
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiRequesterAlloc, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_channel::request_response_channel::feagi_request_response_error::FeagiRequestResponseError;
use feagi_basis::parallelism::feagi_signal::implementations::std_signal::FeagiSignalStdBackend;
use feagi_basis::parallelism::feagi_signal::signal_errors::FeagiSignalClosed;
use feagi_basis::parallelism::feagi_signal::signal_traits::{FeagiSignalListener, FeagiSignalNotifier};
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::FeagiWorkerSpawnError;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::{FeagiWorkerBuilder, WorkerStartError};
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_router::{FeagiWorkerAwaitingRouter, FeagiWorkerExit, FeagiWorkerPollingRouter, FeagiWorkerStep};
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};

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

/// Parent side of a test worker's request channel
type WorkerRequester = FeagiRequesterAlloc<FeagiWorkerRequest<u32>, FeagiWorkerResponse<u32>>;
/// Worker side of a test worker's request channel
type WorkerResponder = FeagiResponderAlloc<FeagiWorkerRequest<u32>, FeagiWorkerResponse<u32>>;

/// Creates the request channel between a test worker and its parent
fn worker_request_channel(capacity: usize) -> (WorkerRequester, WorkerResponder) {
    FeagiRequestResponseChannelsAllocBackend::create_channels(capacity)
}

/// Worker state that doubles custom requests
struct DoublingHandler;

impl FeagiWorkerCommandProcessor for DoublingHandler {
    type CustomRequest = u32;
    type CustomResponse = u32;

    async fn on_custom(&mut self, request: u32) -> u32 {
        request * 2
    }
}

/// Worker body that doubles custom requests and stops on `Stop`
async fn doubling_worker(requests: WorkerResponder) {
    FeagiWorkerAwaitingRouter::new(DoublingHandler, requests).run_until_stopped().await;
}

/// Worker state that publishes each custom request onto an outgoing event channel, and echoes it
/// back as the response
struct EventHandler {
    event_sender: thingbuf::mpsc::Sender<Number, RecyclableRecyclePolicy<Number>>,
}

impl FeagiWorkerCommandProcessor for EventHandler {
    type CustomRequest = u32;
    type CustomResponse = u32;

    async fn on_custom(&mut self, request: u32) -> u32 {
        self.event_sender.async_send(Number(u64::from(request))).await.unwrap();
        request
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
    let (sender, receiver) = FeagiChannelAllocBuilder::<Number>::create_channels(4);
    for value in 0..4 {
        FeagiChannelTransmitter::try_send(&sender, Number(value)).unwrap();
    }
    for value in 0..4 {
        assert_eq!(FeagiChannelReceiver::try_recv(&receiver).unwrap(), Number(value));
    }
}

#[test]
fn dynamic_channel_helpers_track_queue_state() {
    let (sender, receiver) = FeagiChannelAllocBuilder::<Number>::create_channels(2);
    assert_eq!(FeagiChannelTransmitter::capacity(&sender), 2);
    assert_eq!(FeagiChannelReceiver::capacity(&receiver), 2);
    assert!(FeagiChannelTransmitter::is_empty(&sender) && FeagiChannelReceiver::is_empty(&receiver));
    assert!(!sender.is_full() && !receiver.is_full());
    assert_eq!(FeagiChannelTransmitter::remaining(&sender), 2);

    FeagiChannelTransmitter::try_send(&sender, Number(1)).unwrap();
    assert_eq!(FeagiChannelTransmitter::len(&sender), 1);
    assert_eq!(FeagiChannelReceiver::len(&receiver), 1);
    assert_eq!(FeagiChannelReceiver::remaining(&receiver), 1);
    assert!(!FeagiChannelTransmitter::is_empty(&sender));

    FeagiChannelTransmitter::try_send(&sender, Number(2)).unwrap();
    assert!(sender.is_full() && receiver.is_full());
    assert_eq!(FeagiChannelTransmitter::remaining(&sender), 0);

    FeagiChannelReceiver::try_recv(&receiver).unwrap();
    assert!(!sender.is_full());
    assert_eq!(FeagiChannelReceiver::len(&receiver), 1);
}

#[test]
fn dynamic_channel_try_send_errors_when_full() {
    let (sender, _receiver) = FeagiChannelAllocBuilder::<Number>::create_channels(1);
    FeagiChannelTransmitter::try_send(&sender, Number(1)).unwrap();
    assert!(matches!(
        FeagiChannelTransmitter::try_send(&sender, Number(2)),
        Err(FeagiChannelTransmitError::ChannelFull)
    ));
    assert!(matches!(
        FeagiChannelTransmitter::try_send_ref(&sender),
        Err(FeagiChannelTransmitError::ChannelFull)
    ));
}

#[test]
fn dynamic_channel_try_recv_errors_when_empty() {
    let (_sender, receiver) = FeagiChannelAllocBuilder::<Number>::create_channels(1);
    assert!(matches!(
        FeagiChannelReceiver::try_recv(&receiver),
        Err(FeagiChannelReceiveError::ChannelEmpty)
    ));
    assert!(matches!(
        FeagiChannelReceiver::try_recv_ref(&receiver),
        Err(FeagiChannelReceiveError::ChannelEmpty)
    ));
}

#[test]
fn dynamic_channel_send_errors_once_receiver_dropped() {
    let (sender, receiver) = FeagiChannelAllocBuilder::<Number>::create_channels(2);
    drop(receiver);
    assert!(matches!(
        FeagiChannelTransmitter::try_send(&sender, Number(1)),
        Err(FeagiChannelTransmitError::ChannelClosed)
    ));
    assert!(matches!(
        FeagiChannelTransmitter::try_send_ref(&sender),
        Err(FeagiChannelTransmitError::ChannelClosed)
    ));
    assert!(matches!(
        block_on_thread(sender.async_send(Number(1))),
        Err(FeagiChannelTransmitError::ChannelClosed)
    ));
}

#[test]
fn dynamic_channel_drains_queue_before_reporting_closed() {
    let (sender, receiver) = FeagiChannelAllocBuilder::<Number>::create_channels(2);
    FeagiChannelTransmitter::try_send(&sender, Number(7)).unwrap();
    drop(sender);
    // TODO: Restore with `FeagiChannelReceiver::is_closed`, disabled due to a thingbuf 0.1.6 bug
    // assert!(receiver.is_closed());
    assert_eq!(FeagiChannelReceiver::try_recv(&receiver).unwrap(), Number(7));
    assert!(matches!(
        FeagiChannelReceiver::try_recv(&receiver),
        Err(FeagiChannelReceiveError::ChannelClosed)
    ));
    assert!(matches!(
        block_on_thread(receiver.async_recv()),
        Err(FeagiChannelReceiveError::ChannelClosed)
    ));
}

#[test]
fn dynamic_channel_closes_only_after_every_cloned_sender_dropped() {
    let (sender, receiver) = FeagiChannelAllocBuilder::<Number>::create_channels(2);
    let second_sender = sender.clone();
    drop(sender);
    // TODO: Restore with `FeagiChannelReceiver::is_closed`, disabled due to a thingbuf 0.1.6 bug.
    // thingbuf reports closed here even though `second_sender` is alive
    // assert!(!receiver.is_closed());
    FeagiChannelTransmitter::try_send(&second_sender, Number(3)).unwrap();
    assert!(matches!(FeagiChannelReceiver::try_recv(&receiver), Ok(Number(3))));
    assert!(matches!(
        FeagiChannelReceiver::try_recv(&receiver),
        Err(FeagiChannelReceiveError::ChannelEmpty)
    ));
    drop(second_sender);
    // assert!(receiver.is_closed());
    assert!(matches!(
        FeagiChannelReceiver::try_recv(&receiver),
        Err(FeagiChannelReceiveError::ChannelClosed)
    ));
}

#[test]
fn dynamic_channel_async_send_and_recv_across_threads() {
    let (sender, receiver) = FeagiChannelAllocBuilder::<Number>::create_channels(2);
    let producers: Vec<_> = (0..4u64)
        .map(|producer| {
            let sender = sender.clone();
            std::thread::spawn(move || {
                block_on_thread(async {
                    for value in 0..100u64 {
                        sender.async_send(Number(producer * 1000 + value)).await.unwrap();
                    }
                })
            })
        })
        .collect();
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
    let expected: Vec<u64> = (0..4u64)
        .flat_map(|producer| (0..100u64).map(move |value| producer * 1000 + value))
        .collect();
    assert_eq!(received, expected);
}


//endregion

//region Send / Receive by Reference

#[test]
fn send_ref_and_recv_ref_reuse_the_slot_value() {
    let (sender, receiver) = FeagiChannelAllocBuilder::<Buffer>::create_channels(1);

    {
        let mut slot = FeagiChannelTransmitter::try_send_ref(&sender).unwrap();
        slot.values.extend_from_slice(&[1, 2, 3]);
    }
    {
        let slot = FeagiChannelReceiver::try_recv_ref(&receiver).unwrap();
        assert_eq!(slot.values, vec![1, 2, 3]);
    }

    // The same slot comes back recycled, keeping its allocation
    let slot = FeagiChannelTransmitter::try_send_ref(&sender).unwrap();
    assert!(slot.values.is_empty());
    assert!(slot.values.capacity() >= 3);
    assert_eq!(slot.times_recycled, 1);
}

#[test]
fn by_value_recv_replaces_the_slot_value() {
    let (sender, receiver) = FeagiChannelAllocBuilder::<Buffer>::create_channels(1);
    {
        let mut slot = FeagiChannelTransmitter::try_send_ref(&sender).unwrap();
        slot.values.extend_from_slice(&[1, 2, 3]);
    }
    let received = FeagiChannelReceiver::try_recv(&receiver).unwrap();
    assert_eq!(received.values, vec![1, 2, 3]);

    // The received value was moved out, so the slot holds a fresh value with no allocation
    let slot = FeagiChannelTransmitter::try_send_ref(&sender).unwrap();
    assert_eq!(slot.values.capacity(), 0);
}

#[test]
fn dropping_unwritten_send_ref_still_publishes_slot() {
    let (sender, receiver) = FeagiChannelAllocBuilder::<Buffer>::create_channels(1);
    drop(FeagiChannelTransmitter::try_send_ref(&sender).unwrap());
    let slot = FeagiChannelReceiver::try_recv_ref(&receiver).unwrap();
    assert!(slot.values.is_empty());
}

#[test]
fn async_ref_methods_round_trip() {
    let (sender, receiver) = FeagiChannelAllocBuilder::<Buffer>::create_channels(1);
    let producer = std::thread::spawn(move || {
        block_on_thread(async {
            for value in 0..10u32 {
                let mut slot = sender.async_send_ref().await.unwrap();
                slot.values.push(value);
            }
        })
    });
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
    let (sender, receiver) = FeagiChannelAllocBuilder::<Number>::create_channels(1);
    let producer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        FeagiChannelTransmitter::try_send(&sender, Number(9)).unwrap();
    });
    assert_eq!(block_on_thread(receiver.async_recv()).unwrap(), Number(9));
    producer.join().unwrap();
}

//endregion

//region Signals

#[test]
fn signal_try_take_returns_only_the_latest_value_once() {
    let (notifier, mut listener) = FeagiSignalStdBackend::create_signal_pair::<u32>();
    assert_eq!(listener.try_take(), Ok(None));

    notifier.notify(1);
    notifier.notify(2);
    assert_eq!(listener.try_take(), Ok(Some(2)));
    assert_eq!(listener.try_take(), Ok(None));
}

#[test]
fn signal_notify_with_merges_into_the_pending_value() {
    let (notifier, mut listener) = FeagiSignalStdBackend::create_signal_pair::<u8>();
    let set_flag = |flag: u8| notifier.notify_with(|pending| *pending.get_or_insert(0) |= flag);

    set_flag(0b001);
    set_flag(0b100);
    assert_eq!(listener.try_take(), Ok(Some(0b101)));
    set_flag(0b010);
    assert_eq!(listener.try_take(), Ok(Some(0b010)));
}

#[test]
fn signal_wait_is_pending_until_notified() {
    let (notifier, mut listener) = FeagiSignalStdBackend::create_signal_pair::<u32>();
    assert!(poll_once(listener.wait()).is_pending());
    notifier.notify(7);
    assert_eq!(poll_once(listener.wait()), Poll::Ready(Ok(7)));
}

#[test]
fn signal_wait_wakes_when_another_thread_notifies() {
    let (notifier, mut listener) = FeagiSignalStdBackend::create_signal_pair::<u32>();
    let producer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        notifier.notify(9);
        // Kept alive until joined, so the wait can only end by being notified
        notifier
    });
    assert_eq!(block_on_thread(listener.wait()), Ok(9));
    drop(producer.join().unwrap());
}

#[test]
fn signal_closes_once_every_notifier_is_dropped_after_delivering_pending_value() {
    let (notifier, mut listener) = FeagiSignalStdBackend::create_signal_pair::<u32>();
    let cloned_notifier = notifier.clone();
    drop(notifier);
    cloned_notifier.notify(3);
    assert_eq!(listener.try_take(), Ok(Some(3)));
    assert_eq!(listener.try_take(), Ok(None));

    cloned_notifier.notify(4);
    drop(cloned_notifier);
    assert_eq!(listener.try_take(), Ok(Some(4)));
    assert_eq!(listener.try_take(), Err(FeagiSignalClosed));
    assert_eq!(block_on_thread(listener.wait()), Err(FeagiSignalClosed));
}

#[test]
fn signal_wait_wakes_when_last_notifier_is_dropped_on_another_thread() {
    let (notifier, mut listener) = FeagiSignalStdBackend::create_signal_pair::<u32>();
    let producer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        drop(notifier);
    });
    assert_eq!(block_on_thread(listener.wait()), Err(FeagiSignalClosed));
    producer.join().unwrap();
}

#[test]
fn signal_notify_after_listener_dropped_does_nothing() {
    let (notifier, listener) = FeagiSignalStdBackend::create_signal_pair::<u32>();
    drop(listener);
    notifier.notify(1);
    notifier.notify_with(|_| panic!("update ran after the listener was dropped"));
}

#[test]
fn signal_still_works_after_a_notify_with_update_panics() {
    let (notifier, mut listener) = FeagiSignalStdBackend::create_signal_pair::<u32>();
    let panicking_notifier = notifier.clone();
    let result = std::thread::spawn(move || panicking_notifier.notify_with(|_| panic!("update panicked"))).join();
    assert!(result.is_err());

    notifier.notify(5);
    assert_eq!(listener.try_take(), Ok(Some(5)));
}

//endregion

//region Request / Response

#[test]
fn many_requesters_get_their_own_responses() {
    let (requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_channels::<Number, Number>(4);
    let server = std::thread::spawn(move || {
        block_on_thread(async { while responder.async_process_next(async |request: Number| Number(request.0 * 10)).await.is_ok() {} })
    });
    let clients: Vec<_> = (0..4u64)
        .map(|client| {
            let mut requester = requester.clone();
            std::thread::spawn(move || {
                block_on_thread(async {
                    for index in 0..200u64 {
                        let value = client * 1000 + index;
                        assert_eq!(requester.make_request(Number(value)).await.unwrap(), Number(value * 10));
                    }
                })
            })
        })
        .collect();
    drop(requester);
    for client in clients {
        client.join().unwrap();
    }
    server.join().unwrap();
}

#[test]
fn in_place_request_and_response() {
    let (mut requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_channels::<Number, Number>(2);
    let server = std::thread::spawn(move || {
        block_on_thread(responder.async_process_next_in_place(async |request: &mut Number, response: &mut Number| response.0 = request.0 + 1))
    });
    let output = block_on_thread(requester.make_request_in_place(|request| request.0 = 41, |response| response.0)).unwrap();
    assert_eq!(output, 42);
    server.join().unwrap().unwrap();
}

#[test]
fn try_process_next_reports_empty_queue() {
    let (_requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_channels::<Number, Number>(1);
    let result = block_on_thread(responder.try_process_next(async |request: Number| request));
    assert!(matches!(result, Err(FeagiChannelReceiveError::ChannelEmpty)));
    let result = block_on_thread(responder.try_process_next_in_place(async |_request: &mut Number, _response: &mut Number| {}));
    assert!(matches!(result, Err(FeagiChannelReceiveError::ChannelEmpty)));
}

#[test]
fn try_process_next_answers_queued_request() {
    let (mut requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_channels::<Number, Number>(1);
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
    let (requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_channels::<Number, Number>(1);
    let second_requester = requester.clone();
    drop(requester);
    drop(second_requester);
    let result = block_on_thread(responder.async_process_next(async |request: Number| request));
    assert!(matches!(result, Err(FeagiChannelReceiveError::ChannelClosed)));
}

#[test]
fn requests_fail_once_responder_is_gone() {
    let (mut requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_channels::<Number, Number>(2);
    drop(responder);
    assert!(matches!(
        block_on_thread(requester.make_request(Number(1))),
        Err(FeagiRequestResponseError::RequestNotSent(_))
    ));
}

#[test]
fn cancelled_request_reply_is_not_delivered_to_next_request() {
    let (mut requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_channels::<Number, Number>(4);
    // Request 1 is sent, then its future is dropped while waiting for the reply
    assert!(poll_once(requester.make_request(Number(1))).is_pending());
    let server = std::thread::spawn(move || {
        block_on_thread(async {
            for _ in 0..2 {
                responder
                    .async_process_next(async |request: Number| Number(request.0 * 10))
                    .await
                    .unwrap();
            }
        })
    });
    assert_eq!(block_on_thread(requester.make_request(Number(2))).unwrap(), Number(20));
    server.join().unwrap();
}

#[test]
fn cancelled_processing_reports_dropped_request() {
    let (mut requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_channels::<Number, Number>(2);
    let server = std::thread::spawn(move || {
        // Ready means the queue was empty. Pending means processing started, then it is dropped
        loop {
            let started = poll_once(responder.try_process_next(async |_request: Number| {
                core::future::pending::<()>().await;
                Number(0)
            }))
            .is_pending();
            if started {
                break;
            }
            std::thread::yield_now();
        }
    });
    let result = block_on_thread(requester.make_request(Number(5)));
    assert!(matches!(result, Err(FeagiRequestResponseError::ResponderDroppedRequest)));
    server.join().unwrap();
}

#[test]
fn dropping_responder_answers_queued_requests() {
    let (mut requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_channels::<Number, Number>(2);
    let client = std::thread::spawn(move || block_on_thread(requester.make_request(Number(1))));
    // Give the request time to be queued, then drop the responder without processing it
    std::thread::sleep(Duration::from_millis(50));
    drop(responder);
    assert!(matches!(client.join().unwrap(), Err(FeagiRequestResponseError::ResponderDroppedRequest)));
}

#[test]
fn request_from_dropped_requester_is_skipped() {
    let (mut requester, responder) = FeagiRequestResponseChannelsAllocBackend::create_channels::<Number, Number>(2);
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

//region Worker Requests

#[test]
fn worker_request_and_response_start_and_recycle_as_nothing() {
    assert!(matches!(
        <FeagiWorkerRequest<u32> as Recyclable>::new_for_recycler(),
        FeagiWorkerRequest::Nothing
    ));
    assert!(matches!(
        <FeagiWorkerResponse<u32> as Recyclable>::new_for_recycler(),
        FeagiWorkerResponse::Nothing
    ));

    let mut request = FeagiWorkerRequest::Custom(5u32);
    request.recycle_self();
    assert!(matches!(request, FeagiWorkerRequest::Nothing));

    let mut response = FeagiWorkerResponse::Custom(5u32);
    response.recycle_self();
    assert!(matches!(response, FeagiWorkerResponse::Nothing));
}

//endregion

//region Std Thread Spawner

#[test]
fn std_thread_spawner_runs_body_on_its_own_thread() {
    let caller_thread = std::thread::current().id();
    let ran_elsewhere = Arc::new(AtomicBool::new(false));
    let ran_elsewhere_in_body = ran_elsewhere.clone();
    let join_handle = FeagiStdThreadSpawner
        .spawn_worker(move || async move {
            ran_elsewhere_in_body.store(std::thread::current().id() != caller_thread, Ordering::SeqCst);
        })
        .unwrap();
    join_handle.join().unwrap();
    assert!(ran_elsewhere.load(Ordering::SeqCst));
}

#[test]
fn std_thread_spawner_join_reports_body_panic() {
    let join_handle = FeagiStdThreadSpawner.spawn_worker(|| async { panic!("worker failed") }).unwrap();
    assert!(join_handle.join().is_err());
}

#[test]
fn spawned_worker_answers_custom_requests_then_acknowledges_stop() {
    let (mut requester, responder) = worker_request_channel(2);
    let join_handle = FeagiStdThreadSpawner.spawn_worker(move || doubling_worker(responder)).unwrap();

    block_on_thread(async {
        for value in 0..50u32 {
            let response = requester.make_request(FeagiWorkerRequest::Custom(value)).await.unwrap();
            assert!(matches!(response, FeagiWorkerResponse::Custom(doubled) if doubled == value * 2));
        }
        let response = requester.make_request(FeagiWorkerRequest::Stop).await.unwrap();
        assert!(matches!(response, FeagiWorkerResponse::Acknowledged));
    });
    join_handle.join().unwrap();
}

#[test]
fn request_after_worker_exited_errors() {
    let (mut requester, responder) = worker_request_channel(1);
    let join_handle = FeagiStdThreadSpawner.spawn_worker(move || async move { drop(responder) }).unwrap();
    join_handle.join().unwrap();
    let result = block_on_thread(requester.make_request(FeagiWorkerRequest::Stop));
    assert!(matches!(result, Err(FeagiRequestResponseError::RequestNotSent(_))));
}

#[test]
fn spawned_worker_can_send_to_additional_channels() {
    let (event_sender, event_receiver) = FeagiChannelAllocBuilder::<Number>::create_channels(4);
    let (mut requester, responder) = worker_request_channel(1);
    let join_handle = FeagiStdThreadSpawner
        .spawn_worker(move || async move {
            FeagiWorkerAwaitingRouter::new(EventHandler { event_sender }, responder).run_until_stopped().await;
        })
        .unwrap();

    block_on_thread(async {
        requester.make_request(FeagiWorkerRequest::Custom(3)).await.unwrap();
        requester.make_request(FeagiWorkerRequest::Custom(4)).await.unwrap();
        requester.make_request(FeagiWorkerRequest::Stop).await.unwrap();
    });
    join_handle.join().unwrap();
    assert_eq!(FeagiChannelReceiver::try_recv(&event_receiver).unwrap(), Number(3));
    assert_eq!(FeagiChannelReceiver::try_recv(&event_receiver).unwrap(), Number(4));
}

/// Worker state that forwards custom requests to a child worker, and stops the child first when
/// stopped
struct ParentHandler {
    child_requester: WorkerRequester,
    child_stopped: Arc<AtomicU32>,
}

impl FeagiWorkerCommandProcessor for ParentHandler {
    type CustomRequest = u32;
    type CustomResponse = u32;

    async fn on_stop(&mut self) {
        self.child_requester.make_request(FeagiWorkerRequest::Stop).await.unwrap();
        self.child_stopped.fetch_add(1, Ordering::SeqCst);
    }

    async fn on_custom(&mut self, request: u32) -> u32 {
        match self.child_requester.make_request(FeagiWorkerRequest::Custom(request)).await.unwrap() {
            FeagiWorkerResponse::Custom(response) => response,
            _ => panic!("child replied with a non custom response"),
        }
    }
}

#[test]
fn spawned_worker_can_spawn_and_stop_a_child_worker() {
    let child_stopped = Arc::new(AtomicU32::new(0));
    let child_stopped_in_parent = child_stopped.clone();
    let (mut parent_requester, parent_responder) = worker_request_channel(1);
    let parent_join_handle = FeagiStdThreadSpawner
        .spawn_worker(move || async move {
            let (child_requester, child_responder) = worker_request_channel(1);
            let child_join_handle = FeagiStdThreadSpawner.spawn_worker(move || doubling_worker(child_responder)).unwrap();
            let handler = ParentHandler {
                child_requester,
                child_stopped: child_stopped_in_parent,
            };
            FeagiWorkerAwaitingRouter::new(handler, parent_responder).run_until_stopped().await;
            child_join_handle.join().unwrap();
        })
        .unwrap();

    block_on_thread(async {
        let response = parent_requester.make_request(FeagiWorkerRequest::Custom(21)).await.unwrap();
        assert!(matches!(response, FeagiWorkerResponse::Custom(42)));
        parent_requester.make_request(FeagiWorkerRequest::Stop).await.unwrap();
    });
    parent_join_handle.join().unwrap();
    assert_eq!(child_stopped.load(Ordering::SeqCst), 1);
}

#[test]
fn polling_router_reports_each_step() {
    let (mut requester, responder) = worker_request_channel(2);
    let mut commands = FeagiWorkerPollingRouter::new(DoublingHandler, responder);

    block_on_thread(async {
        assert_eq!(commands.try_handle_next().await, FeagiWorkerStep::Idle);

        // The requester waits for each reply, so the request is sent and handled concurrently.
        // `futures_join` polls the request first, so it is queued before `try_handle_next` checks
        let (response, step) = futures_join(
            requester.make_request(FeagiWorkerRequest::Custom(5)),
            commands.try_handle_next(),
        )
        .await;
        assert!(matches!(response.unwrap(), FeagiWorkerResponse::Custom(10)));
        assert_eq!(step, FeagiWorkerStep::Handled);

        let (response, step) = futures_join(
            requester.make_request(FeagiWorkerRequest::Stop),
            commands.try_handle_next(),
        )
        .await;
        assert!(matches!(response.unwrap(), FeagiWorkerResponse::Acknowledged));
        assert_eq!(step, FeagiWorkerStep::Stopped);
    });

    drop(requester);
    assert_eq!(block_on_thread(commands.try_handle_next()), FeagiWorkerStep::Disconnected);
}

#[test]
fn awaiting_router_reports_each_exit() {
    let (mut requester, responder) = worker_request_channel(2);
    let mut commands = FeagiWorkerAwaitingRouter::new(DoublingHandler, responder);

    block_on_thread(async {
        let (response, exit) = futures_join(
            requester.make_request(FeagiWorkerRequest::Custom(5)),
            commands.handle_next(),
        )
        .await;
        assert!(matches!(response.unwrap(), FeagiWorkerResponse::Custom(10)));
        assert_eq!(exit, None);

        let (response, exit) = futures_join(
            requester.make_request(FeagiWorkerRequest::Stop),
            commands.handle_next(),
        )
        .await;
        assert!(matches!(response.unwrap(), FeagiWorkerResponse::Acknowledged));
        assert_eq!(exit, Some(FeagiWorkerExit::Stopped));
    });

    drop(requester);
    assert_eq!(block_on_thread(commands.handle_next()), Some(FeagiWorkerExit::Disconnected));
}

#[test]
fn awaiting_router_gives_back_processor_when_disconnected() {
    let (requester, responder) = worker_request_channel(1);
    drop(requester);
    let (DoublingHandler, exit) = block_on_thread(FeagiWorkerAwaitingRouter::new(DoublingHandler, responder).run_until_stopped());
    assert_eq!(exit, FeagiWorkerExit::Disconnected);
}

/// Polls both futures until both complete, on the current thread
async fn futures_join<A: Future, B: Future>(first: A, second: B) -> (A::Output, B::Output) {
    let mut first = pin!(first);
    let mut second = pin!(second);
    let mut first_output = None;
    let mut second_output = None;
    core::future::poll_fn(|context| {
        if first_output.is_none() {
            if let Poll::Ready(output) = first.as_mut().poll(context) {
                first_output = Some(output);
            }
        }
        if second_output.is_none() {
            if let Poll::Ready(output) = second.as_mut().poll(context) {
                second_output = Some(output);
            }
        }
        if first_output.is_some() && second_output.is_some() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await;
    (first_output.unwrap(), second_output.unwrap())
}

//endregion

//region Worker Builder

/// Builds a worker that doubles custom requests, with no extra channels
struct DoublingWorkerBuilder {
    request_queue_capacity: usize,
}

impl FeagiWorkerBuilder<FeagiStdThreadSpawner> for DoublingWorkerBuilder {
    type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;
    type BuildInputs = ();
    type BuildOutputs = ();
    type RequiredChannelPairs = ();
    type WorkerCommandHandler = DoublingHandler;
    type WorkerStartError = FeagiWorkerSpawnError;

    fn build_worker(
        self,
        spawner: &FeagiStdThreadSpawner,
        _inputs: Self::BuildInputs,
    ) -> Result<(FeagiWorker<FeagiStdThreadSpawner, Self::ChannelBackend, u32, u32>, Self::BuildOutputs), WorkerStartError<Self::WorkerStartError>> {
        let (requester, responder) = worker_request_channel(self.request_queue_capacity);
        let handle = spawner.spawn_worker(move || doubling_worker(responder))?;
        Ok((FeagiWorker::new(requester, handle), ()))
    }

    fn spawn_related_channel_pairs(&self) -> Self::RequiredChannelPairs {}
}

/// Builds a worker that publishes each custom request onto an outgoing event channel. The
/// channel pair is created first, the worker gets the sender and the caller keeps the receiver
struct EventWorkerBuilder {
    event_queue_capacity: usize,
}

impl FeagiWorkerBuilder<FeagiStdThreadSpawner> for EventWorkerBuilder {
    type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;
    type BuildInputs = thingbuf::mpsc::Sender<Number, RecyclableRecyclePolicy<Number>>;
    type BuildOutputs = ();
    type RequiredChannelPairs = (
        thingbuf::mpsc::Sender<Number, RecyclableRecyclePolicy<Number>>,
        thingbuf::mpsc::Receiver<Number, RecyclableRecyclePolicy<Number>>,
    );
    type WorkerCommandHandler = EventHandler;
    type WorkerStartError = FeagiWorkerSpawnError;

    fn build_worker(
        self,
        spawner: &FeagiStdThreadSpawner,
        event_sender: Self::BuildInputs,
    ) -> Result<(FeagiWorker<FeagiStdThreadSpawner, Self::ChannelBackend, u32, u32>, Self::BuildOutputs), WorkerStartError<Self::WorkerStartError>> {
        let (requester, responder) = worker_request_channel(1);
        let handle = spawner.spawn_worker(move || async move {
            FeagiWorkerAwaitingRouter::new(EventHandler { event_sender }, responder).run_until_stopped().await;
        })?;
        Ok((FeagiWorker::new(requester, handle), ()))
    }

    fn spawn_related_channel_pairs(&self) -> Self::RequiredChannelPairs {
        FeagiChannelAllocBuilder::create_channels(self.event_queue_capacity)
    }
}

/// Builds a worker that exits immediately without answering anything
struct ExitingWorkerBuilder;

impl FeagiWorkerBuilder<FeagiStdThreadSpawner> for ExitingWorkerBuilder {
    type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;
    type BuildInputs = ();
    type BuildOutputs = ();
    type RequiredChannelPairs = ();
    type WorkerCommandHandler = DoublingHandler;
    type WorkerStartError = FeagiWorkerSpawnError;

    fn build_worker(
        self,
        spawner: &FeagiStdThreadSpawner,
        _inputs: Self::BuildInputs,
    ) -> Result<(FeagiWorker<FeagiStdThreadSpawner, Self::ChannelBackend, u32, u32>, Self::BuildOutputs), WorkerStartError<Self::WorkerStartError>> {
        let (requester, responder) = worker_request_channel(1);
        let handle = spawner.spawn_worker(move || async move { drop(responder) })?;
        Ok((FeagiWorker::new(requester, handle), ()))
    }

    fn spawn_related_channel_pairs(&self) -> Self::RequiredChannelPairs {}
}

#[test]
fn built_worker_answers_custom_requests_then_stops() {
    let builder = DoublingWorkerBuilder { request_queue_capacity: 2 };
    builder.spawn_related_channel_pairs();
    let (mut worker, ()) = builder.build_worker(&FeagiStdThreadSpawner, ()).unwrap();

    block_on_thread(async {
        for value in 0..50u32 {
            assert_eq!(worker.make_request(value).await.unwrap(), value * 2);
        }
        worker.stop().await.unwrap();
        // TODO: These are refused by `FeagiWorker` itself to avoid the request / response drain
        // race (see the top of feagi_worker.rs). Once that race is fixed, they should also fail
        // when sent to the exited worker, and can be checked that way
        assert!(matches!(
            worker.make_request(1).await,
            Err(FeagiRequestResponseError::RequestNotSent(FeagiChannelTransmitError::ChannelClosed))
        ));
        assert!(matches!(
            worker.stop().await,
            Err(FeagiRequestResponseError::RequestNotSent(FeagiChannelTransmitError::ChannelClosed))
        ));
    });
    worker.join().unwrap();
}

#[test]
fn built_worker_sends_on_its_required_channel_pair() {
    let builder = EventWorkerBuilder { event_queue_capacity: 4 };
    let (event_sender, event_receiver) = builder.spawn_related_channel_pairs();
    let (mut worker, ()) = builder.build_worker(&FeagiStdThreadSpawner, event_sender).unwrap();

    block_on_thread(async {
        assert_eq!(worker.make_request(3).await.unwrap(), 3);
        assert_eq!(worker.make_request(4).await.unwrap(), 4);
        worker.stop().await.unwrap();
    });
    worker.join().unwrap();
    assert_eq!(FeagiChannelReceiver::try_recv(&event_receiver).unwrap(), Number(3));
    assert_eq!(FeagiChannelReceiver::try_recv(&event_receiver).unwrap(), Number(4));
}

#[test]
fn stopping_an_exited_built_worker_errors() {
    let (mut worker, ()) = ExitingWorkerBuilder.build_worker(&FeagiStdThreadSpawner, ()).unwrap();
    // TODO: Can rarely hang. The worker was never stopped, so `FeagiWorker` cannot refuse the
    // request itself and the request / response drain race applies (see the top of
    // feagi_worker.rs). Fix with the race
    // Depending on timing the request is either rejected outright or drained unanswered when the
    // worker's responder drops. Both must surface as errors rather than hanging
    let result = block_on_thread(worker.stop());
    assert!(matches!(
        result,
        Err(FeagiRequestResponseError::RequestNotSent(_)) | Err(FeagiRequestResponseError::ResponderDroppedRequest)
    ));
}

//endregion
