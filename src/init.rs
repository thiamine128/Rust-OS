use crate::env;
use crate::env::env_accept;
use crate::env::schedule;
use crate::memory;
// stable modules
use crate::println;
use crate::memory::*;
use crate::env::bare::*;
use crate::env_create_pri;
// unstable modules

/// rust entry
#[no_mangle]
pub extern "C" fn rust_main(_argc: u32, _argv: *const *const u8, _penv: *const *const u8, ram_low_size: usize) {
    println!("os init");
    //println!("{}, {}", exc_gen_entry as usize, tlb_miss_entry as usize);
    heap::init_heap();
    memory::init_memory(ram_low_size);
        
    env::env_init();
    //env_create_pri!(test_fs_strong_check, 1);
    //env_create_pri!(fs_serv, 1);
    
    schedule::schedule(0);
    // unsafe {env::test::load_icode_check()};
    // unsafe { frame::test::physical_memory_manage_strong_check(); }
    // unsafe { frame::test::page_strong_check(); }
    // unsafe { frame::test::tlb_refill_check(); }
    println!("Success!");
}

#[no_mangle]
pub extern "C" fn de() {
    println!("Except");
}