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
pub mod tlb;



pub fn init_memory(memsize: usize) {
    extern "C" {
        fn end();
    }
    let nframes = memsize / PAGE_SIZE;
    let freemem = VirtAddr::new(end as usize);
    
    init_frame_allocator(freemem, nframes);
}

/// memset in c
fn memset(dst: *mut u8, c: i32, n: usize){
    let mut dst = dst as usize;
    let max = dst + n;
    let byte= (c & 0xff) as u8;
    let word = (byte as u32)  | (byte as u32) << 8 | (byte as u32) << 16 | (byte as u32) << 24;
    while (dst & 3) != 0 && dst < max {
        unsafe { *(dst as *mut u8) = byte; }
        dst += 1;
    }

    while dst + 4 <= max {
        unsafe { *(dst as *mut u32) = word; }
        dst += 4;
    }

    while dst < max {
        unsafe { *(dst as *mut u8) = byte; }
        dst += 1;
    }
}