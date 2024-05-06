#![cfg_attr(target_arch = "mips", feature(asm_experimental_arch))]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(step_trait)]
#![no_std]
#![no_main]

use core::arch::global_asm;

extern crate alloc;
extern crate buddy_system_allocator;
extern crate spin;
extern crate lazy_static;

/// kernel print.
pub mod print;
/// kernel panic.
pub mod panic;
/// kernel init.
pub mod init;
/// memory management.
pub mod memory;
/// serial device.
pub mod device;
/// bit util
pub mod bitops;
/// process
pub mod process;
/// handle exception
pub mod exception;
/// for test
pub mod test;

global_asm!(include_str!("init/start.gen.S"));
global_asm!(include_str!("memory/tlb_asm.gen.S"));