#![no_std]
#![no_main]

global_asm!(include_str!("boot.s"));

use core::arch::asm;
use core::arch::global_asm;

use common::*;

extern "C" {
    /// The address of this number is set in the link.ld file to be the first byte of the next
    /// section. We can use the address of this to transmute it to a function pointer and call it.
    static _second_stage_start: u8;
}

#[no_mangle]
pub extern "C" fn main(drive_number: u16) {
    unsafe {
        check_int13();
    }
    load_sectors(drive_number);

    // Transmute the pointer to the beginning of the next stage to a function and call it.
    let next_stage: extern "C" fn(disk_number: u16) =
        unsafe { core::mem::transmute(&_second_stage_start as *const u8 as *const ()) };
    next_stage(drive_number);
    hlt();
}

/// Check that inturrupt 13 is avaliable
#[inline(always)]
unsafe fn check_int13() {
    let ax: u16;
    asm!(
      "mov ah, 0x41",
      "mov bx, 0x55aa",
      // dl contains drive number
      "int 0x13",
      // Put carry flag into ax
      "mov {0:x}, 0",
      "adc {0:x}, 0",
       out(reg) ax
    );

    if ax != 0 {
        fail(b"int13");
    }
}

fn load_sectors(drive_number: u16) {
    let mut num_sectors: u8 = SECTORS_TO_READ as u8;
    let requested_sectors = num_sectors;
    let to_address: u16 = 0x7e00;
    let carry: u16;
    let exit_status: u8;

    // TODO: Replace with more modern LBA loading instead of CHS
    unsafe {
        asm!(
            "mov ah, 2", // 2 for reading disk to memory
            "mov ch, 0", // Cylander number
            "mov cl, 2", // Sector number
            "mov dh, 0", // Head number
            "int 0x13", // Perform read inturrupt
            "mov cx, 0",
            "adc cx, 0",
            in("bx") to_address,
            in("dl") drive_number as u8,
            inout("al") num_sectors,
            out("ah") exit_status,
            out("cx") carry
        )
    }
    let read_sectors = num_sectors;
    let disk_read_ok = exit_status == 0 && carry == 0;

    if !disk_read_ok {
        fail(b"disk read");
    }

    if requested_sectors != read_sectors {
        fail(b"num sector mismatch");
    }
}
