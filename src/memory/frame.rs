use core::{alloc::Layout, mem::size_of, ptr::write_bytes};

use alloc::vec::Vec;

use crate::{err::Error, sync::cell::UPSafeCell, util::queue::IndexLink};

use super::mmu::{PhysAddr, PhysPageNum, VirtAddr, PAGE_SIZE};

/// global frame allocator
static FRAME_ALLOCATOR: UPSafeCell<FrameAllocator> = UPSafeCell::new(FrameAllocator::new());

/// alloc frame
#[inline]
pub fn frame_alloc() -> Result<PhysPageNum, Error> { FRAME_ALLOCATOR.borrow_mut().alloc() }
/// dealloc frame
#[inline]
pub fn frame_dealloc(ppn: PhysPageNum) { FRAME_ALLOCATOR.borrow_mut().dealloc(ppn); }
/// increase ref count for frame
#[inline]
pub fn frame_incref(ppn: PhysPageNum) { FRAME_ALLOCATOR.borrow_mut().frames[ppn.as_usize()].pf_ref += 1; }
/// init frame allocator
#[inline]
pub fn init_frame_allocator(freemem: VirtAddr, nframes: usize) { FRAME_ALLOCATOR.borrow_mut().init(freemem, nframes); }
/// decrease ref count for frame
#[inline]
pub fn frame_decref(ppn: PhysPageNum) { FRAME_ALLOCATOR.borrow_mut().decref(ppn); }
/// get frame base address
#[inline]
pub fn frame_base_phy_addr() -> PhysAddr { FRAME_ALLOCATOR.borrow_mut().base_phy_addr() }
/// get frame base size
#[inline]
pub fn frame_base_size() -> usize { FRAME_ALLOCATOR.borrow_mut().base_size }
/// number of free frames
#[inline]
pub fn num_free_frames() -> usize { FRAME_ALLOCATOR.borrow_mut().frames_free_list.len() }
/// clear a frame by setting 0
#[inline]
fn frame_clear(ppn: PhysPageNum) {
    let dst = ppn.into_kva().as_mut_ptr::<u8>();
    unsafe { write_bytes(dst, 0, PAGE_SIZE); }
}

/// phys frame
#[derive(Clone)]
pub struct PhysFrame {
    pf_ref: u16
}
/// frame allocator
pub struct FrameAllocator {
    frames: Vec<PhysFrame>,
    nframes: usize,
    frames_free_list: IndexLink,
    base_addr: VirtAddr,
    base_size: usize
}

impl FrameAllocator {
    /// create a frame allocator
    #[inline]
    pub const fn new() -> Self {
        Self {
            frames: Vec::new(),
            nframes: 0,
            frames_free_list: IndexLink::new(),
            base_addr: VirtAddr::new(0),
            base_size: 0
        }
    }
    /// init a frame allocator
    #[inline]
    pub fn init(&mut self, freemem: VirtAddr, nframes: usize) {
        let frames_size = nframes * size_of::<PhysFrame>();
        let layout = Layout::from_size_align(frames_size, PAGE_SIZE).unwrap();
        let base_addr = VirtAddr::from_ptr(unsafe {
          alloc::alloc::alloc(layout)
        });
        self.base_addr = base_addr;
        self.base_size = frames_size;
        let frames_addr = base_addr.as_mut_ptr();
        self.frames = unsafe {Vec::from_raw_parts(frames_addr, nframes, nframes)};
        self.frames_free_list.init(nframes);
        self.nframes = nframes;

        let used_ppn = PhysPageNum::from(PhysAddr::from_kva(freemem));
        for ppn in PhysPageNum::new(0)..used_ppn {
            self.frames[ppn.as_usize()].pf_ref = 1;
        }
        for ppn in used_ppn..PhysPageNum::new(nframes) {
            self.frames[ppn.as_usize()].pf_ref = 0;
            self.insert_tail(ppn);
        }
    }

    /// alloc frame
    #[inline]
    pub fn alloc(&mut self) -> Result<PhysPageNum, Error> {
        match self.frames_free_list.first() {
            Some(ind) => {
                let ppn = PhysPageNum::new(ind);
                frame_clear(ppn);
                self.pop_first();
                Ok(ppn)
            },
            None => Err(Error::NoMem),
        }
    }

    /// dealloc frame
    #[inline]
    pub fn dealloc(&mut self, ppn: PhysPageNum) {
        assert!(self.frames[ppn.as_usize()].pf_ref == 0);
        self.insert_tail(ppn);
    }
    /// decrease ref count of frame
    #[inline]
    pub fn decref(&mut self, ppn: PhysPageNum) {
        let frame = &mut self.frames[ppn.as_usize()];
        assert!(frame.pf_ref > 0);
        frame.pf_ref -= 1;
        if frame.pf_ref == 0 {
            self.dealloc(ppn);
        }
    }
    /// insert free frame to tail
    #[inline]
    fn insert_tail(&mut self, ppn: PhysPageNum) {
        self.frames_free_list.insert_tail(ppn.as_usize());
    }
    /// pop first free frame
    #[inline]
    fn pop_first(&mut self) {
        self.frames_free_list.remove(self.frames_free_list.first().unwrap());
    }
    /// frames base physical address
    #[inline]
    pub fn base_phy_addr(&mut self) -> PhysAddr {
        PhysAddr::from_kva(self.base_addr)
    }
}