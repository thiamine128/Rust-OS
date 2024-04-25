use core::panic::PanicInfo;

use crate::println;

/// kernel panic implementation.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}