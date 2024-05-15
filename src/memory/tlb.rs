use crate::{env::{cur_pgdir, ASID}, util::bitops::genmask};

use super::{frame::{frame_alloc, frame_incref}, mmu::{VirtAddr, NASID, PAGE_SIZE, PGSHIFT, PTE_D, UENVS, ULIM, UPAGES, USTACKTOP, UTEMP, UVPT}, page_table::PageTable};

extern "C" {
    fn tlb_out(entry: usize);
}

#[inline]
pub fn tlb_invalidate(asid: ASID, va: VirtAddr) {
    let entry = (va.as_usize() & !genmask(PGSHIFT, 0)) | (asid.as_usize() & (NASID - 1));
    unsafe {
        tlb_out(entry);
    }
}

fn passive_alloc(va: VirtAddr, pgdir: &mut PageTable, asid: ASID) {
    if va.as_usize() < UTEMP {
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
    pgdir.insert(asid, ppn, va.page_align_down(), 
        if va >= UVPT && va.as_usize() < ULIM {
            0
        } else {
            PTE_D
        }).unwrap();
}


#[no_mangle]
pub extern "C" fn _do_tlb_refill(pentrylo: &mut [usize; 2], va: VirtAddr, asid: ASID) {
    cur_pgdir(|pgdir| {
        tlb_invalidate(asid, va);

        let pte = loop {
            if let Ok((_, pte)) = pgdir.lookup(va) {
                break pte;
            }
            passive_alloc(va, pgdir, asid);
        };
        
        pte.fill_tlb_entry(pentrylo);
    })
}