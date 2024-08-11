#![no_std]
#![no_main]

global_asm!(include_str!("boot.s"));

use core::arch::asm;
use core::arch::global_asm;

use common::*;
const SECTORS_TO_READ: u8 = config::REAL_MODE_SECTIONS as u8;

extern "C" {
    /// The address of this number is set in the link.ld file to be the first byte of the next
    /// section. We can use the address of this to transmute it to a function pointer and call it.
    static _second_stage_start: u8;
}

#[no_mangle]
pub extern "C" fn main(drive_number: u16) {
    println(b"Starting Bootloader");
    load_sectors(drive_number);

    // Transmute the pointer to the beginning of the next stage to a function and call it.
    let next_stage: extern "C" fn(disk_number: u16) =
        unsafe { core::mem::transmute(&_second_stage_start as *const u8 as *const ()) };
    next_stage(drive_number);
    hlt();
}

fn load_sectors(drive_number: u16) {
    //print_char(&b'j');
    //print_dec(drive_number);
    let mut num_sectors: u8 = SECTORS_TO_READ;
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
        //print_dec(exit_status.into());
        fail(b"disk read");
    }

    if requested_sectors != read_sectors {
        //print_dec(requested_sectors.into());
        //print_char(b' ');
        //print_dec(read_sectors.into());
        fail(b"num sector mismatch");
    }
}
