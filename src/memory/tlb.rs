use core::mem::size_of;

use crate::{env::{cur_pgdir, user_tlb_mod_entry, ASID}, exception::traps::Trapframe, util::bitops::genmask};

use super::{frame::{frame_alloc, frame_incref}, mmu::{VirtAddr, NASID, PAGE_SIZE, PGSHIFT, PTE_D, UENVS, ULIM, UPAGES, USTACKTOP, UTEMP, UVPT, UXSTACKTOP}, page_table::PageTable};

extern "C" {
    fn tlb_out(entry: usize);
}

#[inline]
pub fn tlb_invalidate(asid: ASID, va: VirtAddr) {
    let entry = (va.as_usize() & !genmask(PGSHIFT, 0)) | (asid.as_usize() & (NASID - 1));
    unsafe { tlb_out(entry); }
}


#[no_mangle]
pub extern "C" fn _do_tlb_refill(entries: &mut [usize; 2], va: VirtAddr, asid: ASID) {
    cur_pgdir(|pgdir| {
        pgdir.do_tlb_refill(entries, va, asid);
    })
}

#[no_mangle]
pub extern "C" fn do_tlb_mod(tf: &mut Trapframe) {
    tf.do_tlb_mod();
}