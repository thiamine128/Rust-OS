use crate::memory::{frame::init_frame_allocator, mmu::VirtAddr};

use self::mmu::PAGE_SIZE;

/// mmu address
pub mod mmu;
/// kernel heap
pub mod heap;
/// frame management
pub mod frame;
/// page table
pub mod page_table;
/// tlb utils
pub mod tlb;
/// shared memory
pub mod shm;

/// initialize frame allocator
pub fn init_memory(memsize: usize) {
    extern "C" {
        fn end();
    }
    let nframes = memsize / PAGE_SIZE;
    let freemem = VirtAddr::new(end as usize);
    
    init_frame_allocator(freemem, nframes);
}