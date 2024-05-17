#ifndef _PMAP_H_
#define _PMAP_H_

#include <mmu.h>
#include <types.h>

struct Page {
	// Ref is the count of pointers (usually in page table entries)
	// to this page.  This only holds for pages allocated using
	// page_alloc.  Pages allocated at boot time using pmap.c's "alloc"
	// do not have valid reference count fields.

	u_short pp_ref;
};
#endif