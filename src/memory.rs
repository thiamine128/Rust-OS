use core::mem::size_of;
use crate::memory::frame::init_frame_allocator;
use crate::memory::{frame::PhysFrame, mmu::VirtAddr};

use self::mmu::PAGE_SIZE;

/// mmu address
pub mod mmu;
/// kernel heap
pub mod heap;
/// frame management
pub mod frame;
/// page table
pub mod page_table;
/// tlb
pub mod tlb;

#[derive(Debug)]
pub enum Error {
    NotMapped,
    NoMem
}

pub fn init_memory(memsize: usize) {
    extern "C" {
        fn end();
    }
    let nframes = memsize / PAGE_SIZE;
    let frames_addr = end as *mut PhysFrame; 
    let mut freemem = VirtAddr::new(end as usize);
    freemem += size_of::<PhysFrame>() * nframes;
    memset(frames_addr as *mut u8, 0, size_of::<PhysFrame>() * nframes);

    init_frame_allocator(frames_addr, freemem, nframes);
}

/// memset in c
fn memset(mut dst: *mut u8, c: i32, n: usize) -> *mut u8{
    unsafe {
        let dstaddr = dst;
        let max = dst.add(n);
        let byte= (c & 0xff) as u8;
        let word = (byte as u32)  | (byte as u32) << 8 | (byte as u32) << 16 | (byte as u32) << 24;
        while ((dst as u32) & 3) != 0 && dst < max {
            *dst = byte;
            dst = dst.offset(1);
        }

        while dst.offset(4) <= max {
            *(dst as *mut u32) = word;
            dst = dst.offset(4);
        }

        while dst < max {
            *dst = byte;
        }

        dstaddr
    }
}