use core::ptr::{addr_of_mut, null_mut};

use spin::mutex::Mutex;

use crate::{memory::frame::{frame_dealloc, get_frame_ref, recover, set_frame_ref, steal}, println};

use super::{frame::{frame_alloc, frame_decref, frame_incref}, mmu::*, tlb::tlb_invalidate, Error};
use lazy_static::lazy_static;
pub const PAGE_TABLE_ENTRIES: usize = PAGE_SIZE / 4;

/// Page table entry, wrapped type.
/// In original mos, there are pte and pde.
/// For better abstraction on paging, pte refers to both pte and pde in mos-rust.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Pte(usize);

/// Page table, aligned with 4096 bytes.
#[repr(align(4096))]
#[repr(C)]
pub struct PageTable {
    entries: [Pte; PAGE_TABLE_ENTRIES]
}

pub struct PageTablePtr(*mut PageTable);

unsafe impl Send for PageTablePtr {
    
}

lazy_static! {
    static ref CUR_PGDIR: Mutex<PageTablePtr> = Mutex::new(PageTablePtr(null_mut()));
}

pub fn cur_pgdir() -> Option<&'static mut PageTable> {
    unsafe {(CUR_PGDIR.lock().0).as_mut()}
}

pub fn set_cur_pgdir(pgdir: &mut PageTable) {
    *CUR_PGDIR.lock() = PageTablePtr(addr_of_mut!(*pgdir));
}


impl Pte {
    /// create a new virtual address from numeric value.
    #[inline]
    pub const fn new(pte: usize) -> Self{
        Self(pte)
    }
    #[inline]
    pub const fn new_from_ppn(ppn: PhysPageNum, perm: usize) -> Self {
        Self((ppn.as_usize() << PGSHIFT) | perm)
    }
    /// get pte as raw value.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0
    }
    /// ppn of this entry.
    #[inline]
    pub const fn ppn(self) -> PhysPageNum {
        PhysPageNum::new((self.0 >> PGSHIFT) & 0xfffff)
    }
    /// flags of this entry.
    #[inline]
    pub const fn perm(self) -> usize {
        self.0 & 0xfff
    }

    #[inline]
    pub const fn valid(self) -> bool {
        self.0 & PTE_V != 0
    }
}

impl PageTable {
    #[inline]
    fn walk_or_create(&mut self, va: VirtAddr, create: i32) -> Result<&mut Pte, Error> {
        let pte = self.entries[va.pdx()];
        let mut ppn = pte.ppn();
        if !pte.valid() {
            if create != 0 {
                ppn = frame_alloc()?;
                frame_incref(ppn);
                self.entries[va.pdx()] = Pte::new_from_ppn(ppn, PTE_C_CACHEABLE | PTE_V);
            } else {
                return Err(Error::NotMapped);
            }
        }
        let page_table: &mut PageTable = unsafe {&mut *(ppn.into_kva().as_mut_ptr())};
        Ok(&mut page_table.entries[va.ptx()])
    }
    #[inline]
    pub fn lookup(&mut self, va: VirtAddr) -> Result<(PhysPageNum, &mut Pte), Error> {
        let pte = self.walk_or_create(va, 0)?;
        Ok((pte.ppn(), pte))
    }
    #[inline]
    pub fn remove(&mut self, asid: usize, va: VirtAddr) {
        match self.lookup(va) {
            Ok((ppn, pte)) => {
                frame_decref(ppn);
                *pte = Pte::new(0);
                tlb_invalidate(asid, va);
            },
            Err(_) => return,
        }
    }
    #[inline]
    pub fn insert(&mut self, asid: usize, ppn: PhysPageNum, va: VirtAddr, perm: usize) -> Result<(), Error>{
        if let Ok(pte) = self.walk_or_create(va, 0) {
            if pte.valid() {
                if pte.ppn() != ppn {
                    self.remove(asid, va);
                } else {
                    tlb_invalidate(asid, va);
                    *pte = Pte::new_from_ppn(ppn, perm | PTE_C_CACHEABLE | PTE_V);
                    return Ok(())
                }
            }
        }
        tlb_invalidate(asid, va);
        let pte = self.walk_or_create(va, 1)?;
        frame_incref(ppn);
        *pte = Pte::new_from_ppn(ppn, perm | PTE_C_CACHEABLE | PTE_V);
        Ok(())
    }

    #[inline]
    pub fn translate(&self, va: VirtAddr) -> Option<PhysAddr> {
        let pte = self.entries[va.pdx()];
        if !pte.valid() {
            None
        } else {
            let pt = unsafe{&*pte.ppn().into_kva().as_ptr::<PageTable>()};
            let pte = pt.entries[va.ptx()];
            if !pte.valid() {
                None
            } else {
                Some(PhysAddr::new_from_pte(pte, va.page_offset()))
            }
        }
    }
}

// from lab2_2
pub fn page_strong_check() {
    unsafe {
	let pp = frame_alloc().unwrap();
    let dirpg = pp;
    let boot_dir = &mut *pp.into_kva().as_mut_ptr::<PageTable>();

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
    
	// temporarily steal the rest of the free pages
	let mut fl: Option<PhysPageNum> = None;
    // now this page_free list must be empty!!!!
    steal(&mut fl);

	// there is no free memory, so we can't allocate a page table
	assert!(boot_dir.insert(0, pp1, VirtAddr::new(0), 0).is_err());

	// should be no free memory
	assert!(frame_alloc().is_err());

	// free pp0 and try again: pp0 should be used for page table
	frame_dealloc(pp0);

    println!("pp0 is {}", pp0.as_usize());
	// check if PTE != PP
	boot_dir.insert(0, pp1, VirtAddr::new(0), 0).unwrap();
	// should be able to map pp2 at PAGE_SIZE because pp0 is already allocated for page table
	assert!(boot_dir.insert(0, pp2, VirtAddr::new(PAGE_SIZE), 0).is_ok());
	assert!(boot_dir.insert(0, pp3, VirtAddr::new(2 * PAGE_SIZE), 0).is_ok());
	assert!(PhysAddr::from(boot_dir.entries[0]) == PhysAddr::from(pp0));

    println!("{}", pp1.as_usize());
	println!("boot_dir.translate(VirtAddr::new(0x0)) is {:p}", boot_dir.translate(VirtAddr::new(0)).unwrap());
	println!("page2pa(pp1) is {:p}", PhysAddr::from(pp1));

	assert!(boot_dir.translate(VirtAddr::new(0)).unwrap() == PhysAddr::from(pp1));
	assert!(get_frame_ref(pp1) == 1);
	assert!(boot_dir.translate(VirtAddr::new(PAGE_SIZE)).unwrap() == PhysAddr::from(pp2));
	assert!(get_frame_ref(pp2) == 1);
	assert!(boot_dir.translate(VirtAddr::new(2 * PAGE_SIZE)).unwrap() == PhysAddr::from(pp3));
	assert!(get_frame_ref(pp3) == 1);

	println!("start page_insert");
	// should be able to map pp2 at PAGE_SIZE because it's already there
	assert!(boot_dir.insert(0, pp2, VirtAddr::new(PAGE_SIZE), 0).is_ok());
	assert!(boot_dir.translate(VirtAddr::new(PAGE_SIZE)).unwrap() == PhysAddr::from(pp2));
	assert!(get_frame_ref(pp2) == 1);

	// should not be able to map at PDMAP because need free page for page table
	assert!(boot_dir.insert(0, pp0, VirtAddr::new(PDMAP), 0).is_err());
	// remove pp1 try again
    boot_dir.remove(0, VirtAddr::new(0));
	assert!(boot_dir.translate(VirtAddr::new(0)).is_none());
	assert!(boot_dir.insert(0, pp0, VirtAddr::new(PDMAP), 0).is_ok());

	// insert pp2 at 2*PAGE_SIZE (replacing pp2)
	assert!(boot_dir.insert(0, pp2, VirtAddr::new(2 * PAGE_SIZE), 0).is_ok());

	// should have pp2 at both 0 and PAGE_SIZE, pp2 nowhere, ...
	assert!(boot_dir.translate(VirtAddr::new(PAGE_SIZE)).unwrap() == PhysAddr::from(pp2));
	assert!(boot_dir.translate(VirtAddr::new(2 * PAGE_SIZE)).unwrap() == PhysAddr::from(pp2));
	// ... and ref counts should reflect this
	assert!(get_frame_ref(pp2) == 2);
	assert!(get_frame_ref(pp3) == 0);
	// try to insert PDMAP+PAGE_SIZE
	assert!(boot_dir.insert(0, pp2, VirtAddr::new(PDMAP + PAGE_SIZE), 0).is_ok());
	assert!(get_frame_ref(pp2) == 3);
	println!("end page_insert");

	// pp2 should be returned by page_alloc
    let pp = frame_alloc().unwrap();
	assert!(pp == pp3);

	// unmapping pp2 at PAGE_SIZE should keep pp1 at 2*PAGE_SIZE
	boot_dir.remove(0, VirtAddr::new(PAGE_SIZE));
	assert!(boot_dir.translate(VirtAddr::new(2 * PAGE_SIZE)).unwrap() == PhysAddr::from(pp2));
	assert!(get_frame_ref(pp2) == 2);
	assert!(get_frame_ref(pp3) == 0);

	// unmapping pp2 at 2*PAGE_SIZE should keep pp2 at PDMAP+PAGE_SIZE
	boot_dir.remove(0,VirtAddr::new(2 * PAGE_SIZE));
	assert!(boot_dir.translate(VirtAddr::new(0x0)).is_none());
	assert!(boot_dir.translate(VirtAddr::new(PAGE_SIZE)).is_none());
	assert!(boot_dir.translate(VirtAddr::new(2 * PAGE_SIZE)).is_none());
	assert!(get_frame_ref(pp2) == 1);
	assert!(get_frame_ref(pp3) == 0);

	// unmapping pp2 at PDMAP+PAGE_SIZE should free it
	boot_dir.remove(0, VirtAddr::new(PDMAP + PAGE_SIZE));
	assert!(boot_dir.translate(VirtAddr::new(0x0)).is_none());
	assert!(boot_dir.translate(VirtAddr::new(PAGE_SIZE)).is_none());
	assert!(boot_dir.translate(VirtAddr::new(2 * PAGE_SIZE)).is_none());
	assert!(boot_dir.translate(VirtAddr::new(PDMAP + PAGE_SIZE)).is_none());
	assert!(get_frame_ref(pp2) == 0);

	// so it should be returned by page_alloc
    let pp = frame_alloc().unwrap();
	assert!(pp == pp2);

	// should be no free memory
	assert!(frame_alloc().is_err());

	// forcibly take pp0 and pp1  back
	assert!(PhysAddr::from(boot_dir.entries[0]) == PhysAddr::from(pp0));
	assert!(PhysAddr::from(boot_dir.entries[1]) == PhysAddr::from(pp1));
	boot_dir.entries[0].0 = 0;
	boot_dir.entries[1].0 = 0;
	assert!(get_frame_ref(pp0) == 2);
	assert!(get_frame_ref(pp1) == 1);
	set_frame_ref(pp0, 0);
    set_frame_ref(pp1, 0);

	// give free list back
	recover(fl);

	// free the pages we took
	frame_dealloc(pp0);
    frame_dealloc(pp1);
    frame_dealloc(pp2);
    frame_dealloc(pp3);
    frame_dealloc(pp4);
    //let tmp = pmap.pa2page();
	//pmap.page_free(tmp);
    frame_dealloc(dirpg);

	println!("page_check_strong() succeeded!");
    }
}
