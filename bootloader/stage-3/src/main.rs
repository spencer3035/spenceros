#![no_std]
#![no_main]

use core::arch::asm;

use common::println_vbe;

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // println!("PANIC: {info}");
    loop {
        unsafe { asm!("hlt") }
    }
}

#[unsafe(link_section = ".start")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println_vbe!("Started long");
    loop {}
}
