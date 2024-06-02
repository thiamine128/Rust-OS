use core::sync::atomic::{self, AtomicI32, AtomicIsize, AtomicUsize};

use alloc::vec::Vec;

use crate::{err::Error, sync::cell::UPSafeCell};

pub const SEM_NUM: usize = 128;

pub static SEM_MAMANER: UPSafeCell<SemManager> = UPSafeCell::new(SemManager::new());

pub fn init() {
    let mut sem_manager = SEM_MAMANER.borrow_mut();
    sem_manager.init();
}

pub struct SemManager {
    sems: Vec<AtomicIsize>,
    free: [u8; SEM_NUM],
}

impl SemManager {
    pub const fn new() -> Self {
        Self {
            sems: Vec::new(),
            free: [1; SEM_NUM]
        }
    }

    pub fn init(&mut self) {
        self.sems.resize_with(SEM_NUM, || {AtomicIsize::new(0)});
    }

    pub fn get(&mut self, ind: usize) -> &mut AtomicIsize {
        &mut self.sems[ind]
    }

    pub fn sem_open(&mut self, ind: usize, v: isize) {
        if self.free[ind] == 1 {
            self.sems[ind].store(v, atomic::Ordering::Relaxed);
            self.free[ind] = 0;
        }
    }

    pub fn sem_free(&mut self, ind: usize) {
        self.free[ind] = 0;
    }
}