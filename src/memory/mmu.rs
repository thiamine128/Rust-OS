use core::{fmt, iter::Step, ops::{Add, AddAssign}};

use super::page_table::Pte;

/// the number of ASIDs
pub const NASID: usize = 256;
/// mips page size
pub const PAGE_SIZE: usize = 4096;
pub const PTMAP: usize = PAGE_SIZE;
pub const PDMAP: usize = 4 * 1024 * 1024;
/// page shift
pub const PGSHIFT: usize = 12;
/// page diretory shift
pub const PDSHIFT: usize = 22;
/// page table entry hard flag shift
pub const PTE_HARDFLAG_SHIFT: usize = 6;
pub const PTE_G: usize = 0x0001 << PTE_HARDFLAG_SHIFT;
pub const PTE_V: usize = 0x0002 << PTE_HARDFLAG_SHIFT;
pub const PTE_D: usize = 0x0004 << PTE_HARDFLAG_SHIFT;
pub const PTE_C_CACHEABLE: usize = 0x0018 << PTE_HARDFLAG_SHIFT;
pub const PTE_C_UNCACHEABLE: usize = 0x0010 << PTE_HARDFLAG_SHIFT;
pub const PTE_COW: usize = 0x0001;
pub const PTE_LIBRARY: usize = 0x0002;
pub const KUSEG: usize = 0x00000000;
pub const KSEG0: usize = 0x80000000;
pub const KSEG1: usize = 0xA0000000;
pub const KSEG2: usize = 0xC0000000;
pub const KERNBASE: usize = 0x80020000;
pub const KSTACKTOP: usize = ULIM + PDMAP;
pub const ULIM: usize = 0x80000000;
pub const UVPT: VirtAddr = VirtAddr::new(ULIM - PDMAP);
pub const UPAGES: VirtAddr = VirtAddr::new(UVPT.0 - PDMAP);
pub const UENVS: VirtAddr = VirtAddr::new(UPAGES.0 - PDMAP);
pub const UTOP: VirtAddr = UENVS;
pub const UXSTACKTOP: VirtAddr = UTOP;
pub const USTACKTOP: VirtAddr = VirtAddr::new(UTOP.0 - 2 * PTMAP);
pub const UTEXT: usize = PDMAP;
pub const UCOW: usize = UTEXT - PTMAP;
pub const UTEMP: usize = UCOW - PTMAP;

/// Physical address, wrapped numeric value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct PhysAddr(usize);

/// Virtual address, wrapped numeric value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct VirtAddr(usize);

/// Physical page number, wrapped numeric value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct PhysPageNum(usize);

/// Virtual page number, wrapped numeric value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct VirtPageNum(usize);

impl VirtAddr {
    /// create a new virtual address from numeric value.
    #[inline]
    pub const fn new(addr: usize) -> Self{
        Self(addr)
    }
    #[inline]
    pub fn zero() -> Self {
        Self(0)
    }
    /// the raw value of virtual address.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0
    }
    /// create a virtual address from a rust pointer.
    #[inline]
    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        Self::new(ptr as *const() as usize)
    }
    /// the const pointer to this address.
    #[inline]
    pub const fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }
    /// the mut pointer to this address.
    #[inline]
    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
    /// check if the address is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
    /// align up.
    #[inline]
    pub const fn align_up(self, align: usize) -> Self {
        Self::new((self.0 + align - 1) & !(align - 1))
    }
    /// page directory index
    #[inline]
    pub const fn pdx(self) -> usize {
        (self.0 >> PDSHIFT) & 0x3ff
    }
    /// page table index
    #[inline]
    pub const fn ptx(self) -> usize {
        (self.0 >> PGSHIFT) & 0x3ff
    }
    #[inline]
    pub const fn page_align_down(self) -> Self {
        Self::new(self.0 & !0xfff)
    }
    #[inline]
    pub const fn page_offset(self) -> usize {
        self.0 & 0xfff
    }
    #[inline]
    pub const fn is_aligned(self, align: usize) -> bool {
        self.0 % align == 0
    }
}

impl fmt::Pointer for VirtAddr {
    /// for convenient print.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl fmt::Pointer for PhysAddr {
    /// for convenient print.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<usize> for VirtAddr {
    type Output = VirtAddr;

    fn add(self, rhs: usize) -> Self::Output {
        Self::Output::new(self.0 + rhs)
    }
}

impl AddAssign<usize> for VirtAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl PhysAddr {
    /// create a new virtual address from numeric value.
    #[inline]
    pub const fn new(addr: usize) -> Self{
        Self(addr)
    }
    #[inline]
    pub const fn new_from_pte(pte: Pte, offset: usize) -> Self {
        Self((pte.ppn().as_usize() << PGSHIFT) + offset)
    }
    /// the raw value of this physical address.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0
    }
    /// convert kernel virtual address to physical address, panic if invalid.
    #[inline]
    pub fn from_kva(kva: VirtAddr) -> Self {
        if kva.as_usize() < ULIM {
            panic!("kva2pa called with invalid kva {:p}", kva);
        } else {
            Self::new(kva.0 - ULIM)
        }
    }
    /// convert the physical address to kernel virtual address.
    #[inline]
    pub const fn into_kva(self) -> VirtAddr {
        VirtAddr::new(self.0 + ULIM)
    }

    #[inline]
    pub const fn is_aligned(self, align: usize) -> bool {
        self.0 % align == 0
    }
}

impl Add<usize> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: usize) -> Self::Output {
        Self::Output::new(self.0 + rhs)
    }
}

impl AddAssign<usize> for PhysAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl PhysPageNum {
    /// create a new virtual address from numeric value.
    #[inline]
    pub const fn new(ppn: usize) -> Self{
        Self(ppn)
    }
    //// the raw value of ppn.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0
    }
}

impl VirtPageNum {
    /// create a new virtual address from numeric value.
    #[inline]
    pub const fn new(vpn: usize) -> Self{
        Self(vpn)
    }
    /// the raw value of vpn.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0
    }
}


impl From<Pte> for PhysAddr {
    /// convert pte to physical address
    fn from(value: Pte) -> Self {
        Self::new(value.as_usize() & !0xfff)
    }
}

impl Add<usize> for PhysPageNum {
    type Output = PhysPageNum;

    fn add(self, rhs: usize) -> Self::Output {
        Self::Output::new(self.0 + rhs)
    }
}

impl AddAssign<usize> for PhysPageNum {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl From<PhysAddr> for PhysPageNum {
    /// convert physcial address to ppn
    fn from(value: PhysAddr) -> Self {
        Self::new(value.0 >> PGSHIFT)
    }
}

impl From<PhysPageNum> for PhysAddr {
    /// convert ppn to physcial address
    fn from(value: PhysPageNum) -> Self {
        Self::new(value.0 << PGSHIFT)
    }
}

impl Step for PhysPageNum {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        if *start <= *end {
            Some(end.0 - start.0)
        } else {
            None
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        match usize::checked_add(start.0, count) {
            Some(n) => Some(Self::new(n)),
            None => None,
        }
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        match usize::backward_checked(start.0, count) {
            Some(n) => Some(Self::new(n)),
            None => None
        }
    }
}

impl PhysPageNum {
    #[inline]
    /// convert ppn to kva
    pub fn into_kva(self) -> VirtAddr {
        PhysAddr::from(self).into_kva()
    }
}

/*
#[inline]
pub fn va2pa(pgdir: *mut usize, va: usize) -> usize{
    unsafe {
    
    }
}*/