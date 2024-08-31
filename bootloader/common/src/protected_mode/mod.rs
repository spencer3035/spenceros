use core::arch::asm;

pub fn hlt() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}

pub mod io;
