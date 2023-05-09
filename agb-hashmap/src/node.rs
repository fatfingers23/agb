use core::{
    mem::{self, MaybeUninit},
    ptr,
};

use crate::HashType;

pub(crate) struct Node<K, V> {
    hash: HashType,

    // distance_to_initial_bucket = -1 => key and value are uninit.
    // distance_to_initial_bucket >= 0 => key and value are init
    distance_to_initial_bucket: i32,
    key: MaybeUninit<K>,
    value: MaybeUninit<V>,
}

impl<K, V> Node<K, V> {
    fn new() -> Self {
        Self {
            hash: 0,
            distance_to_initial_bucket: -1,
            key: MaybeUninit::uninit(),
            value: MaybeUninit::uninit(),
        }
    }

    pub(crate) fn new_with(key: K, value: V, hash: HashType) -> Self {
        Self {
            hash,
            distance_to_initial_bucket: 0,
            key: MaybeUninit::new(key),
            value: MaybeUninit::new(value),
        }
    }

    pub(crate) fn value_ref(&self) -> Option<&V> {
        if self.has_value() {
            Some(unsafe { self.value.assume_init_ref() })
        } else {
            None
        }
    }

    pub(crate) fn value_mut(&mut self) -> Option<&mut V> {
        if self.has_value() {
            Some(unsafe { self.value.assume_init_mut() })
        } else {
            None
        }
    }

    pub(crate) fn key_ref(&self) -> Option<&K> {
        if self.distance_to_initial_bucket >= 0 {
            Some(unsafe { self.key.assume_init_ref() })
        } else {
            None
        }
    }

    pub(crate) fn key_value_ref(&self) -> Option<(&K, &V)> {
        if self.has_value() {
            Some(unsafe { (self.key.assume_init_ref(), self.value.assume_init_ref()) })
        } else {
            None
        }
    }

    pub(crate) fn key_value_mut(&mut self) -> Option<(&K, &mut V)> {
        if self.has_value() {
            Some(unsafe { (self.key.assume_init_ref(), self.value.assume_init_mut()) })
        } else {
            None
        }
    }

    pub(crate) fn has_value(&self) -> bool {
        self.distance_to_initial_bucket >= 0
    }

    pub(crate) fn take_key_value(&mut self) -> Option<(K, V, HashType)> {
        if self.has_value() {
            let key = mem::replace(&mut self.key, MaybeUninit::uninit());
            let value = mem::replace(&mut self.value, MaybeUninit::uninit());
            self.distance_to_initial_bucket = -1;

            Some(unsafe { (key.assume_init(), value.assume_init(), self.hash) })
        } else {
            None
        }
    }

    pub(crate) fn replace_value(&mut self, value: V) -> V {
        if self.has_value() {
            let old_value = mem::replace(&mut self.value, MaybeUninit::new(value));
            unsafe { old_value.assume_init() }
        } else {
            panic!("Cannot replace an uninitialised node");
        }
    }

    pub(crate) fn replace(&mut self, key: K, value: V) -> (K, V) {
        if self.has_value() {
            let old_key = mem::replace(&mut self.key, MaybeUninit::new(key));
            let old_value = mem::replace(&mut self.value, MaybeUninit::new(value));

            unsafe { (old_key.assume_init(), old_value.assume_init()) }
        } else {
            panic!("Cannot replace an uninitialised node");
        }
    }

    pub(crate) fn increment_distance(&mut self) {
        self.distance_to_initial_bucket += 1;
    }

    pub(crate) fn decrement_distance(&mut self) {
        self.distance_to_initial_bucket -= 1;
        if self.distance_to_initial_bucket < 0 {
            panic!("Cannot decrement distance to below 0");
        }
    }

    pub(crate) fn distance(&self) -> i32 {
        self.distance_to_initial_bucket
    }

    pub(crate) fn hash(&self) -> HashType {
        self.hash
    }
}

impl<K, V> Drop for Node<K, V> {
    fn drop(&mut self) {
        if self.has_value() {
            unsafe { ptr::drop_in_place(self.key.as_mut_ptr()) };
            unsafe { ptr::drop_in_place(self.value.as_mut_ptr()) };
        }
    }
}

impl<K, V> Default for Node<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
