use core::{mem::{self, size_of}, ptr::NonNull, slice};

use alloc::alloc::handle_alloc_error;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use crate::{memory::frame, print, println};

use super::{memset, mmu::{PhysAddr, PhysPageNum, VirtAddr, PAGE_SIZE}, Error};

lazy_static! {
    static ref FRAME_ALLOCATOR: Mutex<FrameAllocator<'static>> = Mutex::new(FrameAllocator::new());
}

#[inline]
pub fn frame_alloc() -> Result<PhysPageNum, Error> {
    FRAME_ALLOCATOR.lock().alloc()
}
#[inline]
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().dealloc(ppn);
}
#[inline]
pub fn frame_incref(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().get_frame_mut_by_ppn(ppn).pf_ref += 1;
}
#[inline]
pub fn init_frame_allocator(baseAddr: *mut PhysFrame, freemem: VirtAddr, nframes: usize) {
    FRAME_ALLOCATOR.lock().init(baseAddr, freemem, nframes);
}
#[inline]
pub fn frame_decref(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().decref(ppn);
}
#[inline]
pub fn steal(fl: &mut Option<PhysPageNum>) {
    *fl = FRAME_ALLOCATOR.lock().frames_free_list.head;
    FRAME_ALLOCATOR.lock().frames_free_list.head = None;
}
#[inline]
pub fn recover(fl: Option<PhysPageNum>) {
    FRAME_ALLOCATOR.lock().frames_free_list.head = fl;
}
#[inline]
pub fn get_frame_ref(ppn: PhysPageNum) -> u16{
    FRAME_ALLOCATOR.lock().get_frame_by_ppn(ppn).pf_ref
}
#[inline]
pub fn set_frame_ref(ppn: PhysPageNum, pf_ref: u16) {
    FRAME_ALLOCATOR.lock().get_frame_mut_by_ppn(ppn).pf_ref = pf_ref;
}

type PhysFrameLink = Option<PhysPageNum>;
pub struct PhysFrame {
    pf_link: PhysFrameLink,
    pf_ref: u16
}
pub struct PhysFrameList {
    head: PhysFrameLink
}
pub struct FrameAllocator<'a> {
    frames: &'a mut [PhysFrame],
    nframes: usize,
    frames_free_list: PhysFrameList
}

impl<'a> FrameAllocator<'a> {
    #[inline]
    pub fn new() -> Self {
        Self {
            frames: &mut [],
            nframes: 0,
            frames_free_list: PhysFrameList {
                head: None
            }
        }
    }

    #[inline]
    fn get_frame_mut_by_ppn(&mut self, ppn: PhysPageNum) -> &mut PhysFrame {
        self.frames.get_mut(ppn.as_usize()).unwrap()
    }

    #[inline]
    fn get_frame_by_ppn(&self, ppn: PhysPageNum) -> &PhysFrame {
        self.frames.get(ppn.as_usize()).unwrap()
    }

    #[inline]
    pub fn init(&mut self, baseAddr: *mut PhysFrame, freemem: VirtAddr, nframes: usize) {
        self.frames = unsafe {slice::from_raw_parts_mut(baseAddr, nframes)};
        self.nframes = nframes;

        let used_ppn = PhysPageNum::from(PhysAddr::from_kva(freemem));
        for ppn in PhysPageNum::new(0)..used_ppn {
            self.get_frame_mut_by_ppn(ppn).pf_ref = 1;
        }
        for ppn in used_ppn..PhysPageNum::new(nframes) {
            self.get_frame_mut_by_ppn(ppn).pf_ref = 0;
            self.push(ppn);
        }
    }

    #[inline]
    pub fn alloc(&mut self) -> Result<PhysPageNum, Error> {
        match self.pop() {
            Some(ppn) => {
                memset(ppn.into_kva().as_mut_ptr(), 0, PAGE_SIZE);
                Ok(ppn)
            },
            None => Err(Error::NO_MEM),
        }
    }

    #[inline]
    pub fn dealloc(&mut self, ppn: PhysPageNum) {
        assert!(self.get_frame_by_ppn(ppn).pf_ref == 0);
        self.push(ppn);
    }

    #[inline]
    pub fn decref(&mut self, ppn: PhysPageNum) {
        let frame = self.get_frame_mut_by_ppn(ppn);
        assert!(frame.pf_ref > 0);
        frame.pf_ref -= 1;
        if frame.pf_ref == 0 {
            self.dealloc(ppn);
        }
    }

    #[inline]
    fn push(&mut self, ppn: PhysPageNum) {
        match self.frames_free_list.head {
            Some(head) => {
                self.get_frame_mut_by_ppn(ppn).pf_link = Some(head);
                self.frames_free_list.head = Some(ppn)
            },
            None => self.frames_free_list.head = Some(ppn),
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<PhysPageNum> {
        match self.frames_free_list.head {
            Some(head) => {
                self.frames_free_list.head = self.get_frame_by_ppn(head).pf_link;
                self.get_frame_mut_by_ppn(head).pf_link = None;
                Some(head)
            },
            None => None,
        }
    }
}

/// from lab2_1
pub fn physical_memory_manage_strong_check() {
	let mut pp0 = frame_alloc().unwrap();
	let pp1 = frame_alloc().unwrap();
	let pp2 = frame_alloc().unwrap();
	let pp3 = frame_alloc().unwrap();
	let pp4 = frame_alloc().unwrap();

	assert!(pp1 != pp0);
	assert!(pp2 != pp1 && pp2 != pp0);
	assert!(pp3 != pp2 && pp3 != pp1 && pp3 != pp0);
	assert!(pp4 != pp3 && pp4 != pp2 && pp4 != pp1 && pp4 != pp0);

	// temporarily steal the rest of the free pages
	let fl = FRAME_ALLOCATOR.lock().frames_free_list.head;
	// now this page_free list must be empty!!!!
    FRAME_ALLOCATOR.lock().frames_free_list.head = None;
	// should be no free memory
	assert!(frame_alloc().is_err());

	let temp1: *mut u32 = pp0.into_kva().as_mut_ptr();
	// write 1000 to pp0
	unsafe {*temp1 = 1000;}
	// free pp0
	FRAME_ALLOCATOR.lock().dealloc(pp0);
	println!("The number in address temp is {}", unsafe {*temp1});

	// alloc again
	pp0 = frame_alloc().unwrap();
	// pp0 should not change
	assert!(temp1 == pp0.into_kva().as_mut_ptr());
	// pp0 should be zero
	assert!(unsafe{*temp1} == 0);

	FRAME_ALLOCATOR.lock().frames_free_list.head = fl;
    FRAME_ALLOCATOR.lock().dealloc(pp0);
    FRAME_ALLOCATOR.lock().dealloc(pp1);
    FRAME_ALLOCATOR.lock().dealloc(pp2);
    FRAME_ALLOCATOR.lock().dealloc(pp3);
    FRAME_ALLOCATOR.lock().dealloc(pp4);


    extern "C" {
        fn end();
    }

    let mut fa = FrameAllocator::new();
    let mut test_addr = end as usize;
    test_addr += FRAME_ALLOCATOR.lock().nframes * size_of::<PhysFrame>();
    let mut freemem = test_addr + 15 * size_of::<PhysFrame>();
    //fa.init(test_addr as *mut PhysFrame, VirtAddr::new(test_addr), 15);
    memset(test_addr as *mut u8, 0, freemem - test_addr);
    fa.frames = unsafe {slice::from_raw_parts_mut(test_addr as *mut PhysFrame, 15)};


	for i in (PhysPageNum::new(5)..PhysPageNum::new(15)).rev() {
		fa.get_frame_mut_by_ppn(i).pf_ref = i.as_usize() as u16;
		fa.push(i);
	}
	for i in PhysPageNum::new(0)..PhysPageNum::new(5) {
		fa.get_frame_mut_by_ppn(i).pf_ref = i.as_usize() as u16;
		fa.push(i);
	}

	let mut p = fa.frames_free_list.head;
	let answer1: [u16; 15] = [4, 3, 2, 1, 0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
	let mut j = 0;
    assert!(p.is_some());
	while !p.is_none() {
		assert!(fa.get_frame_by_ppn(p.unwrap()).pf_ref == answer1[j]);
		j += 1;
		p = fa.get_frame_by_ppn(p.unwrap()).pf_link;
	}
    FRAME_ALLOCATOR.lock().frames_free_list.head = fl;
	println!("physical_memory_manage_check_strong() succeeded");
}