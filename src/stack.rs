use core::mem::MaybeUninit;
use core::ptr;

/// Inline zero-allocation stack implementation.
pub struct Stack<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> Stack<T, N> {
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

    /// Pushes an item onto the stack, returning an error if full
    pub fn push(&mut self, item: T) -> Result<(), StackError> {
        if self.len >= N {
            return Err(StackError::Overflow);
        }

        unsafe {
            ptr::write(self.data[self.len].as_mut_ptr(), item);
        }
        self.len += 1;
        Ok(())
    }

    /// Pops an item from the stack, returning an error if empty
    pub fn pop(&mut self) -> Result<T, StackError> {
        if self.len == 0 {
            return Err(StackError::Underflow);
        }

        self.len -= 1;
        let item = unsafe { ptr::read(self.data[self.len].as_ptr()) };
        Ok(item)
    }

    /// Returns a reference to the top item without removing it, returning `None` if empty
    pub fn peek(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            Some(unsafe { &*self.data[self.len - 1].as_ptr() })
        }
    }

    /// Returns a mutable reference to the top item without removing it, returning `None` if empty
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            Some(unsafe { &mut *self.data[self.len - 1].as_mut_ptr() })
        }
    }

    pub fn clear(&mut self) {
        while self.pop().is_ok() {}
    }

    pub fn iter(&self) -> StackIter<'_, T> {
        StackIter {
            data: &self.data[..self.len],
            index: self.len,
        }
    }

    pub fn try_push(&mut self, item: T) -> Result<(), T> {
        if self.len >= N {
            return Err(item);
        }

        unsafe {
            ptr::write(self.data[self.len].as_mut_ptr(), item);
        }
        self.len += 1;
        Ok(())
    }
}

impl<T, const N: usize> Default for Stack<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for Stack<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct StackIter<'a, T> {
    data: &'a [MaybeUninit<T>],
    index: usize,
}

impl<'a, T> Iterator for StackIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            None
        } else {
            self.index -= 1;
            Some(unsafe { &*self.data[self.index].as_ptr() })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.index, Some(self.index))
    }
}

impl<'a, T> ExactSizeIterator for StackIter<'a, T> {
    fn len(&self) -> usize {
        self.index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackError {
    Overflow,
    Underflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack() {
        let stack = Stack::<i32, 8>::new();
        assert_eq!(stack.capacity(), 8);
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
        assert!(!stack.is_full());
        assert_eq!(stack.remaining_capacity(), 8);
    }

    #[test]
    fn test_push_pop() {
        let mut stack = Stack::<i32, 8>::new();
        assert!(stack.push(42).is_ok());
        assert_eq!(stack.len(), 1);
        assert!(!stack.is_empty());
        assert_eq!(stack.remaining_capacity(), 7);

        let value = stack.pop().unwrap();
        assert_eq!(value, 42);
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_lifo_order() {
        let mut stack = Stack::<i32, 8>::new();
        for i in 0..5 {
            stack.push(i).unwrap();
        }

        for i in (0..5).rev() {
            assert_eq!(stack.pop().unwrap(), i);
        }
    }

    #[test]
    fn test_overflow() {
        let mut stack = Stack::<i32, 2>::new();
        assert!(stack.push(1).is_ok());
        assert!(stack.push(2).is_ok());
        assert!(stack.is_full());
        assert_eq!(stack.push(3), Err(StackError::Overflow));
    }

    #[test]
    fn test_underflow() {
        let mut stack = Stack::<i32, 2>::new();
        assert_eq!(stack.pop(), Err(StackError::Underflow));
    }

    #[test]
    fn test_peek() {
        let mut stack = Stack::<i32, 8>::new();
        assert!(stack.peek().is_none());
        stack.push(42).unwrap();
        assert_eq!(stack.peek(), Some(&42));
        assert_eq!(stack.len(), 1);

        stack.push(84).unwrap();
        assert_eq!(stack.peek(), Some(&84));
    }

    #[test]
    fn test_peek_mut() {
        let mut stack = Stack::<i32, 8>::new();
        stack.push(42).unwrap();
        if let Some(top) = stack.peek_mut() {
            *top = 99;
        }

        assert_eq!(stack.pop().unwrap(), 99);
    }

    #[test]
    fn test_clear() {
        let mut stack = Stack::<i32, 8>::new();
        for i in 0..5 {
            stack.push(i).unwrap();
        }

        assert_eq!(stack.len(), 5);
        stack.clear();
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_iterator() {
        let mut stack = Stack::<i32, 8>::new();
        for i in 0..5 {
            stack.push(i).unwrap();
        }

        let expected = [4, 3, 2, 1, 0];
        for (i, &exp) in expected.iter().enumerate() {
            assert_eq!(*stack.iter().nth(i).unwrap(), exp);
        }
    }

    #[test]
    fn test_try_push() {
        let mut stack = Stack::<i32, 2>::new();
        assert!(stack.try_push(1).is_ok());
        assert!(stack.try_push(2).is_ok());
        match stack.try_push(3) {
            Err(item) => assert_eq!(item, 3),
            Ok(()) => panic!("Should have failed"),
        }
    }
}
