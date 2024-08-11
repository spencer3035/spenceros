#![no_std]
#![no_main]

use core::arch::asm;
use core::arch::global_asm;
use core::panic::PanicInfo;

pub mod debug;
use debug::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    fail(b"panic");
}

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) -> ! {
    println(b"We made it bois");
    hlt();
}

// TODO: Put in common crate
/// Prints '![char]' where [char] should be the top element on the stack when this is called
///
/// Should not be called with jump commands from assembly. Will not work unless called
fn fail(code: &[u8]) -> ! {
    print(b"Fail: ");
    println(code);
    hlt()
}

// TODO: Put in common crate
/// Halts CPU
fn hlt() -> ! {
    println(b"Halt.");
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
