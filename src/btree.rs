use core::mem::MaybeUninit;
use core::ptr;

use crate::unlikely;

#[cfg(test)]
extern crate std;

type NodeIndex = usize;
const MAX_ORDER: usize = 128;

/// B+ Tree Implementation
pub struct BTree<K, V, const ORDER: usize> {
    root: Option<NodeIndex>,
    nodes: [MaybeUninit<Node<K, V>>; ORDER],
    free_list: [bool; ORDER],
    next_free: usize,
    len: usize,
}

#[repr(align(64))]
struct Node<K, V> {
    keys: [MaybeUninit<K>; MAX_ORDER],
    values: [MaybeUninit<V>; MAX_ORDER],          // leaf nodes
    children: [Option<NodeIndex>; MAX_ORDER + 1], // internal nodes
    next_leaf: Option<NodeIndex>,                 // leaf nodes - seq access
    key_count: usize,
    is_leaf: bool,
}

impl<K, V, const ORDER: usize> BTree<K, V, ORDER>
where
    K: Ord + Copy,
    V: Clone,
{
    /// default order of 8
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: unsafe { MaybeUninit::uninit().assume_init() },
            free_list: [true; ORDER],
            next_free: 0,
            len: 0,
        }
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
    pub fn capacity(&self) -> usize {
        ORDER * ORDER * ORDER * ORDER // approx
    }

    fn allocate_node(&mut self) -> Result<NodeIndex, BTreeError> {
        let mut index = self.next_free;
        let mut found = false;

        for i in 0..64 {
            let current_index = (self.next_free + i) % 64;
            let is_free = self.free_list[current_index];
            index = (current_index & (is_free as usize).wrapping_sub(1))
                | (index & ((!is_free) as usize).wrapping_sub(1));
            found |= is_free;

            if found {
                break;
            }
        }

        if !found {
            return Err(BTreeError::Full);
        }

        self.free_list[index] = false;
        self.next_free = (index + 1) % 64;
        unsafe {
            let node = &mut *self.nodes[index].as_mut_ptr();
            ptr::write(
                node,
                Node {
                    keys: MaybeUninit::uninit().assume_init(),
                    values: MaybeUninit::uninit().assume_init(),
                    children: [None; MAX_ORDER + 1],
                    next_leaf: None,
                    key_count: 0,
                    is_leaf: true,
                },
            );
        }

        Ok(index)
    }

    fn deallocate_node(&mut self, index: NodeIndex) {
        if index < 64 {
            unsafe {
                let node = &mut *self.nodes[index].as_mut_ptr();
                for i in 0..node.key_count {
                    if node.is_leaf {
                        ptr::drop_in_place(node.values[i].as_mut_ptr());
                    }
                    ptr::drop_in_place(node.keys[i].as_mut_ptr());
                }
            }
            self.free_list[index] = true;
        }
    }

    /// retuns (found, position) -- position is where key should be
    fn search_node(&self, node_index: NodeIndex, key: &K) -> (bool, usize) {
        let node = unsafe { &*self.nodes[node_index].as_ptr() };

        let mut left = 0;
        let mut right = node.key_count;
        while left < right {
            let mid = (left + right) >> 1;
            let node_key = unsafe { &*node.keys[mid].as_ptr() };

            // cmp: -1, 0, or 1
            let cmp = key.cmp(node_key) as i8;
            let is_less = (cmp < 0) as usize;
            let is_greater = (cmp > 0) as usize;

            if cmp == 0 {
                return (true, mid);
            }

            // less -- right = mid, left unchanged
            // greater -- left = mid + 1, right unchanged
            right = mid * is_less + right * (1 - is_less);
            left = (mid + 1) * is_greater + left * (1 - is_greater);
        }

        let found = if unlikely!(left < node.key_count) {
            let node_key = unsafe { &*node.keys[left].as_ptr() };
            key == node_key
        } else {
            false
        };

        (found, left)
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, BTreeError> {
        if self.root.is_none() {
            let root_index = self.allocate_node()?;
            self.root = Some(root_index);

            let root = unsafe { &mut *self.nodes[root_index].as_mut_ptr() };
            unsafe {
                ptr::write(root.keys[0].as_mut_ptr(), key);
                ptr::write(root.values[0].as_mut_ptr(), value);
            }
            root.key_count = 1;
            self.len += 1;
            return Ok(None);
        }

        let root_index = self.root.unwrap();
        self.insert_recursive(root_index, key, value)
    }

    fn insert_recursive(
        &mut self,
        node_index: NodeIndex,
        key: K,
        value: V,
    ) -> Result<Option<V>, BTreeError> {
        let node = unsafe { &mut *self.nodes[node_index].as_mut_ptr() };
        let (found, pos) = self.search_node(node_index, &key);

        if node.is_leaf {
            if found {
                let old_value = unsafe { ptr::read(node.values[pos].as_ptr()) };
                unsafe {
                    ptr::write(node.values[pos].as_mut_ptr(), value);
                }
                return Ok(Some(old_value));
            }

            if node.key_count >= ORDER {
                return Err(BTreeError::Full);
            }

            for i in (pos..node.key_count).rev() {
                unsafe {
                    let src_key = ptr::read(node.keys[i].as_ptr());
                    let src_value = ptr::read(node.values[i].as_ptr());
                    ptr::write(node.keys[i + 1].as_mut_ptr(), src_key);
                    ptr::write(node.values[i + 1].as_mut_ptr(), src_value);
                }
            }

            unsafe {
                ptr::write(node.keys[pos].as_mut_ptr(), key);
                ptr::write(node.values[pos].as_mut_ptr(), value);
            }
            node.key_count += 1;
            self.len += 1;
            Ok(None)
        } else {
            let child_index = node.children[pos + found as usize].unwrap();
            self.insert_recursive(child_index, key, value)
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let root_index = self.root?;
        self.get_recursive(root_index, key)
    }

    fn get_recursive(&self, node_index: NodeIndex, key: &K) -> Option<&V> {
        let node = unsafe { &*self.nodes[node_index].as_ptr() };
        let (found, pos) = self.search_node(node_index, key);

        if node.is_leaf {
            if found {
                Some(unsafe { &*node.values[pos].as_ptr() })
            } else {
                None
            }
        } else {
            let child_index = node.children[pos + found as usize]?;
            self.get_recursive(child_index, key)
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let root_index = self.root?;
        let result = self.remove_recursive(root_index, key);
        self.len = self.len.saturating_sub(result.is_some() as usize);
        result
    }

    fn remove_recursive(&mut self, node_index: NodeIndex, key: &K) -> Option<V> {
        let node = unsafe { &mut *self.nodes[node_index].as_mut_ptr() };
        let (found, pos) = self.search_node(node_index, key);

        if node.is_leaf {
            if !found {
                return None;
            }

            let removed_value = unsafe { ptr::read(node.values[pos].as_ptr()) };
            for i in pos..node.key_count - 1 {
                unsafe {
                    let src_key = ptr::read(node.keys[i + 1].as_ptr());
                    let src_value = ptr::read(node.values[i + 1].as_ptr());
                    ptr::write(node.keys[i].as_mut_ptr(), src_key);
                    ptr::write(node.values[i].as_mut_ptr(), src_value);
                }
            }

            node.key_count -= 1;
            Some(removed_value)
        } else {
            let child_index = node.children[pos + found as usize]?;
            self.remove_recursive(child_index, key)
        }
    }

    pub fn iter(&self) -> BTreeIter<'_, K, V, ORDER> {
        BTreeIter {
            tree: self,
            current_node: self.find_leftmost_leaf(),
            current_pos: 0,
        }
    }

    fn find_leftmost_leaf(&self) -> Option<NodeIndex> {
        let mut current = self.root?;

        loop {
            let node = unsafe { &*self.nodes[current].as_ptr() };
            if node.is_leaf {
                return Some(current);
            }
            current = node.children[0]?;
        }
    }

    pub fn clear(&mut self) {
        if let Some(root) = self.root {
            self.clear_recursive(root);
            self.root = None;
            self.len = 0;
            self.free_list = [true; ORDER];
            self.next_free = 0;
        }
    }

    fn clear_recursive(&mut self, node_index: NodeIndex) {
        let node = unsafe { &*self.nodes[node_index].as_ptr() };

        if !node.is_leaf {
            for i in 0..=node.key_count {
                if let Some(child) = node.children[i] {
                    self.clear_recursive(child);
                }
            }
        }

        self.deallocate_node(node_index);
    }
}

impl<K, V, const ORDER: usize> Default for BTree<K, V, ORDER>
where
    K: Ord + Copy,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const ORDER: usize> BTree<K, V, ORDER> {
    fn drop_recursive(&mut self, node_index: NodeIndex) {
        if node_index < 64 {
            unsafe {
                let node = &mut *self.nodes[node_index].as_mut_ptr();
                for i in 0..node.key_count {
                    if node.is_leaf {
                        ptr::drop_in_place(node.values[i].as_mut_ptr());
                    }
                    ptr::drop_in_place(node.keys[i].as_mut_ptr());
                }

                if !node.is_leaf {
                    for i in 0..=node.key_count {
                        if let Some(child) = node.children[i] {
                            self.drop_recursive(child);
                        }
                    }
                }
            }
        }
    }
}

impl<K, V, const ORDER: usize> Drop for BTree<K, V, ORDER> {
    fn drop(&mut self) {
        if let Some(root) = self.root {
            self.drop_recursive(root);
        }
    }
}

pub struct BTreeIter<'a, K, V, const ORDER: usize> {
    tree: &'a BTree<K, V, ORDER>,
    current_node: Option<NodeIndex>,
    current_pos: usize,
}

impl<'a, K, V, const ORDER: usize> Iterator for BTreeIter<'a, K, V, ORDER> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let node_index = self.current_node?;
        let node = unsafe { &*self.tree.nodes[node_index].as_ptr() };
        if self.current_pos >= node.key_count {
            self.current_node = node.next_leaf;
            self.current_pos = 0;

            let next_node_index = self.current_node?;
            let next_node = unsafe { &*self.tree.nodes[next_node_index].as_ptr() };
            if next_node.key_count == 0 {
                return None;
            }

            let key = unsafe { &*next_node.keys[0].as_ptr() };
            let value = unsafe { &*next_node.values[0].as_ptr() };
            self.current_pos = 1;
            return Some((key, value));
        }

        let key = unsafe { &*node.keys[self.current_pos].as_ptr() };
        let value = unsafe { &*node.values[self.current_pos].as_ptr() };
        self.current_pos += 1;

        Some((key, value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.tree.len))
    }
}

unsafe impl<K, V, const ORDER: usize> Send for BTree<K, V, ORDER>
where
    K: Send,
    V: Send,
{
}

unsafe impl<K, V, const ORDER: usize> Sync for BTree<K, V, ORDER>
where
    K: Sync,
    V: Sync,
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BTreeError {
    Full,
    NotFound,
    InvalidOperation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[test]
    fn test_insert_get() {
        let mut tree = BTree::<u32, i32, 8>::new();

        assert!(tree.insert(42, 100).unwrap().is_none());
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());

        assert_eq!(tree.get(&42), Some(&100));
        assert_eq!(tree.get(&99), None);
    }

    #[test]
    fn test_insert_replace() {
        let mut tree = BTree::<u32, i32, 8>::new();

        tree.insert(42, 100).unwrap();
        let old_value = tree.insert(42, 200).unwrap();

        assert_eq!(old_value, Some(100));
        assert_eq!(tree.get(&42), Some(&200));
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut tree = BTree::<u32, i32, 8>::new();
        tree.insert(42, 100).unwrap();
        assert_eq!(tree.len(), 1);

        let removed = tree.remove(&42);
        assert_eq!(removed, Some(100));
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());

        assert_eq!(tree.get(&42), None);
    }

    #[test]
    fn test_contains_key() {
        let mut tree = BTree::<u32, i32, 8>::new();
        assert!(!tree.contains_key(&42));

        tree.insert(42, 100).unwrap();
        assert!(tree.contains_key(&42));
        assert!(!tree.contains_key(&99));
    }

    #[test]
    fn test_multiple_insertions() {
        let mut tree = BTree::<u32, i32, 8>::new();
        for i in 0..10 {
            tree.insert(i, i as i32 * 2).unwrap();
        }
        assert_eq!(tree.len(), 10);
        for i in 0..10 {
            assert_eq!(tree.get(&i), Some(&(i as i32 * 2)));
        }
    }

    #[test]
    fn test_ordered_iteration() {
        let mut tree = BTree::<u32, i32, 8>::new();
        let keys = [5, 2, 8, 1, 9, 3, 7, 4, 6];
        for &key in &keys {
            tree.insert(key, key as i32).unwrap();
        }

        let mut sorted_keys: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
        sorted_keys.sort();

        let expected: Vec<_> = (1..=9).collect();
        assert_eq!(sorted_keys, expected);
    }

    #[test]
    fn test_clear() {
        let mut tree = BTree::<u32, i32, 8>::new();
        for i in 0..5 {
            tree.insert(i, i as i32).unwrap();
        }

        assert_eq!(tree.len(), 5);
        tree.clear();
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());

        for i in 0..5 {
            assert_eq!(tree.get(&i), None);
        }
    }

    #[test]
    fn test_branchless_search() {
        let mut tree = BTree::<u32, i32, 4>::new();
        for i in 0..20 {
            tree.insert(i * 2, i as i32).unwrap();
        }

        for i in 0..20 {
            assert_eq!(tree.get(&(i * 2)), Some(&(i as i32)));
            assert_eq!(tree.get(&(i * 2 + 1)), None);
        }
    }
}
