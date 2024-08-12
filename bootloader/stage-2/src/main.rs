#![no_std]
#![no_main]

use core::arch::asm;

use common::*;

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) -> ! {
    unsafe {
        // Print char
        asm!("mov ah, 0x0f", "mov al, 'A'", "mov [0xb8000], ax")
    }
    loop {}
}
