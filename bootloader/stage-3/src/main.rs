#![no_std]
#![no_main]

use core::arch::asm;

use common::{print, println};

use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC: {info}");
    loop {
        unsafe { asm!("hlt") }
    }
}

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("mov ah, 0xf0", "mov al, 'L'", "mov [0xb8000], ax",);
    }
    loop {}
}
