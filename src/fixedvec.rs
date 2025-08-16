use core::mem::MaybeUninit;
use core::ops::{Index, IndexMut};
use core::ptr;
use core::slice;

/// Similar interface to `Vec`, but with a fixed capacity and inline storage
pub struct FixedVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    pub const fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    #[inline]
    pub const fn remaining_capacity(&self) -> usize {
        N - self.len
    }

    /// Pushes an element to the end of the vector, returning an error if full
    pub fn push(&mut self, value: T) -> Result<(), FixedVecError> {
        if self.len >= N {
            return Err(FixedVecError::Full);
        }

        unsafe {
            ptr::write(self.data[self.len].as_mut_ptr(), value);
        }
        self.len += 1;
        Ok(())
    }

    /// Removes and returns the last element, returning `None` if empty
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { ptr::read(self.data[self.len].as_ptr()) })
        }
    }

    /// Returns a reference to the element at the given index,
    /// returning `None` if out of bounds
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            Some(unsafe { &*self.data[index].as_ptr() })
        } else {
            None
        }
    }

    /// Returns a mutable reference to the element at the given index,
    /// returning `None` if out of bounds
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            Some(unsafe { &mut *self.data[index].as_mut_ptr() })
        } else {
            None
        }
    }

    /// Returns a reference to the first element, returning `None` if empty
    pub fn first(&self) -> Option<&T> {
        self.get(0)
    }

    /// Returns a reference to the last element, returning `None` if empty
    pub fn last(&self) -> Option<&T> {
        if self.len > 0 {
            self.get(self.len - 1)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the first element, returning `None` if empty
    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }

    /// Returns a mutable reference to the last element, returning `None` if empty
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.len > 0 {
            let last_index = self.len - 1;
            self.get_mut(last_index)
        } else {
            None
        }
    }

    /// Inserts an element at the given index, shifting all elements after the index to the right
    /// and returning an error if the vector is full or the index is invalid
    pub fn insert(&mut self, index: usize, value: T) -> Result<(), FixedVecError> {
        if self.len >= N {
            return Err(FixedVecError::Full);
        }
        if index > self.len {
            return Err(FixedVecError::IndexOutOfBounds);
        }

        for i in (index..self.len).rev() {
            unsafe {
                let src = self.data[i].as_ptr();
                let dst = self.data[i + 1].as_mut_ptr();
                ptr::copy_nonoverlapping(src, dst, 1);
            }
        }

        unsafe {
            ptr::write(self.data[index].as_mut_ptr(), value);
        }
        self.len += 1;
        Ok(())
    }

    /// Removes and returns the element at the given index,
    /// shifting all elements after the index to the left and
    /// returning `None` if the index is out of bounds
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }

        let value = unsafe { ptr::read(self.data[index].as_ptr()) };

        // Shift elements to the left
        for i in index..self.len - 1 {
            unsafe {
                let src = self.data[i + 1].as_ptr();
                let dst = self.data[i].as_mut_ptr();
                ptr::copy_nonoverlapping(src, dst, 1);
            }
        }

        self.len -= 1;
        Some(value)
    }

    /// Swaps two elements in the vector, panicking if either index is out of bounds
    pub fn swap(&mut self, a: usize, b: usize) {
        assert!(a < self.len, "Index {} out of bounds", a);
        assert!(b < self.len, "Index {} out of bounds", b);

        if a != b {
            unsafe {
                let pa = self.data[a].as_mut_ptr();
                let pb = self.data[b].as_mut_ptr();
                ptr::swap(pa, pb);
            }
        }
    }

    /// Reverses the order of elements
    pub fn reverse(&mut self) {
        let mut left = 0;
        let mut right = self.len;

        while left < right {
            right -= 1;
            self.swap(left, right);
            left += 1;
        }
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    /// Truncates the vector to the specified length
    ///
    /// If current length is greater than `len`, the vec is truncated
    /// to exactly `len` elements
    pub fn truncate(&mut self, len: usize) {
        while self.len > len {
            self.pop();
        }
    }

    pub fn iter(&self) -> FixedVecIter<'_, T> {
        FixedVecIter {
            data: self.as_slice(),
            index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> FixedVecIterMut<'_, T> {
        FixedVecIterMut {
            data: self.as_mut_slice(),
            index: 0,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, self.len) }
    }

    /// Attempts to push an element, returning the element if full
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.len >= N {
            return Err(value);
        }

        unsafe {
            ptr::write(self.data[self.len].as_mut_ptr(), value);
        }
        self.len += 1;
        Ok(())
    }

    /// Extends the vector with the contents of an iterator, returning
    /// the number of elements that didn't fit.
    pub fn extend_from_iter<I>(&mut self, iter: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        let mut failed_count = 0;
        for item in iter {
            if self.push(item).is_err() {
                failed_count += 1;
            }
        }
        failed_count
    }
}

impl<T, const N: usize> Default for FixedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for FixedVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N: usize> Index<usize> for FixedVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Index out of bounds")
    }
}

impl<T, const N: usize> IndexMut<usize> for FixedVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("Index out of bounds")
    }
}

pub struct FixedVecIter<'a, T> {
    data: &'a [T],
    index: usize,
}

impl<'a, T> Iterator for FixedVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let item = &self.data[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.data.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for FixedVecIter<'a, T> {
    fn len(&self) -> usize {
        self.data.len() - self.index
    }
}

/// Mutable iterator over fixed vector elements
pub struct FixedVecIterMut<'a, T> {
    data: &'a mut [T],
    index: usize,
}

impl<'a, T> Iterator for FixedVecIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let ptr = self.data.as_mut_ptr();
            let item = unsafe { &mut *ptr.add(self.index) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.data.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for FixedVecIterMut<'a, T> {
    fn len(&self) -> usize {
        self.data.len() - self.index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixedVecError {
    Full,
    IndexOutOfBounds,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut vec = FixedVec::<i32, 8>::new();

        assert!(vec.push(42).is_ok());
        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());
        assert_eq!(vec.remaining_capacity(), 7);

        let value = vec.pop().unwrap();
        assert_eq!(value, 42);
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_index() {
        let mut vec = FixedVec::<i32, 8>::new();
        for i in 0..5 {
            vec.push(i).unwrap();
        }
        for i in 0..5 {
            assert_eq!(vec[i], i as i32);
        }
        vec[2] = 99;
        assert_eq!(vec[2], 99);
    }

    #[test]
    fn test_get() {
        let mut vec = FixedVec::<i32, 8>::new();
        vec.push(42).unwrap();
        assert_eq!(vec.get(0), Some(&42));
        assert_eq!(vec.get(1), None);
        *vec.get_mut(0).unwrap() = 99;
        assert_eq!(vec.get(0), Some(&99));
    }

    #[test]
    fn test_first_last() {
        let mut vec = FixedVec::<i32, 8>::new();
        assert_eq!(vec.first(), None);
        assert_eq!(vec.last(), None);
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        assert_eq!(vec.first(), Some(&1));
        assert_eq!(vec.last(), Some(&3));
        *vec.first_mut().unwrap() = 10;
        *vec.last_mut().unwrap() = 30;
        assert_eq!(vec.first(), Some(&10));
        assert_eq!(vec.last(), Some(&30));
    }

    #[test]
    fn test_insert_remove() {
        let mut vec = FixedVec::<i32, 8>::new();
        vec.push(1).unwrap();
        vec.push(3).unwrap();
        vec.insert(1, 2).unwrap();
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
        let removed = vec.remove(1).unwrap();
        assert_eq!(removed, 2);
        assert_eq!(vec.as_slice(), &[1, 3]);
    }

    #[test]
    fn test_swap() {
        let mut vec = FixedVec::<i32, 8>::new();

        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.swap(0, 2);
        assert_eq!(vec.as_slice(), &[3, 2, 1]);
    }

    #[test]
    fn test_reverse() {
        let mut vec = FixedVec::<i32, 8>::new();
        for i in 1..=5 {
            vec.push(i).unwrap();
        }
        vec.reverse();
        assert_eq!(vec.as_slice(), &[5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_clear() {
        let mut vec = FixedVec::<i32, 8>::new();
        for i in 0..5 {
            vec.push(i).unwrap();
        }

        assert_eq!(vec.len(), 5);
        vec.clear();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_truncate() {
        let mut vec = FixedVec::<i32, 8>::new();
        for i in 0..5 {
            vec.push(i).unwrap();
        }

        vec.truncate(3);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[0, 1, 2]);
        vec.truncate(10); // no effect
        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn test_iterator() {
        let mut vec = FixedVec::<i32, 8>::new();
        for i in 0..5 {
            vec.push(i).unwrap();
        }

        let items: [i32; 5] = [0, 1, 2, 3, 4];
        for (i, &expected) in items.iter().enumerate() {
            assert_eq!(*vec.get(i).unwrap(), expected);
        }

        for item in vec.iter_mut() {
            *item *= 2;
        }

        let expected = [0, 2, 4, 6, 8];
        for (i, &exp) in expected.iter().enumerate() {
            assert_eq!(*vec.get(i).unwrap(), exp);
        }
    }

    #[test]
    fn test_overflow() {
        let mut vec = FixedVec::<i32, 2>::new();

        assert!(vec.push(1).is_ok());
        assert!(vec.push(2).is_ok());
        assert!(vec.is_full());
        assert_eq!(vec.push(3), Err(FixedVecError::Full));
    }

    #[test]
    fn test_try_push() {
        let mut vec = FixedVec::<i32, 2>::new();

        assert!(vec.try_push(1).is_ok());
        assert!(vec.try_push(2).is_ok());
        match vec.try_push(3) {
            Err(item) => assert_eq!(item, 3),
            Ok(()) => panic!("Should have failed"),
        }
    }

    #[test]
    fn test_extend_iter() {
        let mut vec = FixedVec::<i32, 5>::new();

        let failed = vec.extend_from_iter(0..3);
        assert_eq!(failed, 0);
        assert_eq!(vec.as_slice(), &[0, 1, 2]);
        let failed = vec.extend_from_iter(3..10);
        assert_eq!(failed, 5); // 5 will fail
        assert_eq!(vec.as_slice(), &[0, 1, 2, 3, 4]);
    }
}
