#![no_std]
#![no_main]

use core::arch::asm;

macro_rules! println {
    ($($args:tt)*) => {
        common::println_screen!($($args)*);
    };
}

macro_rules! print {
    ($($args:tt)*) => {
        common::print_screen!($($args)*);
    };
}

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
