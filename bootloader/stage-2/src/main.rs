#![no_std]
#![no_main]

mod common;
mod io;

use common::hlt;
use io::{clear_screen, print, println};

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) -> ! {
    clear_screen();

    for ii in 0usize..25 {
        println!("bruh, we have finally done it {ii}");
    }
    print!("Some extra stuff");
    hlt();
}
