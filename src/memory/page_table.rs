use crate::err::Error;

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
        let pt_addr = ppn.into_kva().as_mut_ptr();
        let page_table: &mut PageTable = unsafe {&mut *pt_addr};
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
}