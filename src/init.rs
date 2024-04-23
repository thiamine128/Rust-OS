use crate::{memory::{heap, pmap::{self, PMAP}}, println, test};

#[no_mangle]
pub extern "C" fn rust_main(_argc: u32, _argv: *const *const u8, _penv: *const *const u8, ram_low_size: usize) {
    println!("os init");
    
    {
        let mut pmap = pmap::PMAP.lock();
        pmap.mips_detect_memory(ram_low_size);
        pmap.mips_vm_init();
        pmap.page_init();
        heap::init_heap();
    }

    panic!("Success!");
}