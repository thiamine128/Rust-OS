#![cfg_attr(target_arch = "mips", feature(asm_experimental_arch))]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

use core::arch::global_asm;
extern crate alloc;
extern crate mos_lib;

mod kern;
mod init;
mod memory;
mod device;
mod string;
mod list;
mod error;
mod bitops;
mod test;
global_asm!(include_str!("init/start.gen.S"));
global_asm!(include_str!("memory/tlb_asm.gen.S"));