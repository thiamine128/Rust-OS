use core::{fmt::{self, Write}, ptr::{read_volatile, write_volatile}};
use spin::Mutex;

use crate::memory::mmu::KSEG1;
use crate::device::malta;

static STDOUT: Mutex<Stdout> = Mutex::new(Stdout{});

fn printcharc(byte: u8) {
    if byte == b'\n' {
        printcharc(b'\r');
    }
    unsafe {
        while read_volatile((KSEG1 + malta::MALTA_SERIAL_LSR) as *const u8) & malta::MALTA_SERIAL_THR_EMPTY == 0 {}
        write_volatile((KSEG1 + malta::MALTA_SERIAL_DATA) as *mut u8, byte);
    }
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
    STDOUT.lock().write_fmt(args).unwrap();
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