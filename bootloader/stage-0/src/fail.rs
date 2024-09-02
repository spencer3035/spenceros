use common::io::bios::{print_chars, println_chars};
use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    fail(b"panic");
}

/// Prints 'Fail: [char]' and halts
pub(crate) fn fail(code: &[u8]) -> ! {
    print_chars(b"Fail: ");
    println_chars(code);
    hlt()
}

/// Halts the CPU
pub(crate) fn hlt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
