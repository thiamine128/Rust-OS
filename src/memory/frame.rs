use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use crate::{err::Error, util::queue::IndexLink};

use super::{memset, mmu::{PhysAddr, PhysPageNum, VirtAddr, PAGE_SIZE}};

lazy_static! {
    static ref FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::new());
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
    FRAME_ALLOCATOR.lock().frames[ppn.as_usize()].pf_ref += 1;
}
#[inline]
pub fn init_frame_allocator(freemem: VirtAddr, nframes: usize) {
    FRAME_ALLOCATOR.lock().init(freemem, nframes);
}
#[inline]
pub fn frame_decref(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().decref(ppn);
}

#[derive(Clone)]
pub struct PhysFrame {
    pf_ref: u16
}

pub struct FrameAllocator {
    frames: Vec<PhysFrame>,
    nframes: usize,
    frames_free_list: IndexLink
}

impl FrameAllocator {
    #[inline]
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            nframes: 0,
            frames_free_list: IndexLink::new()
        }
    }

    #[inline]
    pub fn init(&mut self, freemem: VirtAddr, nframes: usize) {
        self.frames.resize(nframes, PhysFrame{ pf_ref: 0});
        self.frames_free_list.init(nframes);
        self.nframes = nframes;

        let used_ppn = PhysPageNum::from(PhysAddr::from_kva(freemem));
        for ppn in PhysPageNum::new(0)..used_ppn {
            self.frames[ppn.as_usize()].pf_ref = 1;
        }
        for ppn in used_ppn..PhysPageNum::new(nframes) {
            self.frames[ppn.as_usize()].pf_ref = 0;
            self.insert_head(ppn);
        }
    }

    #[inline]
    pub fn alloc(&mut self) -> Result<PhysPageNum, Error> {
        match self.frames_free_list.first() {
            Some(ind) => {
                let ppn = PhysPageNum::new(ind);
                memset(ppn.into_kva().as_mut_ptr(), 0, PAGE_SIZE);
                self.pop_first();
                Ok(ppn)
            },
            None => Err(Error::NoMem),
        }
    }

    #[inline]
    pub fn dealloc(&mut self, ppn: PhysPageNum) {
        assert!(self.frames[ppn.as_usize()].pf_ref == 0);
        self.insert_head(ppn);
    }

    #[inline]
    pub fn decref(&mut self, ppn: PhysPageNum) {
        let frame = &mut self.frames[ppn.as_usize()];
        assert!(frame.pf_ref > 0);
        frame.pf_ref -= 1;
        if frame.pf_ref == 0 {
            self.dealloc(ppn);
        }
    }

    #[inline]
    fn insert_head(&mut self, ppn: PhysPageNum) {
        self.frames_free_list.insert_head(ppn.as_usize());
    }

    #[inline]
    fn pop_first(&mut self) {
        self.frames_free_list.remove(self.frames_free_list.first().unwrap());
    }
}

/*
pub mod test {
    use core::ptr::addr_of_mut;

    use alloc::vec;

    use crate::memory::{frame::*, mmu::{PDMAP, ULIM}, page_table::PageTable};

    pub unsafe fn physical_memory_manage_strong_check() {
        

        let pp0 = frame_alloc().unwrap();
        let pp1 = frame_alloc().unwrap();
        let pp2 = frame_alloc().unwrap();
        let pp3 = frame_alloc().unwrap();
        let pp4 = frame_alloc().unwrap();

        
    
        assert!(pp1 != pp0);
        assert!(pp2 != pp1 && pp2 != pp0);
        assert!(pp3 != pp2 && pp3 != pp1 && pp3 != pp0);
        assert!(pp4 != pp3 && pp4 != pp2 && pp4 != pp1 && pp4 != pp0);
    
        while !frame_alloc().is_err() {
            
        }

        let temp1 = pp0.into_kva().as_mut_ptr();
        // write 1000 to pp0
        *temp1 = 1000;
        // free pp0
        frame_dealloc(pp0);
        println!("The number in address temp is {}", *temp1);
        
        // alloc again
        let pp0 = frame_alloc().unwrap();
        
        // pp0 should not change
        assert!(temp1 == pp0.into_kva().as_mut_ptr());
        // pp0 should be zero
        assert!(*temp1 == 0);
        
        frame_dealloc(pp0);
        frame_dealloc(pp1);
        frame_dealloc(pp2);
        frame_dealloc(pp3);
        frame_dealloc(pp4);

        let nn = FRAME_ALLOCATOR.lock().nframes;
        for i in 0..nn {
            if FRAME_ALLOCATOR.lock().frames[i].pf_ref == 0 {
                FRAME_ALLOCATOR.lock().frames_free_list.remove(i);
                FRAME_ALLOCATOR.lock().dealloc(PhysPageNum::new(i));
            }
        }

        let mut fa = FrameAllocator {
            frames: vec![PhysFrame{ pf_ref: 0 }; 17],
            nframes: 17,
            frames_free_list: IndexLink::new()
        };
        fa.frames_free_list.init(17);
        for i in (5..15).rev() {
            fa.frames[i].pf_ref = i as u16;
            fa.frames_free_list.insert_head(i);
        }
        for i in 0..5 {
            fa.frames[i].pf_ref = i as u16;
            fa.frames_free_list.insert_head(i);
        }
        let p = fa.frames_free_list.first();
        let answer1: [u16; 15] = [4, 3, 2, 1, 0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        assert!(p.is_some());
        let mut j = 0;
        for p in fa.frames_free_list.iter() {
            assert!(fa.frames[p].pf_ref == answer1[j]);
            j += 1;
        }
        // insert_after test
        let answer2: [u16; 17] = [4, 40, 20, 3, 2, 1, 0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        fa.frames[15].pf_ref = 20;
        fa.frames[16].pf_ref = 40;

        fa.frames_free_list.insert_after(4, 15);
        fa.frames_free_list.insert_after(4, 16);
        
        j = 0;
        for p in fa.frames_free_list.iter() {
            assert!(fa.frames[p].pf_ref == answer2[j]);
            j += 1;
        }
        
        println!("physical_memory_manage_check_strong() succeeded");
    }    

    pub unsafe fn page_strong_check() {
        // should be able to allocate a page for directory
        let pp = frame_alloc().unwrap();
        let pgdir = pp.into_kva().as_mut_ptr::<PageTable>().as_mut().unwrap();
    
        // should be able to allocate three pages
        let pp0 = frame_alloc().unwrap();
        let pp1 = frame_alloc().unwrap();
        let pp2 = frame_alloc().unwrap();
        let pp3 = frame_alloc().unwrap();
        let pp4 = frame_alloc().unwrap();
    
        assert!(pp1 != pp0);
        assert!(pp2 != pp1 && pp2 != pp0);
        assert!(pp3 != pp2 && pp3 != pp1 && pp3 != pp0);
        assert!(pp4 != pp3 && pp4 != pp2 && pp4 != pp1 && pp4 != pp0);
        
        while !frame_alloc().is_err() {}

        // there is no free memory, so we can't allocate a page table
        assert!(pgdir.insert(0, pp1, VirtAddr::new(0), 0).is_err());
    
        // should be no free memory
        assert!(frame_alloc().is_err());

        // free pp0 and try again: pp0 should be used for page table
        frame_dealloc(pp0);
        // check if PTE != PP
        assert!(pgdir.insert(0, pp1, VirtAddr::new(0), 0).is_ok());
        // should be able to map pp2 at PAGE_SIZE because pp0 is already allocated for page table
        assert!(pgdir.insert(0, pp2, VirtAddr::new(PAGE_SIZE), 0).is_ok());
        assert!(pgdir.insert(0, pp3, VirtAddr::new(2 * PAGE_SIZE), 0).is_ok());
        assert!(PhysAddr::from(pgdir.entries[0]) == PhysAddr::from(pp0));
    
        println!("va2pa(boot_pgdir, 0x0) is {:x}", pgdir.translate(VirtAddr::new(0)).unwrap().as_usize());
        println!("page2pa(pp1) is {:x}", PhysAddr::from(pp1).as_usize());
    
        assert!(pgdir.translate(VirtAddr::new(0)).unwrap() == PhysAddr::from(pp1));
        assert!(FRAME_ALLOCATOR.lock().frames[pp1.as_usize()].pf_ref == 1);
        assert!(pgdir.translate(VirtAddr::new(PAGE_SIZE)).unwrap() == PhysAddr::from(pp2));
        assert!(FRAME_ALLOCATOR.lock().frames[pp2.as_usize()].pf_ref== 1);
        assert!(pgdir.translate(VirtAddr::new(2 * PAGE_SIZE)).unwrap() == PhysAddr::from(pp3));
        assert!(FRAME_ALLOCATOR.lock().frames[pp3.as_usize()].pf_ref == 1);
    
        println!("start page_insert");
        // should be able to map pp2 at PAGE_SIZE because it's already there
        assert!(pgdir.insert(0, pp2, VirtAddr::new(PAGE_SIZE), 0).is_ok());
        assert!(pgdir.translate(VirtAddr::new(PAGE_SIZE)).unwrap() == PhysAddr::from(pp2));
        assert!(FRAME_ALLOCATOR.lock().frames[pp2.as_usize()].pf_ref == 1);
    
        // should not be able to map at PDMAP because need free page for page table
        assert!(pgdir.insert(0, pp0, VirtAddr::new(PDMAP), 0).is_err());
        // remove pp1 try again
        pgdir.remove(0, VirtAddr::new(0));
        assert!(pgdir.translate(VirtAddr::new(0x0)).is_none());
        assert!(pgdir.insert(0, pp0, VirtAddr::new(PDMAP), 0).is_ok());
    
        // insert pp2 at 2*PAGE_SIZE (replacing pp2)
        assert!(pgdir.insert(0, pp2, VirtAddr::new(2 * PAGE_SIZE), 0).is_ok());
    
        // should have pp2 at both 0 and PAGE_SIZE, pp2 nowhere, ...
        assert!(pgdir.translate(VirtAddr::new(PAGE_SIZE)).unwrap() == PhysAddr::from(pp2));
        assert!(pgdir.translate(VirtAddr::new(2 * PAGE_SIZE)).unwrap() == PhysAddr::from(pp2));
        // ... and ref counts should reflect this
        assert!(FRAME_ALLOCATOR.lock().frames[pp2.as_usize()].pf_ref == 2);
        assert!(FRAME_ALLOCATOR.lock().frames[pp3.as_usize()].pf_ref == 0);
        // try to insert PDMAP+PAGE_SIZE
        assert!(pgdir.insert(0, pp2, VirtAddr::new(PDMAP + PAGE_SIZE), 0).is_ok());
        assert!(FRAME_ALLOCATOR.lock().frames[pp2.as_usize()].pf_ref == 3);
        println!("end page_insert");
    
        // pp2 should be returned by page_alloc
        let pp = frame_alloc().unwrap();
        assert!(pp == pp3);
    
        // unmapping pp2 at PAGE_SIZE should keep pp1 at 2*PAGE_SIZE
        pgdir.remove(0, VirtAddr::new(PAGE_SIZE));
        assert!(pgdir.translate(VirtAddr::new(2 * PAGE_SIZE)).unwrap() == PhysAddr::from(pp2));
        assert!(FRAME_ALLOCATOR.lock().frames[pp2.as_usize()].pf_ref == 2);
        assert!(FRAME_ALLOCATOR.lock().frames[pp3.as_usize()].pf_ref == 0);
    
        // unmapping pp2 at 2*PAGE_SIZE should keep pp2 at PDMAP+PAGE_SIZE
        pgdir.remove(0, VirtAddr::new(2 * PAGE_SIZE));
        assert!(pgdir.translate(VirtAddr::new(0)).is_none());
        assert!(pgdir.translate(VirtAddr::new(PAGE_SIZE)).is_none());
        assert!(pgdir.translate(VirtAddr::new(2 * PAGE_SIZE)).is_none());
        assert!(FRAME_ALLOCATOR.lock().frames[pp2.as_usize()].pf_ref == 1);
        assert!(FRAME_ALLOCATOR.lock().frames[pp3.as_usize()].pf_ref == 0);
    
        // unmapping pp2 at PDMAP+PAGE_SIZE should free it
        pgdir.remove(0, VirtAddr::new(PDMAP + PAGE_SIZE));
        assert!(pgdir.translate(VirtAddr::new(0)).is_none());
        assert!(pgdir.translate(VirtAddr::new(PAGE_SIZE)).is_none());
        assert!(pgdir.translate(VirtAddr::new(2 * PAGE_SIZE)).is_none());
        assert!(pgdir.translate(VirtAddr::new(PDMAP + PAGE_SIZE)).is_none());
        assert!(FRAME_ALLOCATOR.lock().frames[pp2.as_usize()].pf_ref == 0);
    
        // so it should be returned by page_alloc
        let pp = frame_alloc().unwrap();
        assert!(pp == pp2);
    
        // should be no free memory
        assert!(frame_alloc().is_err());
    
        // forcibly take pp0 and pp1  back
        assert!(pgdir.entries[0].ppn() == pp0);
        assert!(pgdir.entries[1].ppn() == pp1);
        assert!(FRAME_ALLOCATOR.lock().frames[pp0.as_usize()].pf_ref == 2);
        assert!(FRAME_ALLOCATOR.lock().frames[pp1.as_usize()].pf_ref == 1);
        FRAME_ALLOCATOR.lock().frames[pp0.as_usize()].pf_ref = 0;
        FRAME_ALLOCATOR.lock().frames[pp1.as_usize()].pf_ref = 0;
    
        // give free list back
        
    
        // free the pages we took
        frame_dealloc(pp0);
        frame_dealloc(pp1);
        frame_dealloc(pp2);
        frame_dealloc(pp3);
        frame_dealloc(pp4);
        frame_dealloc(PhysPageNum::from(PhysAddr::from_kva(VirtAddr::from_ptr(addr_of_mut!(*pgdir)))));

        let nn = FRAME_ALLOCATOR.lock().nframes;
        for i in 0..nn {
            if FRAME_ALLOCATOR.lock().frames[i].pf_ref == 0 {
                FRAME_ALLOCATOR.lock().frames_free_list.remove(i);
                FRAME_ALLOCATOR.lock().dealloc(PhysPageNum::new(i));
            }
        }
    
        println!("page_check_strong() succeeded!\n");
    }    
}*/