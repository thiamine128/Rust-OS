// stable modules
use crate::{memory, println, test};
use crate::memory::heap;
// unstable modules

/// rust entry
#[no_mangle]
pub extern "C" fn rust_main(_argc: u32, _argv: *const *const u8, _penv: *const *const u8, ram_low_size: usize) {
    println!("os init");
    heap::init_heap();
    memory::init_memory(ram_low_size);

    //test::lab2_1::lab2_1();
    //test::lab2_2::lab2_2();
    test::lab2_3::lab2_3();
    panic!("Success!");
}