use crate::env;
use crate::env::schedule;
use crate::env::sem;
use crate::env_create_pri;
use crate::memory;
use crate::println;
use crate::memory::*;
use crate::env::bare::*;

pub struct Init;

impl Init {
    pub fn init(&mut self, ram_low_size: usize) {
        println!("mos init");
        heap::init_heap();
        memory::init_memory(ram_low_size);

        env::env_init();
        shm::init();
        sem::init();
        
        env_create_pri!(USER_ICODE, 1);
        env_create_pri!(FS_SERV, 1);
        
        schedule::schedule(0);
    }
}

/// rust entry
#[no_mangle]
pub extern "C" fn rust_main(_argc: u32, _argv: *const *const u8, _penv: *const *const u8, ram_low_size: usize) {
    Init.init(ram_low_size);
}