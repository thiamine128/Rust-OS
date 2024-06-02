use core::ptr::copy;

use crate::memory::mmu::{PhysAddr, VirtAddr, KSEG1};

pub mod malta;

pub struct DeviceManager;

impl DeviceManager {
    pub fn read<T>(&self, va: VirtAddr, pa: PhysAddr) {
        let kva = VirtAddr::new(pa.as_usize() | KSEG1);
        self.dev_copy::<T>(kva, va);
    }
    
    pub fn write<T>(&self, va: VirtAddr, pa: PhysAddr) {
        let kva = VirtAddr::new(pa.as_usize() | KSEG1);
        self.dev_copy::<T>(va, kva);
    }

    #[inline]
    fn dev_copy<T>(&self, src: VirtAddr, dst: VirtAddr) {
        let src = src.as_ptr::<T>();
        let dst = dst.as_mut_ptr::<T>();
        unsafe { copy(src, dst, 1); }
    }
}

pub const CONSOLE_ADDR: PhysAddr = PhysAddr::new(0x180003f8);
pub const DISK_ADDR: PhysAddr = PhysAddr::new(0x180001f0);
pub const CONSOLE_LEN: usize = 0x20;
pub const DISK_LEN: usize = 0x8;