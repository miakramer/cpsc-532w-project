use std::iter::FromIterator;

use im::Vector;


#[derive(Debug, PartialEq, Clone)]
pub struct EqMap<Key: PartialEq + std::fmt::Debug + Clone, Val: std::fmt::Debug + Clone> {
    storage: Vector<(Key, Val)>,
}

impl<Key: PartialEq + std::fmt::Debug + Clone, Val: std::fmt::Debug + Clone> EqMap<Key, Val> {
    pub fn new() -> Self {
        Self {
            storage: Vector::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    fn index_of(&self, key: &Key) -> Option<usize> {
        for (i, (k, _)) in self.storage.iter().enumerate() {
            if k == key {
                return Some(i);
            }
        }
        None
    }

    pub fn contains(&self, key: &Key) -> bool {
        for (k, _) in self.storage.iter() {
            if k == key {
                return true;
            }
        }
        false
    }

    pub fn get<'a>(&'a self, key: &Key) -> Option<&'a Val> {
        for (k, v) in self.storage.iter() {
            if k == key {
                return Some(v)
            }
        }
        None
    }

    pub fn update(&self, key: Key, val: Val) -> Self {
        if let Some(idx) = self.index_of(&key) {
            Self {
                storage: self.storage.update(idx, (key, val))
            }
        } else {
            let mut storage = self.storage.clone();
            storage.push_back((key, val));
            Self { storage }
        }
    }
}


impl<Key: PartialEq + std::fmt::Debug + Clone, Val: std::fmt::Debug + Clone> FromIterator<(Key, Val)> for EqMap<Key, Val> {
    fn from_iter<T>(iter: T) -> Self
    where
        T : IntoIterator<Item = (Key, Val)>
    {
        let storage = Vector::from_iter(iter);
        Self {storage}
    }
}


impl<'a, Key: PartialEq + std::fmt::Debug + Clone, Val: std::fmt::Debug + Clone> IntoIterator for &'a EqMap<Key, Val> {
    type Item = &'a (Key, Val);
    type IntoIter = EqMapIter<'a, Key, Val>;

    fn into_iter(self) -> Self::IntoIter {
        EqMapIter{ eq: self, index: 0 }
    }
}


pub struct EqMapIter<'a, Key: PartialEq + std::fmt::Debug + Clone, Val: std::fmt::Debug + Clone> {
    eq: &'a EqMap<Key, Val>,
    index: usize,
}


impl<'a, Key: PartialEq + std::fmt::Debug + Clone, Val: std::fmt::Debug + Clone> Iterator for EqMapIter<'a, Key, Val> {
    type Item = &'a (Key, Val);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.eq.len() {
            None
        } else {
            let ret = &self.eq.storage[self.index];
            self.index += 1;
            Some(ret)
        }
    }
}
