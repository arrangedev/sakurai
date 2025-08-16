use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Fixed capacity queue for single producer single consumer concurrent operations
/// without locks.
#[repr(align(64))]
pub struct Queue<T, const N: usize> {
    data: [UnsafeCell<MaybeUninit<T>>; N],
    head: AtomicUsize,
    tail: AtomicUsize,
}

pub struct Producer<'a, T, const N: usize> {
    queue: &'a Queue<T, N>,
}

pub struct Consumer<'a, T, const N: usize> {
    queue: &'a Queue<T, N>,
}

impl<T, const N: usize> Queue<T, N> {
    /// Panics if N is not a power of 2 or is 0
    pub const fn new() -> Self {
        assert!(N > 0, "Queue size must be greater than 0");
        assert!(N.is_power_of_two(), "Queue size must be a power of 2");

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
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head == tail
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        ((head + 1) & (N - 1)) == tail
    }

    pub fn split(&self) -> (Producer<'_, T, N>, Consumer<'_, T, N>) {
        (Producer { queue: self }, Consumer { queue: self })
    }
}

impl<'a, T, const N: usize> Producer<'a, T, N> {
    /// Pushes an item to the queue, returning an error if full
    pub fn push(&mut self, item: T) -> Result<(), QueueError> {
        let head = self.queue.head.load(Ordering::Relaxed);
        let next_head = (head + 1) & (N - 1);

        if next_head == self.queue.tail.load(Ordering::Acquire) {
            return Err(QueueError::Full);
        }

        unsafe {
            let slot = &mut *self.queue.data[head].get();
            ptr::write(slot.as_mut_ptr(), item);
        }

        self.queue.head.store(next_head, Ordering::Release);
        Ok(())
    }

    /// Attempts to push an item, returning the item back if the queue is full
    pub fn try_push(&mut self, item: T) -> Result<(), T> {
        let head = self.queue.head.load(Ordering::Relaxed);
        let next_head = (head + 1) & (N - 1);

        if next_head == self.queue.tail.load(Ordering::Acquire) {
            return Err(item);
        }

        unsafe {
            let slot = &mut *self.queue.data[head].get();
            ptr::write(slot.as_mut_ptr(), item);
        }

        self.queue.head.store(next_head, Ordering::Release);
        Ok(())
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

impl<'a, T, const N: usize> Consumer<'a, T, N> {
    /// Pops an item from the queue, returning an error if empty
    pub fn pop(&mut self) -> Result<T, QueueError> {
        let tail = self.queue.tail.load(Ordering::Relaxed);

        if tail == self.queue.head.load(Ordering::Acquire) {
            return Err(QueueError::Empty);
        }

        let item = unsafe {
            let slot = &*self.queue.data[tail].get();
            ptr::read(slot.as_ptr())
        };

        let next_tail = (tail + 1) & (N - 1);
        self.queue.tail.store(next_tail, Ordering::Release);

        Ok(item)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

unsafe impl<T: Send, const N: usize> Send for Queue<T, N> {}
unsafe impl<T: Send, const N: usize> Sync for Queue<T, N> {}

unsafe impl<T: Send, const N: usize> Send for Producer<'_, T, N> {}
unsafe impl<T: Send, const N: usize> Send for Consumer<'_, T, N> {}

impl<T, const N: usize> Drop for Queue<T, N> {
    fn drop(&mut self) {
        let mut consumer = Consumer { queue: self };
        while consumer.pop().is_ok() {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueError {
    Full,
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_queue() {
        let queue = Queue::<i32, 8>::new();
        assert_eq!(queue.capacity(), 8);
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
        assert!(!queue.is_full());
    }

    #[test]
    fn test_push_pop() {
        let queue = Queue::<i32, 8>::new();
        let (mut producer, mut consumer) = queue.split();

        assert!(producer.push(42).is_ok());
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        let value = consumer.pop().unwrap();
        assert_eq!(value, 42);
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_fifo_order() {
        let queue = Queue::<i32, 8>::new();
        let (mut producer, mut consumer) = queue.split();

        for i in 0..5 {
            producer.push(i).unwrap();
        }

        for i in 0..5 {
            assert_eq!(consumer.pop().unwrap(), i);
        }
    }

    #[test]
    fn test_full_queue() {
        let queue = Queue::<i32, 4>::new();
        let (mut producer, _consumer) = queue.split();

        for i in 0..3 {
            assert!(producer.push(i).is_ok());
        }

        assert!(queue.is_full());
        assert_eq!(producer.push(99), Err(QueueError::Full));
    }

    #[test]
    fn test_empty_queue() {
        let queue = Queue::<i32, 4>::new();
        let (_producer, mut consumer) = queue.split();

        assert_eq!(consumer.pop(), Err(QueueError::Empty));
    }

    #[test]
    fn test_wraparound() {
        let queue = Queue::<i32, 4>::new();
        let (mut producer, mut consumer) = queue.split();

        for cycle in 0..3 {
            for i in 0..3 {
                assert!(producer.push(cycle * 10 + i).is_ok());
            }

            for i in 0..3 {
                let value = consumer.pop().unwrap();
                assert_eq!(value, cycle * 10 + i);
            }
        }
    }

    #[test]
    fn test_try_push() {
        let queue = Queue::<i32, 4>::new();
        let (mut producer, _consumer) = queue.split();

        for i in 0..3 {
            assert!(producer.try_push(i).is_ok());
        }

        match producer.try_push(99) {
            Err(item) => assert_eq!(item, 99),
            Ok(()) => panic!("Should have failed"),
        }
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;
        use std::vec::Vec;

        let queue = Arc::new(Queue::<i32, 1024>::new());
        let queue_clone = queue.clone();

        let producer_handle = thread::spawn(move || {
            let (mut producer, _) = queue_clone.split();
            for i in 0..1000 {
                while producer.push(i).is_err() {
                    thread::yield_now();
                }
            }
        });

        let consumer_handle = thread::spawn(move || {
            let (_, mut consumer) = queue.split();
            let mut received = Vec::new();

            while received.len() < 1000 {
                match consumer.pop() {
                    Ok(value) => received.push(value),
                    Err(_) => thread::yield_now(),
                }
            }
            received
        });

        producer_handle.join().unwrap();
        let received = consumer_handle.join().unwrap();

        for (i, &value) in received.iter().enumerate() {
            assert_eq!(value, i as i32);
        }
    }
}
