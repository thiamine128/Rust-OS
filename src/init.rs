use crate::memory;
// stable modules
use crate::println;
use crate::memory::*;
// unstable modules

/// rust entry
#[no_mangle]
pub extern "C" fn rust_main(_argc: u32, _argv: *const *const u8, _penv: *const *const u8, ram_low_size: usize) {
    println!("os init");
    heap::init_heap();
    memory::init_memory(ram_low_size);

    //unsafe { frame::test::physical_memory_manage_strong_check(); }
    //unsafe { frame::test::page_strong_check(); }
    //unsafe { frame::test::tlb_refill_check(); }
    panic!("Success!");
}