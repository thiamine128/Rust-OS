use core::{fmt::{self, Write}, ptr::{read_volatile, write_volatile}};

use crate::memory::mmu::KSEG1;
use crate::device::malta;


const STATE_ADDR: *const u8 = (KSEG1 + malta::MALTA_SERIAL_LSR) as *const u8;
const DATA_ADDR: *mut u8 = (KSEG1 + malta::MALTA_SERIAL_DATA) as *mut u8;

pub fn printcharc(byte: u8) {
    if byte == b'\n' {
        printcharc(b'\r');
    }
    
    while unsafe { read_volatile(STATE_ADDR) } & malta::MALTA_SERIAL_THR_EMPTY == 0 {}
    unsafe {write_volatile(DATA_ADDR, byte)};
}

pub fn scancharc() -> u8 {
    if unsafe {
        read_volatile(STATE_ADDR) & malta::MALTA_SERIAL_DATA_READY
    } != 0 {
        return unsafe { read_volatile(DATA_ADDR) };
    }
    return 0;
}
// Stdout to serial device.
pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            printcharc(byte);
        }
        Ok(())
    }
}

/// kernel print implementation.
pub fn _print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// print to serial device.
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print::_print(format_args!($fmt $(, $($arg)+)?));
    };
}

/// println to serial device,
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print::_print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}
