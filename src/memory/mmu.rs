use super::pmap::{Pde, Pte, PMAP};

pub const NASID: usize = 256;
pub const PAGE_SIZE: usize = 4096;
pub const PTMAP: usize = PAGE_SIZE;
pub const PDMAP: usize = 4 * 1024 * 1024;
pub const PGSHIFT: usize = 12;
pub const PDSHIFT: usize = 22;
pub const PTE_HARDFLAG_SHIFT: usize = 6;
pub const PTE_G: usize = 0x0001 << PTE_HARDFLAG_SHIFT;
pub const PTE_V: usize = 0x0002 << PTE_HARDFLAG_SHIFT;
pub const PTE_D: usize = 0x0004 << PTE_HARDFLAG_SHIFT;
pub const PTE_C_CACHEABLE: usize = (0x0018 << PTE_HARDFLAG_SHIFT);
pub const PTE_C_UNCACHEABLE: usize = (0x0010 << PTE_HARDFLAG_SHIFT);
pub const PTE_COW: usize = 0x0001;
pub const PTE_LIBRARY: usize = 0x0002;
pub const KUSEG: usize = 0x00000000;
pub const KSEG0: usize = 0x80000000;
pub const KSEG1: usize = 0xA0000000;
pub const KSEG2: usize = 0xC0000000;
pub const KERNBASE: usize = 0x80020000;
pub const KSTACKTOP: usize = ULIM + PDMAP;
pub const ULIM: usize = 0x80000000;
pub const UVPT: usize = ULIM - PDMAP;
pub const UPAGES: usize = UVPT - PDMAP;
pub const UENVS: usize = UPAGES - PDMAP;
pub const UTOP: usize = UENVS;
pub const UXSTACKTOP: usize = UTOP;
pub const USTACKTOP: usize = UTOP - 2 * PTMAP;
pub const UTEXT: usize = PDMAP;
pub const UCOW: usize = UTEXT - PTMAP;
pub const UTEMP: usize = UCOW - PTMAP;


#[inline]
pub fn kva2pa(kva: usize) -> usize {
    if kva < ULIM {
        panic!("kva2pa called with invalid kva {:x}", kva);
    } else {
        kva - ULIM
    }
}

#[inline]
pub fn pte_addr(pte: Pte) -> usize {
    pte.0 & !0xfff
}

#[inline]
pub fn pa2kva(pa: usize) -> usize {
    pa + ULIM
}

#[inline]
pub fn ppn(pa: usize) -> usize {
    pa >> PGSHIFT
}

#[inline]
pub fn round(a: usize, n: usize) -> usize {
    (a + n - 1) & !(n - 1)
}

#[inline]
pub fn pdx(va: usize) -> usize {
    (va >> PDSHIFT) & 0x3ff
}

#[inline]
pub fn ptx(va: usize) -> usize {
    (va >> PGSHIFT) & 0x3ff
}

#[inline]
pub fn pte_flags(pte: usize) -> usize {
    pte & 0xfff
}

#[inline]
pub fn va2pa(pgdir: *mut Pde, va: usize) -> usize{
    unsafe {
    let pgdir = pgdir.add(pdx(va));
    if ((*pgdir).0 & PTE_V) == 0 {
        !0
    } else {
        let p = pa2kva(pte_addr(Pte((*pgdir).0))) as *mut usize;
        if ((*p.add(ptx(va))) & PTE_V) == 0 {
            !0
        } else {
            pte_addr(Pte(*p.add(ptx(va))))
        }
    }
    }
}