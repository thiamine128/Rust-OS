use core::arch::asm;

use crate::{memory::{frame::{frame_alloc, frame_dealloc, steal}, mmu::{PhysAddr, PhysPageNum, VirtAddr, PAGE_SIZE}, page_table::{set_cur_pgdir, PageTable}, tlb::_do_tlb_refill}, println};


extern "C" {
    fn do_tlb_refill_call(a: usize, b: usize, c: usize);
}

pub fn lab2_3() {
    tlb_refill_check();
}
pub fn tlb_refill_check() {
	unsafe {
        
	// should be able to allocate a page for directory
	let pp = frame_alloc().unwrap();
	let boot_pgdir: &mut PageTable = &mut *(pp.into_kva().as_mut_ptr());
	set_cur_pgdir(boot_pgdir);

	// should be able to allocate three pages
    let pp0 = frame_alloc().unwrap();
    let pp1 = frame_alloc().unwrap();
    let pp2 = frame_alloc().unwrap();
    let pp3 = frame_alloc().unwrap();
    let pp4 = frame_alloc().unwrap();


	// temporarily steal the rest of the free pages
	// now this page_free list must be empty!!!!
	let mut fl: Option<PhysPageNum> = None;
    steal(&mut fl);
	// free pp0 and try again: pp0 should be used for page table
	frame_dealloc(pp0);
	// check if PTE != PP
	assert!(boot_pgdir.insert(0, pp1, VirtAddr::new(0x0), 0).is_ok());
	// should be able to map pp2 at PAGE_SIZE because pp0 is already allocated for page table
	assert!(boot_pgdir.insert(0, pp2, VirtAddr::new(PAGE_SIZE), 0).is_ok());

	println!("tlb_refill_check() begin!");

    let mut entrys: [usize; 2] = [0, 0];
    _do_tlb_refill(&mut entrys, VirtAddr::new(PAGE_SIZE), 0);
    
    let (walk_page, walk_pte) = boot_pgdir.lookup(VirtAddr::new(PAGE_SIZE)).unwrap();
    assert!((entrys[0] == (walk_pte.as_usize() >> 6)) as i32 + (entrys[1] == walk_pte.as_usize() >> 6) as i32 == 1);
    assert!(PhysAddr::from(pp2) == boot_pgdir.translate(VirtAddr::new(PAGE_SIZE)).unwrap());

	println!("test point 1 ok");

	frame_dealloc(pp4);
    frame_dealloc(pp3);
    
    assert!(boot_pgdir.lookup(VirtAddr::new(0x00400000)).is_err());

    
	_do_tlb_refill(&mut entrys, VirtAddr::new(0x00400000), 0);
    let (pp, walk_pte) = boot_pgdir.lookup(VirtAddr::new(0x00400000)).unwrap();
	assert!(boot_pgdir.translate(VirtAddr::new(0x00400000)).unwrap() == PhysAddr::from(pp3));
    
	println!("test point 2 ok");

	let mut badva = 0usize;
    let mut entryhi = 0usize;
    let mut entrylo = 0usize;
	let mut index = 0usize;
	badva = 0x00400000;
	entryhi = badva & 0xffffe000;
    asm!("mtc0 {}, $10", out(reg) entryhi);
    
	//asm volatile("mtc0 %0, $10" : : "r"(entryhi));
	do_tlb_refill_call(0, badva, entryhi);
    println!("live");
	entrylo = 0;
	index = 0xffffffff;
	badva = 0x00400000;
	entryhi = badva & 0xffffe000;
    
    asm!("mtc0 {}, $10", in(reg) entryhi);
    asm!("mtc0 {}, $10", in(reg) index);
    asm!("tlbp");
    asm!("nop");
	/*asm volatile("mtc0 %0, $10" : : "r"(entryhi));
	asm volatile("mtc0 %0, $0" : : "r"(index));
	asm volatile("tlbp" : :);
	asm volatile("nop" : :);*/


    asm!("mfc0 {}, $0", out(reg) index);
    assert!(index >= 0);
    asm!("tlbr");
    asm!("mfc0 {}, $2", out(reg) entrylo);
    assert!((entrylo == (entrys[0])) as i32 + (entrylo == (entrys[1])) as i32 == 1);
	/*asm volatile("mfc0 %0, $0" : "=r"(index) :);
	assert(index >= 0);
	asm volatile("tlbr" : :);
	asm volatile("mfc0 %0, $2" : "=r"(entrylo) :);
	assert((entrylo == (entrys[0])) + (entrylo == (entrys[1])) == 1);
    */
	println!("tlb_refill_check() succeed!");
    }
}
