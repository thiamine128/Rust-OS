use core::ptr::addr_of_mut;

use crate::bitops::genmask;

use super::{frame::{frame_alloc, frame_incref}, mmu::{VirtAddr, NASID, PAGE_SIZE, PGSHIFT, PTE_D, UENVS, ULIM, UPAGES, USTACKTOP, UTEMP, UVPT}, page_table::{cur_pgdir, PageTable}};

extern "C" {
    fn tlb_out(entry: usize);
}

#[inline]
pub fn tlb_invalidate(asid: usize, va: VirtAddr) {
    unsafe {
        tlb_out((va.as_usize() & !genmask(PGSHIFT, 0)) | (asid & (NASID - 1)));
    }
}

fn passive_alloc(va: VirtAddr, pgdir: &mut PageTable, asid: usize) {
    if va.as_usize() < UTEMP {
		panic!("address too low");
	}

	if va.as_usize() >= USTACKTOP && va.as_usize() < USTACKTOP + PAGE_SIZE {
		panic!("invalid memory");
	}

	if va.as_usize() >= UENVS && va.as_usize() < UPAGES {
		panic!("envs zone");
	}

	if va.as_usize() >= UPAGES && va.as_usize() < UVPT {
		panic!("pages zone");
	}

	if va.as_usize() >= ULIM {
		panic!("kernel address");
	}
    let ppn = frame_alloc().unwrap();
    frame_incref(ppn);
    pgdir.insert(asid, ppn, va.page_align_down(), 
        if va.as_usize() >= UVPT && va.as_usize() < ULIM {
            0
        } else {
            PTE_D
        }).unwrap();
}


#[no_mangle]
pub extern "C" fn _do_tlb_refill(pentrylo: &mut [usize; 2], va: VirtAddr, asid: usize) {
	tlb_invalidate(asid, va);

	let pte = loop {
        if let Ok((_, pte)) = cur_pgdir().unwrap().lookup(va) {
            break pte;
        }
		passive_alloc(va, cur_pgdir().unwrap(), asid);
	};
    unsafe {
        let mut ppte: *mut usize = addr_of_mut!(*pte) as *mut usize;
        ppte = ((ppte as usize) & !0x7) as *mut usize;
        pentrylo[0] = (*ppte as usize) >> 6;
        pentrylo[1] = (*ppte.offset(1) as usize) >> 6;
    }
}