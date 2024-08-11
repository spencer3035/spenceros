#![no_std]
#![no_main]

global_asm!(include_str!("boot.s"));

use core::arch::asm;
use core::arch::global_asm;
use core::panic::PanicInfo;

pub mod debug;
use debug::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    fail(b'P');
}

#[no_mangle]
pub extern "C" fn main(drive_number: u8) {
    //println(b"Reading");
    load_sector(drive_number);
    let char_out_of_range: u16;
    unsafe {
        asm!(
            "mov ax, [0x7e01]",
            "mov {0:x}, ax",
            out(reg) char_out_of_range
        );
    }

    print_char(&(char_out_of_range as u8));
    //println(b"Done");
    //print_dec(123);
    //println(b"\r\n");
    hlt();
}

fn load_sector(drive_number: u8) {
    let mut num_sectors: u8 = 1;
    let requested_sectors = num_sectors;
    let mut to_address: u16 = 0x7e00;
    let exit_status: u8;

    // TODO: Something is broken here
    unsafe {
        asm!(
            "mov bx, {0:x}", // Address to load into
            "mov ah, 2", // 2 for reading disk to memory
            "mov al, {2}", // Number of sectors to read
            "mov ch, 0", // Cylander number
            "mov cl, 1", // Sector number
            "mov dh, 0", // Head number
            "mov dl, {1}", // Drive number
            "int 0x13", // Perform read inturrupt
            "mov dx, 0", // Put Carry flag into ax
            "adc dx, 0",
            "mov {0:x}, dx",
            "mov {3}, ah",
            inout(reg) to_address,
            in(reg_byte) drive_number,
            inout(reg_byte) num_sectors,
            out(reg_byte) exit_status,
        )
    }
    // TODO: This is wrong, change it to ==
    //let disk_read_ok = to_address == 0;
    let disk_read_ok = exit_status == 0;

    if !disk_read_ok {
        print_dec(exit_status.into());
        fail(b'd');
    }

    let read_sectors = num_sectors;
    if requested_sectors != read_sectors {
        fail(b'r');
    }
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
    hlt()
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
