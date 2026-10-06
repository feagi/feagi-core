/// A pool that has limited items and that can be accessed by multiple threads at once. Does this
/// with an internal mutex, but the lock time is so short it should not be a practical issue.
/// Last In First Out (Stack)
pub struct BlockingPool<T, const N: usize> (
    std::sync::Mutex<heapless::Vec<T, N>>
);

impl<T, const N: usize> BlockingPool<T, N> {

    /// Creates new pool
    pub const fn new(initial: heapless::Vec<T, N>) -> Self {
        Self(std::sync::Mutex::new(initial))
    }
    
    /// Briefly locks the internal to try to remove an item. Returns none if empty
    pub fn take_item(&mut self) -> Option<T> {
        let vec = &mut self.0.lock();
        if let Ok(mg) = vec {
            return mg.pop()
        }
        // given we relock the mutex instantly without control of whatever thread is using this,
        // Any mutex poisoning implies something has already gone catastrophically wrong anyways
        panic!("Blocking Pool Mutex has become poisoned!");
    }

    /// Returns the item by locking the internal and pushing it back in.
    pub fn return_item(&mut self, item: T) -> Result<(), T> {
        let vec = &mut self.0.lock();
        if let Ok(mg) = vec {
            return mg.push(item)
        }
        // given we relock the mutex instantly without control of whatever thread is using this,
        // Any mutex poisoning implies something has already gone catastrophically wrong anyways
        panic!("Blocking Pool Mutex has become poisoned!");
    }
}

unsafe impl<T, const N: usize> Send for BlockingPool<T, N> where T: Send {}

impl<T, const N: usize> Clone for BlockingPool<T, N>
where T: Clone
{
    fn clone(&self) -> Self {
        let vec = self.0.lock().unwrap();
        Self {
            0: std::sync::Mutex::new(vec.clone()),
        }
    }
}

// TODO no-std impl version for embassy? We can use types as a proxy and pick the correct one without
// changing the name
