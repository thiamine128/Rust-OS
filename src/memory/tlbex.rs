use crate::println;

use crate::bitops::genmask;

use super::{mmu::{pte_addr, NASID, PAGE_SIZE, PGSHIFT, PTE_D, UENVS, ULIM, UPAGES, USTACKTOP, UTEMP, UVPT}, pmap::{PMap, Pde, Pte, PMAP}};

extern "C" {
    fn tlb_out(entry: usize);
}

pub fn tlb_invalidate(asid: usize, va: usize) {
    unsafe {
        tlb_out((va & !genmask(PGSHIFT, 0)) | (asid & (NASID - 1)));
    }
}

fn passive_alloc(va: usize, pgdir: *mut Pde, asid: usize) {
    if va < UTEMP {
		panic!("address too low");
	}

	if va >= USTACKTOP && va < USTACKTOP + PAGE_SIZE {
		panic!("invalid memory");
	}

	if va >= UENVS && va < UPAGES {
		panic!("envs zone");
	}

	if va >= UPAGES && va < UVPT {
		panic!("pages zone");
	}

	if va >= ULIM {
		panic!("kernel address");
	}
    let mut pmap = PMAP.lock();
    let pp = pmap.page_alloc().unwrap();
    pmap.page_insert(pgdir, asid, pp, pte_addr(Pte(va)), 
        if va >= UVPT && va < ULIM {
            0
        } else {
            PTE_D
        }).unwrap();
}


#[no_mangle]
pub extern "C" fn _do_tlb_refill(pentrylo: *mut usize, va:usize , asid: usize) {
	tlb_invalidate(asid, va);
    let cur_pgdir = PMAP.lock().cur_pgdir;
	let mut ppte = loop {
        let (pp, ppte) = PMAP.lock().page_lookup(cur_pgdir, va);
        if !pp.is_null() {
            break ppte;
        }
		passive_alloc(va, cur_pgdir, asid);
	};
	ppte = ((ppte as usize) & !0x7) as *mut Pte;
    unsafe {
        *pentrylo = (*ppte).0 >> 6;
        *pentrylo.offset(1) = (*ppte.offset(1)).0 >> 6;
    }
}
