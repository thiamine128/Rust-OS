use core::mem::size_of;

use alloc::vec::Vec;

use crate::memory::mmu::VirtAddr;

/// index link for linked list
pub struct IndexLink {
    n: usize,
    le_next: Vec<Option<usize>>,
    le_prev: Vec<Option<usize>>,
    rem: usize
}

/// iterator
pub struct IndexIterator<'a> {
    index_link: &'a IndexLink,
    index: usize,
}

impl IndexLink {
    /// create a new link
    #[inline]
    pub const fn new() -> Self {
        IndexLink {
            n: 0,
            le_next: Vec::new(),
            le_prev: Vec::new(),
            rem: 0
        }
    }
    /// init list from raw pointer
    #[inline]
    pub fn init_from_ptr(&mut self, addr: VirtAddr, n: usize) {
        self.n = n;
        let next_addr: *mut Option<usize> = addr.as_mut_ptr();
        let occupied = n + 2;
        self.le_next = unsafe { Vec::from_raw_parts(next_addr, occupied, occupied) };
        let prev_addr: *mut Option<usize> = (addr + occupied * size_of::<Option<usize>>()).as_mut_ptr();
        self.le_prev = unsafe {
            Vec::from_raw_parts(prev_addr, occupied, occupied)
        };
        self.le_next[n] = Some(n + 1);
        self.le_prev[n + 1] = Some(n);
    }

    /// simple init
    #[inline]
    pub fn init(&mut self, n: usize) {
        self.n = n;
        let occupied = n + 2;
        self.le_next = Vec::with_capacity(occupied);
        self.le_next.resize(occupied, None);
        self.le_prev = Vec::with_capacity(occupied);
        self.le_prev.resize(occupied, None);
        self.le_next[n] = Some(n + 1);
        self.le_prev[n + 1] = Some(n);
    }

    /// check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.le_next[self.n] == Some(self.n + 1)
    }

    /// first index
    #[inline]
    pub fn first(&self) -> Option<usize> {
        self.le_next[self.n]
    }

    /// insert after an element index
    #[inline]
    pub fn insert_after(&mut self, listelm: usize, elm: usize) {
        self.le_next[elm] = self.le_next[listelm];
        if let Some(x) = self.le_next[listelm] {
            self.le_prev[x] = Some(elm);
        }
        self.le_next[listelm] = Some(elm);
        self.le_prev[elm] = Some(listelm);
        self.rem += 1;
    }

    /// insert before an element index
    #[inline]
    pub fn insert_before(&mut self, listelm: usize, elm: usize) {
        self.le_prev[elm] = self.le_prev[listelm];
        self.le_next[elm] = Some(listelm);
        if let Some(x) = self.le_prev[listelm] {
            self.le_next[x] = Some(elm);
        }
        self.le_prev[listelm] = Some(elm);
        self.rem += 1;
    }
    /// wrapped call for insert after
    #[inline]
    pub fn insert_head(&mut self, elm: usize) {
        self.insert_after(self.n, elm);
    }

    /// wrapped call for insert before
    #[inline]
    pub fn insert_tail(&mut self, elm: usize) {
        self.insert_before(self.n + 1, elm);
    }

    /// remove an index from list
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
        self.rem -= 1;
    }

    /// get iterator for list
    #[inline]
    pub fn iter(&self) -> IndexIterator {
        IndexIterator {
            index_link: self,
            index: self.n
        }
    }

    /// get size for an index link of size len
    #[inline]
    pub fn get_size_for(len: usize) -> usize {
        (len + 2) * size_of::<Option<usize>>() * 2
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.rem
    }
}

impl<'a> Iterator for IndexIterator<'a> {
    type Item = usize;
    
    /// next element
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