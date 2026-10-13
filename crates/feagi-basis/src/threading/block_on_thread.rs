use core::future::Future;
use core::task::{Context, Poll};

/// Polls `future` on this thread.
///
/// Socket reads and writes block inside `poll`, so the future reaches `Ready` without a reactor.
/// `Pending` yields and polls again; a correct socket impl does not stay there.
pub fn block_on_thread<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    let waker = std::task::Waker::noop();
    let mut context = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}