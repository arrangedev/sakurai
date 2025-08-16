use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Classic ring buffer implementation.
#[repr(align(64))]
pub struct RingBuffer<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Panics if N is not a power of 2 or is 0
    pub const fn new() -> Self {
        assert!(N > 0, "Ring buffer size must be greater than 0");
        assert!(N.is_power_of_two(), "Ring buffer size must be a power of 2");

        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        (head.wrapping_sub(tail)) & (N - 1)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() == N - 1
    }

    /// Pushes an item to the buffer, returning an error if full
    pub fn push(&self, item: T) -> Result<(), RingBufferError> {
        let head = self.head.load(Ordering::Relaxed);
        let next_head = (head + 1) & (N - 1);

        if next_head == self.tail.load(Ordering::Acquire) {
            return Err(RingBufferError::Full);
        }

        unsafe {
            let data_ptr = self.data.as_ptr() as *mut MaybeUninit<T>;
            ptr::write((*data_ptr.add(head)).as_mut_ptr(), item);
        }

        self.head.store(next_head, Ordering::Release);
        Ok(())
    }

    /// Pops an item from the buffer, returning an error if empty
    pub fn pop(&self) -> Result<T, RingBufferError> {
        let tail = self.tail.load(Ordering::Relaxed);

        if tail == self.head.load(Ordering::Acquire) {
            return Err(RingBufferError::Empty);
        }

        let item = unsafe {
            let data_ptr = self.data.as_ptr();
            ptr::read((*data_ptr.add(tail)).as_ptr())
        };
        let next_tail = (tail + 1) & (N - 1);
        self.tail.store(next_tail, Ordering::Release);

        Ok(item)
    }

    #[inline]
    pub fn try_push(&self, item: T) -> Result<(), RingBufferError> {
        self.push(item)
    }

    #[inline]
    pub fn try_pop(&self) -> Result<T, RingBufferError> {
        self.pop()
    }
}

unsafe impl<T: Send, const N: usize> Send for RingBuffer<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for RingBuffer<T, N> {}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        while self.pop().is_ok() {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingBufferError {
    Full,
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = RingBuffer::<i32, 8>::new();
        assert_eq!(buffer.capacity(), 8);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
    }

    #[test]
    fn test_push_pop() {
        let buffer = RingBuffer::<i32, 8>::new();

        assert!(buffer.push(42).is_ok());
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());

        let value = buffer.pop().unwrap();
        assert_eq!(value, 42);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_full_buffer() {
        let buffer = RingBuffer::<i32, 4>::new();
        for i in 0..3 {
            assert!(buffer.push(i).is_ok());
        }

        assert!(buffer.is_full());
        assert!(buffer.push(99).is_err());
    }

    #[test]
    fn test_empty_buffer() {
        let buffer = RingBuffer::<i32, 4>::new();
        assert!(buffer.pop().is_err());
    }

    #[test]
    fn test_wraparound() {
        let buffer = RingBuffer::<i32, 4>::new();
        for cycle in 0..3 {
            for i in 0..3 {
                assert!(buffer.push(cycle * 10 + i).is_ok());
            }

            for i in 0..3 {
                let value = buffer.pop().unwrap();
                assert_eq!(value, cycle * 10 + i);
            }
        }
    }
}
