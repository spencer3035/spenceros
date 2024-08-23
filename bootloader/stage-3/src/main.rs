#![no_std]
#![no_main]

use core::arch::asm;

use common::*;

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("mov ah, 0xf0", "mov al, 'L'", "mov [0xb8000], ax",);
    }
    loop {}
}
