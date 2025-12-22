#![no_std]
#![no_main]

#[unsafe(link_section = ".start")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    let ptr = 0x1234 as *mut u16;
    unsafe {
        ptr.write(314);
    }
    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
