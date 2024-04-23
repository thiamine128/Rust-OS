use core::{mem::size_of, ptr::{addr_of_mut, null, null_mut}};

use lazy_static::lazy_static;
use spin::mutex::Mutex;

use crate::{error::Error, list::linked_list::{Head, Link}, list_empty, list_first, list_insert_head, list_remove, memory::{mmu::{kva2pa, round, PAGE_SIZE}}, println, string::memset};

use super::{mmu::{pa2kva, pdx, ppn, pte_addr, ptx, PGSHIFT, PTE_C_CACHEABLE, PTE_V}, tlbex::tlb_invalidate};

lazy_static! {
    pub static ref PMAP: Mutex<PMap> = Mutex::new(PMap{
        memsize: 0,
        npage: 0,
        freemem: 0,
        pages: null_mut(),
        page_free_list: Head::new(),
        cur_pgdir: null_mut()
    });
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Pde(pub usize);
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Pte(pub usize);


impl From<Pde> for Pte {
    fn from(value: Pde) -> Self {
        Self(value.0)
    }
}
pub struct PhysPageNum(usize);

pub struct Page {
    pub pp_link: Link<Page>,
    pub pp_ref: u16
}

pub struct PMap {
    memsize: usize,
    npage: usize,
    freemem: usize,
    pages: *mut Page,
    pub page_free_list: Head<Page>,
    pub cur_pgdir: *mut Pde
}

unsafe impl Send for PMap {}

impl PMap {
    pub fn alloc(&mut self, n: usize, align: usize, clear: i32) -> *mut u8 {
        extern "C" {
            fn end();
        }
        if self.freemem == 0 {
            self.freemem = end as usize;
        }
        self.freemem = round(self.freemem, align);
        let allocted_mem = self.freemem as *mut u8;
        self.freemem = self.freemem + n;
        assert!(kva2pa(self.freemem) < self.memsize);
        if clear != 0 {
            memset(allocted_mem, 0, n);
        }
        allocted_mem
    }
    pub fn mips_detect_memory(&mut self, memsize: usize) {
        self.memsize = memsize;
        self.npage = self.memsize / PAGE_SIZE;
        println!("Memory size: {} KiB, number of pages: {}", self.memsize / 1024, self.npage);
    }
    pub fn mips_vm_init(&mut self) {
        self.pages = self.alloc(self.npage * size_of::<Page>(), PAGE_SIZE, 1) as *mut Page;
        println!("to memory {:x} for struct Pages.", self.freemem);
	    println!("pmap:\t mips vm init success.");
    }
    pub fn page_init(&mut self) {
        self.freemem = round(self.freemem, PAGE_SIZE);
        for i in 0..ppn(kva2pa(self.freemem)) {
            unsafe {
                self.pages.add(i).as_mut().unwrap().pp_ref = 1;
            }
        }
        for i in ppn(kva2pa(self.freemem))..self.npage {
            unsafe {
                let page = self.pages.add(i).as_mut().unwrap();
                page.pp_ref = 0;
                list_insert_head!(self.page_free_list, page, pp_link);
            }
        }
    }
    pub fn page_alloc(&mut self) -> Result<*mut Page, Error> {
        if list_empty!(self.page_free_list) {
            Err(Error::E_NO_MEM)
        } else {
            let page = list_first!(self.page_free_list);
            list_remove!(*page, pp_link);
            memset(self.page2kva(page) as *mut u8, 0, PAGE_SIZE);
            Ok(page)
        }
    }
    pub fn page_free(&mut self, page: *mut Page) {
        unsafe {
            assert!(page.as_ref().unwrap().pp_ref == 0);
            list_insert_head!(self.page_free_list, page.as_mut().unwrap(), pp_link);
        }
    }
    pub fn pgdir_walk(&mut self, pgdir: *mut Pde, va: usize, create: i32) -> Result<*mut Pte, Error> {
        unsafe {
            let pgdir_entry = pgdir.add(pdx(va));
            if (*pgdir_entry).0 & PTE_V == 0 {
                if create != 0 {
                    match self.page_alloc() {
                        Ok(page) => {
                            (*pgdir_entry).0 = self.page2pa(page) | PTE_C_CACHEABLE | PTE_V;
                            (*page).pp_ref += 1;
                        },
                        Err(err) => return Err(err),
                    }
                } else {
                    return Ok(null_mut())
                }
            }  
            Ok((pa2kva(pte_addr(Pte::from(*pgdir_entry))) as *mut Pte).add(ptx(va)))
        }
    }
    pub fn page_insert(&mut self, pgdir: *mut Pde, asid: usize, pp: *mut Page, va: usize, perm: usize) -> Result<(), Error>{
        match self.pgdir_walk(pgdir, va, 0) {
            Ok(pte) => unsafe {
                if !pte.is_null() && ((*pte).0 & PTE_V) != 0 {
                    if self.pa2page((*pte).0) != pp {
                        self.page_remove(pgdir, asid, va);
                    } else {
                        tlb_invalidate(asid, va);
                        *pte = Pte(self.page2pa(pp) | perm | PTE_C_CACHEABLE | PTE_V);
                        return Ok(())
                    }
                }
            },
            Err(err) => return Err(err),
        }
        tlb_invalidate(asid, va);
        match self.pgdir_walk(pgdir, va, 1) {
            Ok(pte) => unsafe {
                *pte = Pte(self.page2pa(pp) | perm | PTE_C_CACHEABLE | PTE_V);
                (*pp).pp_ref += 1;
                return Ok(());
            },
            Err(err) => return Err(err)
        }
    }
    pub fn page_lookup(&mut self, pgdir: *mut Pde, va: usize) -> (*mut Page, *mut Pte) {
        let pte = self.pgdir_walk(pgdir, va, 0).unwrap();
        unsafe {
            if pte.is_null() || ((*pte).0 & PTE_V) == 0 {
                (null_mut(), null_mut())
            } else {
                let pp = self.pa2page((*pte).0);
                (pp, pte)
            }
        }
    }
    pub fn page_decref(&mut self, pp: *mut Page) {
        unsafe {
            assert!((*pp).pp_ref > 0);
            (*pp).pp_ref -= 1;
            if (*pp).pp_ref == 0 {
                self.page_free(pp);
            }
        }
    }
    pub fn page_remove(&mut self, pgdir: *mut Pde, asid: usize, va: usize) {
        let (pp, pte) = self.page_lookup(pgdir, va);
        if pp.is_null() {
            return;
        }
        self.page_decref(pp);
        unsafe {
            (*pte).0 = 0;
        }
        tlb_invalidate(asid, va);
        return;
    }
    pub fn page2pa(&mut self, page: *mut Page) -> usize {
        unsafe {
            (page.offset_from(self.pages) as usize) << PGSHIFT
        }
    }
    pub fn pa2page(&mut self, pa: usize) -> *mut Page {
        unsafe {
            self.pages.add(ppn(pa))
        }
    }
    pub fn page2kva(&mut self, page: *mut Page) -> usize {
        pa2kva(self.page2pa(page))
    }
}