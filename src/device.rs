use core::ptr::copy;

use crate::memory::mmu::{PhysAddr, VirtAddr, KSEG1};

pub mod malta;

/// device manager struct
pub struct DeviceManager;

impl DeviceManager {
    /// read bytres from device
    pub fn read<T>(&self, va: VirtAddr, pa: PhysAddr) {
        let kva = VirtAddr::new(pa.as_usize() | KSEG1);
        self.dev_copy::<T>(kva, va);
    }
    /// write bytes to device
    pub fn write<T>(&self, va: VirtAddr, pa: PhysAddr) {
        let kva = VirtAddr::new(pa.as_usize() | KSEG1);
        self.dev_copy::<T>(va, kva);
    }

    /// 'memory' copy
    #[inline]
    fn dev_copy<T>(&self, src: VirtAddr, dst: VirtAddr) {
        let src = src.as_ptr::<T>();
        let dst = dst.as_mut_ptr::<T>();
        unsafe { copy(src, dst, 1); }
    }
}

/// console device address
pub const CONSOLE_ADDR: PhysAddr = PhysAddr::new(0x180003f8);
/// disk device address
pub const DISK_ADDR: PhysAddr = PhysAddr::new(0x180001f0);
/// console device len
pub const CONSOLE_LEN: usize = 0x20;
/// disk device len
pub const DISK_LEN: usize = 0x8;