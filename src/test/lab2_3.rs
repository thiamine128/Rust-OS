use core::{arch::{asm}, f32::consts::E, ptr::{addr_of, addr_of_mut, null_mut}};

use crate::{memory::{mmu::{va2pa, PAGE_SIZE}, pmap::{Pde, PMAP}, tlbex::_do_tlb_refill}, println};

extern "C" {
    fn do_tlb_refill_call(a: usize, b: usize, c: usize);
}

pub fn tlb_refill_check() {
	unsafe {
        let pp;
        let boot_pgdir;
        let pp0;
        let pp1;
        let pp2;
        let pp3;
        let pp4;
    {
	// should be able to allocate a page for directory
	let mut pmap = PMAP.lock();
    pp = pmap.page_alloc().unwrap();
	boot_pgdir = pmap.page2kva(pp) as *mut Pde;
	pmap.cur_pgdir = boot_pgdir;

	// should be able to allocate three pages
	 pp0 = pmap.page_alloc().unwrap();
     pp1 = pmap.page_alloc().unwrap();
     pp2 = pmap.page_alloc().unwrap();
     pp3 = pmap.page_alloc().unwrap();
     pp4 = pmap.page_alloc().unwrap();


	// temporarily steal the rest of the free pages
	// now this page_free list must be empty!!!!
	pmap.page_free_list.first = null_mut();

	// free pp0 and try again: pp0 should be used for page table
	pmap.page_free(pp0);
	// check if PTE != PP
	assert!(pmap.page_insert(boot_pgdir, 0, pp1, 0x0, 0).is_ok());
	// should be able to map pp2 at PAGE_SIZE because pp0 is already allocated for page table
	assert!(pmap.page_insert(boot_pgdir, 0, pp2, PAGE_SIZE, 0).is_ok());

	println!("tlb_refill_check() begin!");
    }
    let mut entrys: [usize; 2] = [0, 0];
    _do_tlb_refill(entrys.as_mut_ptr(), PAGE_SIZE, 0);
    
    {
    let mut pmap = PMAP.lock();
    let (walk_page, walk_pte) = pmap.page_lookup(boot_pgdir, PAGE_SIZE);
    assert!(!walk_page.is_null());
    assert!((entrys[0] == ((*walk_pte).0 >> 6)) as i32 + (entrys[1] == ((*walk_pte).0 >> 6)) as i32 == 1);
    assert!(pmap.page2pa(pp2) == va2pa(boot_pgdir, PAGE_SIZE));

	println!("test point 1 ok");

	pmap.page_free(pp4);
	pmap.page_free(pp3);
    
    let (res, walk_pte) = pmap.page_lookup(boot_pgdir, 0x00400000);
	assert!(res.is_null());
    }
	_do_tlb_refill(entrys.as_mut_ptr(), 0x00400000, 0);
    {
        let mut pmap = PMAP.lock();
    let (pp, walk_pte) = pmap.page_lookup(boot_pgdir, 0x00400000);
    assert!(!pp.is_null());
	assert!(va2pa(boot_pgdir, 0x00400000) == pmap.page2pa(pp3));
    }
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
