use core::iter::Iterator;
use core::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct InvalidatableVector<T> {
    data: Vec<Option<T>>,
    free_indices: Vec<usize>,
    len: usize,
}

impl<T> InvalidatableVector<T> {
    /// Creates a new empty InvalidatableVector
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            free_indices: Vec::new(),
            len: 0,
        }
    }

    /// Creates a new InvalidatableVector with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            free_indices: Vec::with_capacity(capacity / 4), // Rough estimate
            len: 0,
        }
    }

    /// Inserts an element, reusing invalid slots or extending the vector
    /// Returns the index where the element was inserted
    pub fn insert(&mut self, value: T) -> usize {
        if let Some(index) = self.free_indices.pop() {
            // Reuse existing invalid slot
            self.data[index] = Some(value);
            self.len += 1;
            index
        } else {
            // Extend the vector
            let index = self.data.len();
            self.data.push(Some(value));
            self.len += 1;
            index
        }
    }

    /// Invalidates (sets to None) the element at the specified index
    pub fn invalidate(&mut self, index: usize) {
        if index < self.data.len() && self.data[index].is_some() {
            self.data[index] = None;
            self.len -= 1;
            self.free_indices.push(index);
        }
    }

    /// Checks if an element at the specified index is valid (not None)
    pub fn is_valid(&self, index: usize) -> bool {
        index < self.data.len() && self.data[index].is_some()
    }

    /// Gets the number of valid elements
    pub fn len(&self) -> usize {
        self.len
    }

    /// Checks if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Gets the total capacity (including invalid slots)
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Clears all elements and resets to empty state
    pub fn clear(&mut self) {
        self.data.clear();
        self.free_indices.clear();
        self.len = 0;
    }

    /// Reserves capacity for at least `additional` more elements
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Reserves the minimum capacity for at least `additional` more elements
    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
    }

    /// Shrinks the capacity of the vector to fit its current size
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
        self.free_indices.shrink_to_fit();
    }

    /// Gets a reference to the element at the specified index (if valid)
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index).and_then(|opt| opt.as_ref())
    }

    /// Gets a mutable reference to the element at the specified index (if valid)
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index).and_then(|opt| opt.as_mut())
    }

    /// Returns an iterator over all valid elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().filter_map(|opt| opt.as_ref())
    }

    /// Returns a mutable iterator over all valid elements
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().filter_map(|opt| opt.as_mut())
    }

    /// Gets the number of total slots (including invalid ones)
    pub fn total_capacity(&self) -> usize {
        self.data.len()
    }
}

impl<T> InvalidatableVector<T> {
    /// Pushes an element to the end (same as insert)
    pub fn push(&mut self, value: T) -> usize {
        self.insert(value)
    }

    /// Removes and returns the last element if it exists
    pub fn pop(&mut self) -> Option<T> {
        while let Some(last_index) = self.data.last().map(|_| self.data.len() - 1) {
            if let Some(value) = self.data[last_index].take() {
                self.len -= 1;
                self.free_indices.push(last_index);
                return Some(value);
            } else {
                // If last element is None, remove it and continue
                self.data.pop();
                self.free_indices.clear(); // Reset because we're changing the structure
            }
        }
        None
    }

    /// Removes all elements, making them invalid
    pub fn clear_all(&mut self) {
        for i in 0..self.data.len() {
            if self.data[i].is_some() {
                self.free_indices.push(i);
            }
            self.data[i] = None;
        }
        self.len = 0;
    }
}


impl<T> Default for InvalidatableVector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<usize> for InvalidatableVector<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Index out of bounds or element invalid")
    }
}

impl<T> IndexMut<usize> for InvalidatableVector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("Index out of bounds or element invalid")
    }
}

// Iterator implementation
impl<T> IntoIterator for InvalidatableVector<T> {
    type Item = T;
    type IntoIter = std::iter::Flatten<std::vec::IntoIter<Option<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter().flatten()
    }
}

impl<'a, T> IntoIterator for &'a InvalidatableVector<T> {
    type Item = &'a T;
    type IntoIter = std::iter::FilterMap<std::slice::Iter<'a, Option<T>>, fn(&'a Option<T>) -> Option<&'a T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter().filter_map(|opt| opt.as_ref())
    }
}