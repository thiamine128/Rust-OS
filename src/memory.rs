pub mod mmu;
pub mod pmap;
pub mod heap;
pub mod page_table;
pub mod tlbex;

const KERNEL_HEAP_SIZE: usize = 0x80000;
