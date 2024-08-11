#![no_std]
#![no_main]

global_asm!(include_str!("boot.s"));

use core::arch::asm;
use core::arch::global_asm;
use core::panic::PanicInfo;

pub mod debug;
use debug::*;

const PANIC: u8 = b'P';
const DISK_READ_ERROR: u8 = b'D';

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    fail_rs(PANIC);
}

#[no_mangle]
pub extern "C" fn main(drive_number: u16) {
    //print_dec(drive_number);
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

    print_char(char_out_of_range as u8);
    //println(b"Done");
    //print_dec(123);
    //println(b"\r\n");
    hlt();
}

fn load_sector(drive_number: u16) {
    //print_char(&b'j');
    //print_dec(drive_number);
    let mut num_sectors: u8 = 1;
    let requested_sectors = num_sectors;
    let to_address: u16 = 0x7e00;
    let exit_status: u8;

    // TODO: Something is broken here
    unsafe {
        asm!(
            "mov ah, 2", // 2 for reading disk to memory
            "mov ch, 0", // Cylander number
            "mov cl, 2", // Sector number
            "mov dh, 0", // Head number
            "push 'D'", // Push error code
            "int 0x13", // Perform read inturrupt
            "jc fail", // Check success
            "pop bx", // Push error code
            in("bx") to_address,
            in("dl") drive_number as u8,
            inout("al") num_sectors,
            out("ah") exit_status,
        )
    }
    let read_sectors = num_sectors;

    // TODO: This is wrong, change it to ==
    //let disk_read_ok = to_address == 0;
    let disk_read_ok = exit_status == 0;

    if !disk_read_ok {
        print_dec(exit_status.into());
        fail_rs(b'S');
    }

    if requested_sectors != read_sectors {
        print_dec(requested_sectors.into());
        print_char(b' ');
        print_dec(read_sectors.into());
        fail_rs(b'R');
    }
}

/// Prints '![char]' where [char] should be the top element on the stack when this is called
///
/// Should not be called with jump commands from assembly. Will not work unless called
#[cold]
#[inline(never)]
#[no_mangle]
pub extern "C" fn fail_rs(code: u8) -> ! {
    print(b"Fail: ");
    print_char(code);
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
