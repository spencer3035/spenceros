#![no_std]
#![no_main]

use core::arch::asm;
use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("boot.s"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    fail(b'P');
}

fn print_char(c: &u8) {
    let ax = *c as u16 | 0x0e00;
    unsafe {
        asm!(
            "int 0x10",
            in("ax") ax,
        );
    }
}

fn print_dec(mut num: u16) {
    let mut num_digits = 0;

    loop {
        let digit: u16 = (num % 10).into();
        unsafe {
            asm!("push {0:x}", in(reg) digit);
        }
        num_digits += 1;
        num /= 10;
        if num == 0 {
            break;
        }
    }

    while num_digits >= 1 {
        let digit: i16;
        unsafe {
            asm!("pop {0:x}", out(reg) digit);
        }
        let value = digit as u8 + b'0';
        print_char(&value);
        num_digits -= 1;
    }
}

fn println(chars: &[u8]) {
    print(chars);
    print(b"\r\n");
}

fn print(chars: &[u8]) {
    for val in chars.iter() {
        print_char(val);
    }
}

#[no_mangle]
pub extern "C" fn main(drive_number: u8) {
    println(b"Main.");
    print_dec(123);
    println(b"\r\n");
    hlt();
}

/// Prints '![char]' where [char] should be the top element on the stack when this is called
///
/// Should not be called with jump commands from assembly. Will not work unless called
#[cold]
#[inline(never)]
#[no_mangle]
pub extern "C" fn fail(code: u8) -> ! {
    print(b"Fail: ");
    print_char(&code);
    print(b"\r\n");
    loop {
        hlt()
    }
}

/// Halts CPU
fn hlt() -> ! {
    println(b"Halt.");
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
