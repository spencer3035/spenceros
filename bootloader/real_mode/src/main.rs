#![no_std]
#![no_main]

use common::*;

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) -> ! {
    println(b"We made it bois");
    hlt();
}
