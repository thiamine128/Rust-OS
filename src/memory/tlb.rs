
use crate::{env::{cur_pgdir, ASID}, exception::traps::Trapframe, util::bitops::genmask};

use super::mmu::{VirtAddr, NASID, PGSHIFT};

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