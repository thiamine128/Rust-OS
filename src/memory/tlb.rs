use core::{mem::size_of, ptr::addr_of_mut, result, slice};

use crate::{env::{cur_pgdir, curenv_id, env_asid, user_tlb_mod_entry, ASID}, err::Error, exception::traps::Trapframe, print, println, util::bitops::genmask};

use super::{frame::{frame_alloc, frame_incref}, mmu::{VirtAddr, NASID, PAGE_SIZE, PGSHIFT, PTE_D, UENVS, ULIM, UPAGES, USTACKTOP, UTEMP, UVPT, UXSTACKTOP}, page_table::{PageTable, Pte}};

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
    pgdir.insert(asid, ppn, va.page_align_down(), 
        if va >= UVPT && va.as_usize() < ULIM {
            0
        } else {
            PTE_D
        }).unwrap();
}


#[no_mangle]
pub extern "C" fn _do_tlb_refill(entries: &mut [usize; 2], va: VirtAddr, asid: ASID) {
    cur_pgdir(|pgdir| {
        tlb_invalidate(asid, va);

        let pte = loop {
            if let Ok((_, pte)) = pgdir.lookup(va) {
                break pte;
            }
            passive_alloc(va, pgdir, asid);
        };

        pte.fill_tlb_entry(entries);
    })
}

#[no_mangle]
pub extern "C" fn do_tlb_mod(tf: &mut Trapframe) {
    let tmp_tf = tf.clone();
    let sp = VirtAddr::new(tf.regs[29]);
    if sp < USTACKTOP || sp >= UXSTACKTOP {
        tf.regs[29] = UXSTACKTOP.as_usize();
    }
    
    tf.regs[29] -= size_of::<Trapframe>();
    let sp: *mut Trapframe = VirtAddr::new(tf.regs[29]).as_mut_ptr();
    let t = unsafe {sp.as_mut()}.unwrap();
    *t = tmp_tf;

    let mod_entry = user_tlb_mod_entry();
    if mod_entry != 0 {
        tf.regs[4] = tf.regs[29];
        tf.regs[29] -= 4;
        tf.cp0_epc = mod_entry;
    } else {
        panic!("TLB Mod but no user handler registered");
    }
    
}