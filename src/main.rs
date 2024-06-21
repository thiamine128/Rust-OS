#![cfg_attr(target_arch = "mips", feature(asm_experimental_arch))]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(step_trait)]
#![feature(concat_idents)]
#![feature(lazy_cell)]
#![allow(dead_code)]
#![no_std]
#![no_main]

use core::arch::global_asm;

extern crate alloc;

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
/// handle exception
pub mod exception;
/// os error enum
pub mod err;
/// rust utils
pub mod util;
/// user process
pub mod env;
/// for sync
pub mod sync;

global_asm!(include_str!("init/start.gen.S"));
global_asm!(include_str!("memory/tlb_asm.gen.S"));
global_asm!(include_str!("env/env_asm.gen.S"));
global_asm!(include_str!("exception/genex.gen.S"));
global_asm!(include_str!("exception/entry.gen.S"));