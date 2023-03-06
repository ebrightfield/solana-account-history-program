use std::{
    fmt::{self, Debug},
    mem::zeroed,
    ops::Index,
};
use std::cmp::min;

/// An array of values that are overwritten in circular FIFO.
///
/// It is a read-only interface except for the [StackVecModulo::push] method,
/// which is how values are added to the collection.
/// See tests for examples on usage.
///
/// There is a companion type [StackVecModuloIterator] that
/// iterates from newest to oldest.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StackVecModulo<T, const N: usize> {
    val: [T; N],
    /// Monotonically increasing index value that increases with every
    /// `self.push` call. During indexing, this number is modulo-divided by `N`
    /// to retrieve a valid index.
    ///
    /// Historically, there are always `self.most_recent_index + 1` total
    /// writes.
    num_push_calls: u64,
}

impl<T, const N: usize> Debug for StackVecModulo<T, N>
where
    T: Debug + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for i in 0..self.len() {
            if i + 1 != self.len() {
                write!(f, "{:?}, ", self.val[i])?;
            } else {
                write!(f, "{:?}", self.val[i])?;
            }
        }
        write!(f, "]")
    }
}

impl<T, const N: usize> Default for StackVecModulo<T, N>
where
    T: Default,
{
    fn default() -> Self {
        let mut ret = Self {
            val: unsafe { zeroed() },
            num_push_calls: 0,
        };

        for i in 0..N {
            ret.val[i as usize] = T::default();
        }

        ret
    }
}

impl<T, const N: usize> StackVecModulo<T, N>
{
    /// The total size of pushed elements, which could be between 0 and N.
    pub fn len(&self) -> usize {
        min(self.num_push_calls(), N)
    }

    /// The intended way to add a new element to this struct.
    pub fn push(&mut self, elem: T) {
        self.val[(self.num_push_calls as usize + 1) % N] = elem;
        self.num_push_calls += 1;
    }

    /// Total number of calls to `self.push`.
    pub fn num_push_calls(&self) -> usize { self.num_push_calls as usize}

    /// Most recently modified index.
    /// Returns zero when there is no data.
    pub fn most_recent_index(&self) -> usize { (self.num_push_calls as usize % N) as usize }

    /// Most recently added value.
    /// Returns a `T::default()` when there is no data.
    pub fn most_recent_entry(&self) -> &T {
        &self.val[self.most_recent_index()]
    }
}

impl<T, const N: usize> Index<usize> for StackVecModulo<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        let index = index % N;

        &self.val[index]
    }
}

/// Iterates from newest value to oldest.
pub struct StackVecModuloIterator<'a, T, const N: usize> {
    val: &'a StackVecModulo<T, N>,
    counter: usize,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for StackVecModuloIterator<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.val.len() {
            let val = &self.val[self.index];
            self.counter += 1;
            self.index = if self.index == 0 { N-1 } else {self.index - 1 };
            Some(val)
        } else {
            None
        }
    }
}

impl<'a, T, const N: usize> From<&'a StackVecModulo<T, N>> for StackVecModuloIterator<'a, T, N> {
    fn from(value: &'a StackVecModulo<T, N>) -> Self {
        let start = value.most_recent_index();
        Self {
            val: &value,
            counter: 0,
            index: start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iteration() {
        let mut vec = StackVecModulo::<u64, 5>::default();
        vec.push(0);
        vec.push(2);
        vec.push(4);
        vec.push(6);
        // len check
        assert_eq!(4, vec.len());

        // Should iterate through 6, 4, 2, 0
        let mut j = 3u64;
        for i in StackVecModuloIterator::from(&vec) {
            assert_eq!(*i, j*2);
            if j > 0 {
                j -= 1;
            }
        }
        // Should not iterate through anything
        let vec = StackVecModulo::<u64, 5>::default();
        let mut j = 0u64;
        for _ in StackVecModuloIterator::from(&vec) {
            j += 1;
        }
        assert_eq!(j, 0);
    }

    #[test]
    fn push() {
        let mut vec = StackVecModulo::<u64, 5>::default();
        // len check
        assert_eq!(0, vec.len());
        assert_eq!(*vec.most_recent_entry(), 0);
        vec.push(0);
        assert_eq!(*vec.most_recent_entry(), 0);
        vec.push(2);
        assert_eq!(*vec.most_recent_entry(), 2);
        vec.push(4);
        vec.push(6);
        vec.push(8);
        assert_eq!(*vec.most_recent_entry(), 8);
        vec.push(10);
        assert_eq!(*vec.most_recent_entry(), 10);
        vec.push(12);
        // len check should now return 5
        assert_eq!(5, vec.len());

        // Should always return last pushed value
        assert_eq!(*vec.most_recent_entry(), 12);
    }
}