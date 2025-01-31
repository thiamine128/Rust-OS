use core::ptr::addr_of_mut;

use crate::{env::ASID, err::Error};

use super::{frame::*, mmu::*, tlb::tlb_invalidate};
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
    pub entries: [Pte; PAGE_TABLE_ENTRIES]
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
    /// valid flag
    #[inline]
    pub const fn valid(self) -> bool {
        self.0 & PTE_V != 0
    }
    /// do fill tlb entry
    #[inline]
    pub fn fill_tlb_entry(&mut self, pentrylo: &mut [usize; 2]) {
        let ppte: usize = addr_of_mut!(*self) as usize;
        let ppte = ppte & !0x7;
        let ptr0 = ppte as *mut usize;
        let ptr1 = (ppte + 4) as *mut usize;
        pentrylo[0] = (unsafe { *ptr0 } as usize) >> 6;
        pentrylo[1] = (unsafe { *ptr1 } as usize) >> 6;
    }
    /// to physical address
    #[inline]
    pub const fn addr(self) -> PhysAddr {
        PhysAddr::new(self.0 & !0xfff)
    }
}

impl PageTable {
    /// create a new page table
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: [Pte::new(0); PAGE_TABLE_ENTRIES]
        }
    }
    /// walk or create entry for va
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
        let pt_addr = ppn.into_kva().as_mut_ptr();
        let page_table: &mut PageTable = unsafe { &mut *pt_addr };
        Ok(&mut page_table.entries[va.ptx()])
    }

    /// look up entry for va
    #[inline]
    pub fn lookup(&mut self, va: VirtAddr) -> Result<(PhysPageNum, &mut Pte), Error> {
        let pte = self.walk_or_create(va, 0)?;
        if !pte.valid() {
            Err(Error::NotMapped)
        } else {
            Ok((pte.ppn(), pte))
        }
    }
    /// look up ppn for va
    #[inline]
    pub fn lookup_ppn(&mut self, va: VirtAddr) -> Result<PhysPageNum, Error> {
        let pte = self.walk_or_create(va, 0)?;
        Ok(pte.ppn())
    }

    /// remove address mapping from page table
    #[inline]
    pub fn remove(&mut self, asid: ASID, va: VirtAddr) {
        match self.lookup(va) {
            Ok((ppn, pte)) => {
                frame_decref(ppn);
                *pte = Pte::new(0);
                tlb_invalidate(asid, va);
            },
            Err(_) => return,
        }
    }
    /// map address to a frame
    #[inline]
    pub fn insert(&mut self, asid: ASID, ppn: PhysPageNum, va: VirtAddr, perm: usize) -> Result<(), Error>{
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

    /// translate virtual address to physical address
    #[inline]
    pub fn translate(&self, va: VirtAddr) -> Option<PhysAddr> {
        let pte = self.entries[va.pdx()];
        if !pte.valid() {
            None
        } else {
            let pt_addr = pte.ppn().into_kva().as_ptr::<PageTable>();
            let pt = unsafe{ &*pt_addr };
            let pte = pt.entries[va.ptx()];
            if !pte.valid() {
                None
            } else {
                Some(PhysAddr::new_from_pte(pte, va.page_offset()))
            }
        }
    }

    /// map segment in page table
    #[inline]
    pub fn map_segment(&mut self, asid: ASID, pa: PhysAddr, va: VirtAddr, size: usize, perm: usize) {

        assert!(pa.is_aligned(PAGE_SIZE));
        assert!(va.is_aligned(PAGE_SIZE));
        assert!(size % PAGE_SIZE == 0);
        for i in (0..size).step_by(PAGE_SIZE) {
            self.insert(asid, PhysPageNum::from(pa + i), va + i, perm).unwrap();
        }
    }

    /// set table entry
    #[inline]
    pub fn set_entry(&mut self, ptx: usize, pte: Pte) {
        self.entries[ptx] = pte;
    }

    /// get table entry
    #[inline]
    pub fn get_entry(&self, ptx: usize) -> Pte {
        self.entries[ptx]
    }

    /// alloc frames passively
    fn passive_alloc(&mut self, va: VirtAddr, asid: ASID) {
        if va < UTEMP {
            panic!("address too low");
        }
    
        if va >= USTACKTOP && va < USTACKTOP + PAGE_SIZE {
            panic!("invalid memory");
        }
    
        if va >= UENVS && va < UPAGES {
            panic!("envs zone");
        }
    
        if va >= UPAGES && va  < UVPT {
            panic!("pages zone");
        }
    
        if va.as_usize() >= ULIM {
            panic!("kernel address");
        }
        let ppn = frame_alloc().unwrap();
        frame_incref(ppn);
        self.insert(asid, ppn, va.page_align_down(), 
            if va >= UVPT && va.as_usize() < ULIM {
                0
            } else {
                PTE_D
            }).unwrap();
    }

    /// do tlb refill according to page table
    #[inline]
    pub fn do_tlb_refill(&mut self, entries: &mut [usize; 2], va: VirtAddr, asid: ASID) {
        tlb_invalidate(asid, va);

        let pte = loop {
            if let Ok((_, pte)) = self.lookup(va) {
                break pte;
            }
            self.passive_alloc(va, asid);
        };

        pte.fill_tlb_entry(entries);
    }
}