use crate::memory::page_table::page_strong_check;


pub fn lab2_2() {
    page_strong_check();
}
/*
pub fn page_strong_check() {
    unsafe {
	let mut pmap = PMAP.lock();
    let pp = pmap.page_alloc().unwrap();
	let boot_pgdir = pmap.page2kva(pp) as *mut Pde;

	// should be able to allocate three pages
    let pp0 = pmap.page_alloc().unwrap();
    let pp1 = pmap.page_alloc().unwrap();
    let pp2 = pmap.page_alloc().unwrap();
    let pp3 = pmap.page_alloc().unwrap();
    let pp4 = pmap.page_alloc().unwrap();

	assert!(!pp0.is_null());
	assert!(!pp1.is_null() && pp1 != pp0);
	assert!(!pp2.is_null() && pp2 != pp1 && pp2 != pp0);
	assert!(!pp3.is_null() && pp3 != pp2 && pp3 != pp1 && pp3 != pp0);
	assert!(!pp4.is_null() && pp4 != pp3 && pp4 != pp2 && pp4 != pp1 && pp4 != pp0);
    
	// temporarily steal the rest of the free pages
	let fl = pmap.page_free_list.first;
	// now this page_free list must be empty!!!!
	pmap.page_free_list.first = null_mut();

	// there is no free memory, so we can't allocate a page table
	assert!(pmap.page_insert(boot_pgdir, 0, pp1, 0x0, 0).is_err());

	// should be no free memory
	assert!(pmap.page_alloc().is_err());

	// free pp0 and try again: pp0 should be used for page table
	pmap.page_free(pp0);
	// check if PTE != PP
	assert!(pmap.page_insert(boot_pgdir, 0, pp1, 0x0, 0).is_ok());
	// should be able to map pp2 at PAGE_SIZE because pp0 is already allocated for page table
	assert!(pmap.page_insert(boot_pgdir, 0, pp2, PAGE_SIZE, 0).is_ok());
	assert!(pmap.page_insert(boot_pgdir, 0, pp3, 2 * PAGE_SIZE, 0).is_ok());
	assert!(pte_addr(Pte((*boot_pgdir).0)) == pmap.page2pa(pp0));

	println!("va2pa(boot_pgdir, 0x0) is {:x}", va2pa(boot_pgdir, 0x0));
	println!("page2pa(pp1) is {:x}\n", pmap.page2pa(pp1));

	assert!(va2pa(boot_pgdir, 0x0) == pmap.page2pa(pp1));
	assert!((*pp1).pp_ref == 1);
	assert!(va2pa(boot_pgdir, PAGE_SIZE) == pmap.page2pa(pp2));
	assert!((*pp2).pp_ref == 1);
	assert!(va2pa(boot_pgdir, 2 * PAGE_SIZE) == pmap.page2pa(pp3));
	assert!((*pp3).pp_ref == 1);

	println!("start page_insert");
	// should be able to map pp2 at PAGE_SIZE because it's already there
	assert!(pmap.page_insert(boot_pgdir, 0, pp2, PAGE_SIZE, 0).is_ok());
	assert!(va2pa(boot_pgdir, PAGE_SIZE) == pmap.page2pa(pp2));
	assert!((*pp2).pp_ref == 1);

	// should not be able to map at PDMAP because need free page for page table
	assert!(pmap.page_insert(boot_pgdir, 0, pp0, PDMAP, 0).is_err());
	// remove pp1 try again
	pmap.page_remove(boot_pgdir, 0, 0x0);
	assert!(va2pa(boot_pgdir, 0x0) == !0);
	assert!(pmap.page_insert(boot_pgdir, 0, pp0, PDMAP, 0).is_ok());

	// insert pp2 at 2*PAGE_SIZE (replacing pp2)
	assert!(pmap.page_insert(boot_pgdir, 0, pp2, 2 * PAGE_SIZE, 0).is_ok());

	// should have pp2 at both 0 and PAGE_SIZE, pp2 nowhere, ...
	assert!(va2pa(boot_pgdir, PAGE_SIZE) == pmap.page2pa(pp2));
	assert!(va2pa(boot_pgdir, 2 * PAGE_SIZE) == pmap.page2pa(pp2));
	// ... and ref counts should reflect this
	assert!((*pp2).pp_ref == 2);
	assert!((*pp3).pp_ref == 0);
	// try to insert PDMAP+PAGE_SIZE
	assert!(pmap.page_insert(boot_pgdir, 0, pp2, PDMAP + PAGE_SIZE, 0).is_ok());
	assert!((*pp2).pp_ref == 3);
	println!("end page_insert");

	// pp2 should be returned by page_alloc
    let pp = pmap.page_alloc().unwrap();
	assert!(pp == pp3);

	// unmapping pp2 at PAGE_SIZE should keep pp1 at 2*PAGE_SIZE
	pmap.page_remove(boot_pgdir, 0, PAGE_SIZE);
	assert!(va2pa(boot_pgdir, 2 * PAGE_SIZE) == pmap.page2pa(pp2));
	assert!((*pp2).pp_ref == 2);
	assert!((*pp3).pp_ref == 0);

	// unmapping pp2 at 2*PAGE_SIZE should keep pp2 at PDMAP+PAGE_SIZE
	pmap.page_remove(boot_pgdir, 0, 2 * PAGE_SIZE);
	assert!(va2pa(boot_pgdir, 0x0) == !0);
	assert!(va2pa(boot_pgdir, PAGE_SIZE) == !0);
	assert!(va2pa(boot_pgdir, 2 * PAGE_SIZE) == !0);
	assert!((*pp2).pp_ref == 1);
	assert!((*pp3).pp_ref == 0);

	// unmapping pp2 at PDMAP+PAGE_SIZE should free it
	pmap.page_remove(boot_pgdir, 0, PDMAP + PAGE_SIZE);
	assert!(va2pa(boot_pgdir, 0x0) == !0);
	assert!(va2pa(boot_pgdir, PAGE_SIZE) == !0);
	assert!(va2pa(boot_pgdir, 2 * PAGE_SIZE) == !0);
	assert!(va2pa(boot_pgdir, PDMAP + PAGE_SIZE) == !0);
	assert!((*pp2).pp_ref == 0);

	// so it should be returned by page_alloc
    let pp = pmap.page_alloc().unwrap();
	assert!(pp == pp2);

	// should be no free memory
	assert!(pmap.page_alloc().is_err());

	// forcibly take pp0 and pp1  back
	assert!(pte_addr(Pte((*boot_pgdir).0)) == pmap.page2pa(pp0));
	assert!(pte_addr(Pte((*boot_pgdir.add(1)).0)) == pmap.page2pa(pp1));
	(*boot_pgdir).0 = 0;
	(*boot_pgdir.add(1)).0 = 0;
	assert!((*pp0).pp_ref == 2);
	assert!((*pp1).pp_ref == 1);
	(*pp0).pp_ref = 0;
	(*pp1).pp_ref = 0;

	// give free list back
	pmap.page_free_list.first = fl;

	// free the pages we took
	pmap.page_free(pp0);
	pmap.page_free(pp1);
	pmap.page_free(pp2);
	pmap.page_free(pp3);
	pmap.page_free(pp4);
    let tmp = pmap.pa2page(kva2pa(boot_pgdir as usize));
	pmap.page_free(tmp);

	println!("page_check_strong() succeeded!");
    }
}


pub fn page_check() {
	unsafe {
	let mut pmap = PMAP.lock();
    let pp = pmap.page_alloc().unwrap();
    let boot_pgdir = pmap.page2kva(pp) as *mut Pde;
    let pp0 = pmap.page_alloc().unwrap();
    let pp1 = pmap.page_alloc().unwrap();
    let pp2 = pmap.page_alloc().unwrap();
    assert!(!pp0.is_null());
    assert!(!pp1.is_null() && pp0 != pp1);
    assert!(!pp2.is_null() && pp1 != pp2 && pp2 != pp0);

    let fl = pmap.page_free_list.first;
    pmap.page_free_list.first = null_mut();
    assert!(pmap.page_alloc().is_err());

    assert!(pmap.page_insert(boot_pgdir, 0, pp1, 0x0, 0).is_err());
    pmap.page_free(pp0);
    assert!(pmap.page_insert(boot_pgdir, 0, pp1, 0x0, 0).is_ok());
    assert!(pte_flags((*boot_pgdir).0) == (PTE_C_CACHEABLE | PTE_V));

    println!("va2pa(boot_pgdir, 0x0) is {:x}", va2pa(boot_pgdir, 0x0));
	println!("page2pa(pp1) is {:x}", pmap.page2pa(pp1));
    assert!(va2pa(boot_pgdir, 0x0) == pmap.page2pa(pp1));
    assert!((*pp1).pp_ref == 1);
    assert!(pmap.page_insert(boot_pgdir, 0, pp2, PAGE_SIZE, 0).is_ok());
    assert!(va2pa(boot_pgdir, PAGE_SIZE) == pmap.page2pa(pp2));
    assert!((*pp2).pp_ref == 1);
    assert!(pmap.page_alloc().is_err());
    println!("start page_insert");
    assert!(pmap.page_insert(boot_pgdir, 0, pp2, PAGE_SIZE, 0).is_ok());
    assert!(va2pa(boot_pgdir, PAGE_SIZE) == pmap.page2pa(pp2));
    assert!((*pp2).pp_ref == 1);

    assert!(pmap.page_alloc().is_err());
    assert!(pmap.page_insert(boot_pgdir, 0, pp0, PDMAP, 0).is_err());
    assert!(pmap.page_insert(boot_pgdir, 0, pp1, PAGE_SIZE, 0).is_ok());

    assert!(va2pa(boot_pgdir, 0x0) == pmap.page2pa(pp1));
    assert!(va2pa(boot_pgdir, PAGE_SIZE) == pmap.page2pa(pp1));

    assert!((*pp1).pp_ref == 2);
    println!("pp2->pp_ref {}", (*pp2).pp_ref);
    assert!((*pp2).pp_ref == 0);
    println!("end page_insert\n");

    let mut pp = pmap.page_alloc().unwrap();
    assert!(pp == pp2);
    //println!("unmap {}", ppn(va2pa(boot_pgdir, PAGE_SIZE)));
    
    pmap.page_remove(boot_pgdir, 0, 0x0);
    assert!(va2pa(boot_pgdir, 0x0) == !0);
    assert!(va2pa(boot_pgdir, PAGE_SIZE) == pmap.page2pa(pp1));
    assert!((*pp1).pp_ref == 1);
    assert!((*pp2).pp_ref == 0);

    pmap.page_remove(boot_pgdir, 0, PAGE_SIZE);
    assert!(va2pa(boot_pgdir, 0x0) == !0);
    assert!(va2pa(boot_pgdir, PAGE_SIZE) == !0);
    assert!((*pp1).pp_ref == 0);
    assert!((*pp2).pp_ref == 0);

    pp = pmap.page_alloc().unwrap();
    assert!(pp == pp1);
    assert!(pmap.page_alloc().is_err());

    assert!(pte_addr(Pte((*boot_pgdir).0)) == pmap.page2pa(pp0));
    (*boot_pgdir).0 = 0;
    assert!((*pp0).pp_ref == 1);
    (*pp0).pp_ref = 0;

    pmap.page_free_list.first = fl;

    pmap.page_free(pp0);
    pmap.page_free(pp1);
    pmap.page_free(pp2);
    let tmp = pmap.pa2page(kva2pa(boot_pgdir as usize));
    pmap.page_free(tmp);
    println!("page_check() succeeded!");
    }
}
*/