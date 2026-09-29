
/// A pool that has limited items and that can be accessed by multiple threads at once. Does this
/// with an internal mutex, but the lock time is so short it should not be a practical issue
pub struct BlockingPool<T, const N: usize> (
    std::sync::Mutex<heapless::Vec<T, N>>
);

impl<T, const N: usize> BlockingPool<T, N> {

    /// Creates new pool
    pub fn new(initial: heapless::Vec<T, N>) -> Self {
        Self(std::sync::Mutex::new(initial))
    }

    pub fn take_item(&mut self) -> Option<T> {
        let vec = &mut self.0.lock();
        if let Ok(mg) = vec {
            return mg.pop()
        }
        None // TODO smarter failure handling?
    }

    pub fn return_item(&mut self, item: T) { // TODO error handling
        let vec = &mut self.0.lock();
        if let Ok(mg) = vec {
            _ = mg.push(item) // TODO what if full?
        }
    }
}

// TODO no-std impl version for embassy? We can use types as a proxy and pick the correct one without
// changing the name