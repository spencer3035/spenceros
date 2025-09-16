#![no_std]
#![no_main]

global_asm!(include_str!("boot.s"));

use core::arch::asm;
use core::arch::global_asm;

pub mod fail;

use common::config::STACK;
use common::config::STACK_END;
use common::config::STAGE_1_START;
use fail::fail;

// TODO: This currently uses up 0x36 values on the stack and they will never get cleaned up
// because the next stage will never return. It seems like a bad idea to mess with `sp` and
// `bp` here though. It might not actually be a problem long term?
#[no_mangle]
pub extern "C" fn _rust_entry(drive_number: u16) {
    let next_stage = main_inner(drive_number);
    next_stage(drive_number);
    fail(b"stage 1")
}

// We force this to not inline so that rust can clean up the stack for us as much as possible
#[inline(never)]
fn main_inner(drive_number: u16) -> extern "C" fn(u16) {
    let bp: u16;
    let sp: u16;
    unsafe {
        asm!(
        "mov {0:x}, bp",
        "mov {1:x}, sp",
        out(reg) bp,
        out(reg) sp
        );
    }

    if bp != STACK_END as u16 || sp < STACK as u16 || sp > STACK_END as u16 {
        fail(b"stack out of range, check bp in boot.s");
    }

    check_int13(drive_number);
    load_sectors(drive_number);

    // Transmute the pointer to the beginning of the next stage to a function and call it.
    let next_stage: extern "C" fn(disk_number: u16) =
        unsafe { core::mem::transmute(STAGE_1_START as *const ()) };
    next_stage
}

/// Check that interrupt 13 is available
#[inline(always)]
fn check_int13(drive_num: u16) {
    let ax: u16;
    unsafe {
        asm!(
          "mov ah, 0x41",
          "mov bx, 0x55aa",
          // dl contains drive number
          "int 0x13",
          // Put carry flag into ax
          "mov ax, 0",
          "jnc 2f",
          "mov ax, 12",
          "2:",
           out("ax") ax,
           in("dx") drive_num,
        );

        if ax != 0 {
            fail(b"int13");
        }
    }
}

fn load_sectors(drive_number: u16) {
    let mut num_sectors: u8 = common::config::SECTORS_TO_READ as u8;
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
