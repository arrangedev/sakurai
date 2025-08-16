use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;
use core::ptr;

/// HashMap implementation designed to be cache-friendly,
/// using open addressing and linear probing.
///
/// Capacity must be a power of 2.
pub struct HashMap<K, V, const N: usize> {
    buckets: [MaybeUninit<Bucket<K, V>>; N],
    len: usize,
}

impl<K, V, const N: usize> HashMap<K, V, N>
where
    K: Hash + PartialEq,
{
    /// Panics if `N` is not a power of 2 (or is 0)
    pub const fn new() -> Self {
        assert!(N > 0, "HashMap size must be greater than 0");
        assert!(N.is_power_of_two(), "HashMap size must be a power of 2");

        Self {
            buckets: unsafe { MaybeUninit::uninit().assume_init() },
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
        self.len >= N * 3 / 4
    }

    #[inline]
    pub fn load_factor(&self) -> f32 {
        self.len as f32 / N as f32
    }

    /// Inserts a key-value pair into the map, returning the old value if the key already exists
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, HashMapError> {
        if self.is_full() {
            return Err(HashMapError::Full);
        }

        let (index, found) = self.find_bucket(&key);
        let bucket = unsafe { &mut *self.buckets[index].as_mut_ptr() };

        if found {
            let old_value = unsafe { ptr::read(bucket.value.as_ptr()) };
            unsafe {
                ptr::write(bucket.value.as_mut_ptr(), value);
            }
            Ok(Some(old_value))
        } else {
            unsafe {
                ptr::write(bucket.key.as_mut_ptr(), key);
                ptr::write(bucket.value.as_mut_ptr(), value);
            }
            bucket.state = BucketState::Occupied;
            self.len += 1;
            Ok(None)
        }
    }

    /// Get a reference to a value for a given key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.find_bucket_ro(key).map(|index| {
            let bucket = unsafe { &*self.buckets[index].as_ptr() };
            unsafe { &*bucket.value.as_ptr() }
        })
    }

    /// Get a mut reference to a value for a given key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.find_bucket_ro(key).map(|index| {
            let bucket = unsafe { &mut *self.buckets[index].as_mut_ptr() };
            unsafe { &mut *bucket.value.as_mut_ptr() }
        })
    }

    /// Remove a key-value pair from the map, returning the value if the key was present.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.find_bucket_ro(key).map(|index| {
            let bucket = unsafe { &mut *self.buckets[index].as_mut_ptr() };

            let value = unsafe { ptr::read(bucket.value.as_ptr()) };
            unsafe {
                ptr::drop_in_place(bucket.key.as_mut_ptr());
            }

            bucket.state = BucketState::Deleted;
            self.len -= 1;

            value
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn clear(&mut self) {
        for i in 0..N {
            let bucket = unsafe { &mut *self.buckets[i].as_mut_ptr() };
            if bucket.is_occupied() {
                unsafe {
                    ptr::drop_in_place(bucket.key.as_mut_ptr());
                    ptr::drop_in_place(bucket.value.as_mut_ptr());
                }
            }
            bucket.state = BucketState::Empty;
        }
        self.len = 0;
    }

    pub fn iter(&self) -> HashMapIter<'_, K, V, N> {
        HashMapIter {
            map: self,
            index: 0,
        }
    }

    fn hash_key(&self, key: &K) -> usize {
        let mut hasher = Fnv1aHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) & (N - 1)
    }

    fn find_bucket(&self, key: &K) -> (usize, bool) {
        let mut index = self.hash_key(key);

        loop {
            let bucket = unsafe { &*self.buckets[index].as_ptr() };

            match bucket.state {
                BucketState::Empty => return (index, false),
                BucketState::Occupied => {
                    let bucket_key = unsafe { &*bucket.key.as_ptr() };
                    if bucket_key == key {
                        return (index, true);
                    }
                }
                BucketState::Deleted => {}
            }

            index = (index + 1) & (N - 1);
        }
    }

    fn find_bucket_ro(&self, key: &K) -> Option<usize> {
        let mut index = self.hash_key(key);

        loop {
            let bucket = unsafe { &*self.buckets[index].as_ptr() };

            match bucket.state {
                BucketState::Empty => return None,
                BucketState::Occupied => {
                    let bucket_key = unsafe { &*bucket.key.as_ptr() };
                    if bucket_key == key {
                        return Some(index);
                    }
                }
                BucketState::Deleted => {}
            }

            index = (index + 1) & (N - 1);

            if index == self.hash_key(key) {
                return None;
            }
        }
    }
}

impl<K, V, const N: usize> Default for HashMap<K, V, N>
where
    K: Hash + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const N: usize> Drop for HashMap<K, V, N> {
    fn drop(&mut self) {
        for i in 0..N {
            let bucket = unsafe { &mut *self.buckets[i].as_mut_ptr() };
            if bucket.is_occupied() {
                unsafe {
                    ptr::drop_in_place(bucket.key.as_mut_ptr());
                    ptr::drop_in_place(bucket.value.as_mut_ptr());
                }
            }
            bucket.state = BucketState::Empty;
        }
    }
}

pub struct HashMapIter<'a, K, V, const N: usize> {
    map: &'a HashMap<K, V, N>,
    index: usize,
}

impl<'a, K, V, const N: usize> Iterator for HashMapIter<'a, K, V, N> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < N {
            let bucket = unsafe { &*self.map.buckets[self.index].as_ptr() };
            self.index += 1;

            if bucket.is_occupied() {
                let key = unsafe { &*bucket.key.as_ptr() };
                let value = unsafe { &*bucket.value.as_ptr() };
                return Some((key, value));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.map.len, Some(self.map.len))
    }
}

#[derive(Clone, Copy)]
enum BucketState {
    Empty,
    Occupied,
    Deleted,
}

struct Bucket<K, V> {
    state: BucketState,
    key: MaybeUninit<K>,
    value: MaybeUninit<V>,
}

impl<K, V> Bucket<K, V> {
    #[allow(unused)]
    const fn new() -> Self {
        Self {
            state: BucketState::Empty,
            key: MaybeUninit::uninit(),
            value: MaybeUninit::uninit(),
        }
    }

    #[inline]
    fn is_occupied(&self) -> bool {
        matches!(self.state, BucketState::Occupied)
    }
}

/// A dead-simple (and fast, of course) hash function based on FNV-1a
struct Fnv1aHasher {
    state: u64,
}

impl Fnv1aHasher {
    const fn new() -> Self {
        Self {
            state: 0xcbf29ce484222325,
        }
    }
}

impl Hasher for Fnv1aHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        const FNV_PRIME: u64 = 0x100000001b3;
        for &byte in bytes {
            self.state ^= byte as u64;
            self.state = self.state.wrapping_mul(FNV_PRIME);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashMapError {
    Full,
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::format;
    use std::string::{String, ToString};

    #[test]
    fn test_access() {
        let mut map = HashMap::<u32, String, 8>::new();

        assert!(map.insert(42, "hello".to_string()).unwrap().is_none());
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());

        assert_eq!(map.get(&42), Some(&"hello".to_string()));
        assert_eq!(map.get(&99), None);
    }

    #[test]
    fn test_replace() {
        let mut map = HashMap::<u32, String, 8>::new();

        map.insert(42, "hello".to_string()).unwrap();
        let old_value = map.insert(42, "world".to_string()).unwrap();

        assert_eq!(old_value, Some("hello".to_string()));
        assert_eq!(map.get(&42), Some(&"world".to_string()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut map = HashMap::<u32, String, 8>::new();

        map.insert(42, "hello".to_string()).unwrap();
        assert_eq!(map.len(), 1);

        let removed = map.remove(&42);
        assert_eq!(removed, Some("hello".to_string()));
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        assert_eq!(map.get(&42), None);
    }

    #[test]
    fn test_contains() {
        let mut map = HashMap::<u32, String, 8>::new();

        assert!(!map.contains_key(&42));

        map.insert(42, "hello".to_string()).unwrap();
        assert!(map.contains_key(&42));
        assert!(!map.contains_key(&99));
    }

    #[test]
    fn test_getmut() {
        let mut map = HashMap::<u32, String, 8>::new();

        map.insert(42, "hello".to_string()).unwrap();

        if let Some(value) = map.get_mut(&42) {
            value.push_str(" world");
        }

        assert_eq!(map.get(&42), Some(&"hello world".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut map = HashMap::<u32, String, 8>::new();

        for i in 0..5 {
            map.insert(i, format!("value{}", i)).unwrap();
        }

        assert_eq!(map.len(), 5);
        map.clear();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_collisions() {
        let mut map = HashMap::<u32, String, 8>::new();
        for i in 0..6 {
            map.insert(i, format!("value{}", i)).unwrap();
        }
        for i in 0..6 {
            assert_eq!(map.get(&i), Some(&std::format!("value{}", i)));
        }
    }

    #[test]
    fn test_iter() {
        let mut map = HashMap::<u32, String, 8>::new();

        for i in 0..5 {
            map.insert(i, std::format!("value{}", i)).unwrap();
        }

        let mut pairs: std::vec::Vec<_> = map.iter().collect();
        pairs.sort_by_key(|(k, _)| *k);

        for (i, (key, value)) in pairs.iter().enumerate() {
            assert_eq!(**key, i as u32);
            assert_eq!(*value, &std::format!("value{}", i));
        }
    }

    #[test]
    fn test_load_factor() {
        let mut map = HashMap::<u32, String, 8>::new();

        assert_eq!(map.load_factor(), 0.0);

        map.insert(1, "one".to_string()).unwrap();
        assert_eq!(map.load_factor(), 0.125);

        map.insert(2, "two".to_string()).unwrap();
        assert_eq!(map.load_factor(), 0.25);
    }
}
