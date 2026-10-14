use core::future::Future;
use core::task::{Context, Poll};

/// Wakes the thread running `block_on_thread` when its future can make progress.
struct ThreadUnparker(std::thread::Thread);

impl std::task::Wake for ThreadUnparker {
    fn wake(self: std::sync::Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        self.0.unpark();
    }
}

/// Polls `future` to completion on this thread, parking the thread whenever it is `Pending`
/// until the future's waker fires.
///
/// Futures that block inside `poll` (such as blocking socket reads) simply complete without
/// parking. Spurious unparks only cause an extra poll.
pub fn block_on_thread<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    let waker = std::task::Waker::from(std::sync::Arc::new(ThreadUnparker(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::park(),
        }
    }
}
