use crate::{print, println};
use core::arch::asm;
use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    hlt();
}

pub fn hlt() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}
