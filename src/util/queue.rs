use core::{mem::size_of, slice};

use alloc::vec::Vec;

use crate::println;

pub struct IndexLink {
    n: usize,
    le_next: Vec<Option<usize>>,
    le_prev: Vec<Option<usize>>
}

pub struct IndexIterator<'a> {
    index_link: &'a IndexLink,
    index: usize,
}

impl IndexLink {
    #[inline]
    pub fn new() -> Self {
        IndexLink {
            n: 0,
            le_next: Vec::new(),
            le_prev: Vec::new()
        }
    }
    #[inline]
    pub fn init(&mut self, n: usize) {
        self.n = n;
        let occupied = n + 2;
        self.le_next.resize(occupied, None);
        self.le_prev.resize(occupied, None);
        self.le_next[n] = Some(n + 1);
        self.le_prev[n + 1] = Some(n);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.le_next[self.n] == Some(self.n + 1)
    }
    #[inline]
    pub fn first(&self) -> Option<usize> {
        self.le_next[self.n]
    }
    #[inline]
    pub fn insert_after(&mut self, listelm: usize, elm: usize) {
        self.le_next[elm] = self.le_next[listelm];
        if let Some(x) = self.le_next[listelm] {
            self.le_prev[x] = Some(elm);
        }
        self.le_next[listelm] = Some(elm);
        self.le_prev[elm] = Some(listelm);
    }
    #[inline]
    pub fn insert_before(&mut self, listelm: usize, elm: usize) {
        self.le_prev[elm] = self.le_prev[listelm];
        self.le_next[elm] = Some(listelm);
        if let Some(x) = self.le_prev[listelm] {
            self.le_next[x] = Some(elm);
        }
        self.le_prev[listelm] = Some(elm);
    }
    #[inline]
    pub fn insert_head(&mut self, elm: usize) {
        self.insert_after(self.n, elm);
    }
    #[inline]
    pub fn insert_tail(&mut self, elm: usize) {
        self.insert_before(self.n + 1, elm);
    }
    #[inline]
    pub fn remove(&mut self, elm: usize) {
        if let Some(x) = self.le_next[elm] {
            self.le_prev[x] = self.le_prev[elm];
        }
        if let Some(x) = self.le_prev[elm] {
            self.le_next[x] = self.le_next[elm];
        }
        self.le_next[elm] = None;
        self.le_prev[elm] = None;
    }
    #[inline]
    pub fn iter(&self) -> IndexIterator {
        IndexIterator {
            index_link: self,
            index: self.n
        }
    }
}

impl<'a> Iterator for IndexIterator<'a> {
    type Item = usize;
    
    fn next(&mut self) -> Option<Self::Item> {

        match self.index_link.le_next[self.index] {
            Some(x) => {
                if x == self.index_link.n + 1 {
                    None
                } else {
                    self.index = x;
                    Some(x)
                }
            },
            None => None
        }
    }
}