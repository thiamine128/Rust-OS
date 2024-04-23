use core::{fmt::{self, Write}, ptr::{read_volatile, write_volatile}};
use spin::Mutex;

use crate::memory::{self, mmu::KSEG1};
use crate::device::malta;

pub static STDOUT: Mutex<Stdout> = Mutex::new(Stdout{});

pub fn printchar(byte: u8) {
    if byte == b'\n' {
        printchar(b'\r');
    }
    unsafe {
        while read_volatile((KSEG1 + malta::MALTA_SERIAL_LSR) as *const u8) & malta::MALTA_SERIAL_THR_EMPTY == 0 {}
        write_volatile((KSEG1 + malta::MALTA_SERIAL_DATA) as *mut u8, byte);
    }
}

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            printchar(byte);
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::kern::print::_print(format_args!($fmt $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::kern::print::_print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}