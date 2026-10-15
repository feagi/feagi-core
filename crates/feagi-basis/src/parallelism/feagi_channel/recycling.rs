/// Easier way to mark an item itself as recyclable, for use with the `RecyclableRecycler`
pub trait Recyclable {
    fn new_for_recycler() -> Self;

    fn recycle_self(&mut self) {
        // By default, nothing! But override if there is something needed
    }
}

/// Allow recycling Results
impl<T: Recyclable, E: core::error::Error> Recyclable for Result<T, E> {
    fn new_for_recycler() -> Self {
        Ok(T::new_for_recycler())
    }

    fn recycle_self(&mut self) {
        match self {
            Ok(t) => t.recycle_self(),
            Err(e) => {} // skip
        }
    }
}

/// Generic recycler policy for all items implementing `Recyclable'
#[derive(Clone, Debug, Default)]
pub struct RecyclableRecyclePolicy<T>
where
    T: Recyclable,
{
    _p: core::marker::PhantomData<T>,
}

impl<T> RecyclableRecyclePolicy<T>
where
    T: Recyclable,
{
    pub const NEW: Self = RecyclableRecyclePolicy {
        _p: core::marker::PhantomData,
    };

    pub fn new() -> Self {
        Self { _p: Default::default() }
    }
}

impl<T> thingbuf::Recycle<T> for RecyclableRecyclePolicy<T>
where
    T: Recyclable,
{
    fn new_element(&self) -> T {
        T::new_for_recycler()
    }

    fn recycle(&self, element: &mut T) {
        element.recycle_self()
    }
}

unsafe impl<T: Recyclable> Send for RecyclableRecyclePolicy<T> {}
