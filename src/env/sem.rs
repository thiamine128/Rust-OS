use core::sync::atomic::{self, AtomicIsize};

use alloc::vec::Vec;

use crate::sync::cell::UPSafeCell;

/// number semaphores
pub const SEM_NUM: usize = 128;

/// global semaphore manager
pub static SEM_MAMANER: UPSafeCell<SemManager> = UPSafeCell::new(SemManager::new());

/// init semaphore manager
pub fn init() {
    let mut sem_manager = SEM_MAMANER.borrow_mut();
    sem_manager.init();
}

/// semaphore manager
pub struct SemManager {
    sems: Vec<AtomicIsize>,
    free: [u8; SEM_NUM],
}

impl SemManager {
    /// create a new semaphore manager
    pub const fn new() -> Self {
        Self {
            sems: Vec::new(),
            free: [1; SEM_NUM]
        }
    }
    /// init semaphore manager
    pub fn init(&mut self) {
        self.sems.resize_with(SEM_NUM, || {AtomicIsize::new(0)});
    }

    /// get semaphore
    pub fn get(&mut self, ind: usize) -> &mut AtomicIsize {
        &mut self.sems[ind]
    }
    /// open a semaphore
    pub fn sem_open(&mut self, ind: usize, v: isize) {
        if self.free[ind] == 1 {
            self.sems[ind].store(v, atomic::Ordering::Relaxed);
            self.free[ind] = 0;
        }
    }
    /// free a semaphore
    pub fn sem_free(&mut self, ind: usize) {
        self.free[ind] = 0;
    }
}