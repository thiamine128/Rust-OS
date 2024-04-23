use core::{mem::size_of, ptr::null_mut};
use core::ptr::addr_of_mut;
use crate::{list::linked_list::Head, list_first, list_insert_after, list_insert_head, memory::{mmu::{pa2kva, PAGE_SIZE}, pmap::{self, Page, PMAP}}, println};

pub fn lab2_1() {
	physical_memory_manage_check();
	physical_memory_manage_strong_check();
}

pub fn physical_memory_manage_strong_check() {
	unsafe {
	let mut pmap = PMAP.lock();
	let mut pp0 = pmap.page_alloc().unwrap();
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
	// should be no free memory
	assert!(pmap.page_alloc().is_err());

	let temp1 = pmap.page2kva(pp0) as *mut u32;
	// write 1000 to pp0
	*temp1 = 1000;
	// free pp0
	pmap.page_free(pp0);
	println!("The number in address temp is {}", *temp1);

	// alloc again
	pp0 = pmap.page_alloc().unwrap();
	assert!(!pp0.is_null());

	// pp0 should not change
	assert!(temp1 == pmap.page2kva(pp0) as *mut u32);
	// pp0 should be zero
	assert!(*temp1 == 0);

	pmap.page_free_list.first = fl;
	pmap.page_free(pp0);
	pmap.page_free(pp1);
	pmap.page_free(pp2);
	pmap.page_free(pp3);
	pmap.page_free(pp4);
	
	let test_pages = pmap.alloc(15 * size_of::<Page>(), PAGE_SIZE, 1) as *mut Page;
	let mut test_free = Head::<Page>::new();
	
	for i in (5..15).rev() {
		(*test_pages.add(i)).pp_ref = i as u16;
		list_insert_head!(test_free, test_pages.add(i).as_mut().unwrap(), pp_link);
	}
	for i in 0..5 {
		(*test_pages.add(i)).pp_ref = i as u16;
		list_insert_head!(test_free, test_pages.add(i).as_mut().unwrap(), pp_link);
	}
	let mut p = list_first!(test_free);
	let answer1: [u16; 15] = [4, 3, 2, 1, 0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
	assert!(!p.is_null());
	let mut j = 0;
	while !p.is_null() {
		assert!((*p).pp_ref == answer1[j]);
		j += 1;
		p = (*p).pp_link.next;
	}
	// insert_after test
	let answer2: [u16; 17] = [4, 40, 20, 3, 2, 1, 0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
	let q = pmap.alloc(size_of::<Page>(), PAGE_SIZE, 1) as *mut Page;
	(*q).pp_ref = 20;
	let qq = pmap.alloc(size_of::<Page>(), PAGE_SIZE, 1) as *mut Page;
	(*qq).pp_ref = 40;

	list_insert_after!(*test_pages.add(4), *q, pp_link);
	list_insert_after!(*test_pages.add(4), *qq, pp_link);
	p = list_first!(test_free);
	j = 0;
	while !p.is_null() {
		assert!((*p).pp_ref == answer2[j]);
		j += 1;
		p = (*p).pp_link.next;
	}
	println!("physical_memory_manage_check_strong() succeeded");
}
}

fn physical_memory_manage_check() {
	// should be able to allocate three pages
	let mut pmap = PMAP.lock();
    let mut pp0 = pmap.page_alloc().unwrap();
    let pp1 = pmap.page_alloc().unwrap();
    let pp2 = pmap.page_alloc().unwrap();

	assert!(!pp0.is_null());
	assert!(pp0 != pp1 && !pp1.is_null());
	assert!(!pp2.is_null() && pp2 != pp1 && pp2 != pp0);

	// temporarily steal the rest of the free pages
    let fl = pmap.page_free_list.first;
	pmap.page_free_list.first = null_mut();
	// should be no free memory
	assert!(pmap.page_alloc().is_err());

	let temp = pa2kva(pmap.page2pa(pp0)) as *mut u32;
	// write 1000 to pp0
    unsafe {
	    *temp = 1000;
    }
	// free pp0
	pmap.page_free(pp0);
    unsafe {
	    println!("The number in address temp is {}", *temp);
    }
	// alloc again
	pp0 = pmap.page_alloc().unwrap();
	assert!(!pp0.is_null());

	// pp0 should not change
	assert!(temp == pa2kva(pmap.page2pa(pp0)) as *mut u32);
	// pp0 should be zero
	unsafe {assert!(*temp == 0);}

	pmap.page_free_list.first = fl;
	pmap.page_free(pp0);
	pmap.page_free(pp1);
	pmap.page_free(pp2);
	
	let test_pages = pmap.alloc(10 * size_of::<Page>(), PAGE_SIZE, 1) as *mut Page;
	let mut test_free = Head::<Page>::new();
	
    for i in (0..10).rev() {
        unsafe {
            (*test_pages.add(i)).pp_ref = i as u16;
            list_insert_head!(test_free, test_pages.add(i).as_mut().unwrap(), pp_link);
        }
    }
	
	let mut p = list_first!(test_free);
	let answer1: [u16; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
	assert!(!p.is_null());
    let mut j = 0;
	while !p.is_null() {
        unsafe {
		assert!((*p).pp_ref == answer1[j]);
		j += 1;
		// printk("ptr: 0x%x v: %d\n",(p->pp_link).le_next,((p->pp_link).le_next)->pp_ref);
		p = (*p).pp_link.next;
        }
	}
	// insert_after test
	let answer2: [u16; 11] = [0, 1, 2, 3, 4, 20, 5, 6, 7, 8, 9];
	let q = pmap.alloc(size_of::<Page>(), PAGE_SIZE, 1) as *mut Page;
	unsafe {
        (*q).pp_ref = 20;
    }
	
	// printk("---%d\n",test_pages[4].pp_ref);
	unsafe {
		list_insert_after!(*test_pages.add(4), *q, pp_link);
	}
	// printk("---%d\n",LIST_NEXT(&test_pages[4],pp_link)->pp_ref);
	p = test_free.first;
	j = 0;
	// printk("into test\n");
	while !p.is_null() {
		unsafe {
		//      printk("%d %d\n",p->pp_ref,answer2[j]);
			assert!((*p).pp_ref == answer2[j]);
			j += 1;
			p = (*p).pp_link.next;
		}
	}

	println!("physical_memory_manage_check() succeeded");
}
