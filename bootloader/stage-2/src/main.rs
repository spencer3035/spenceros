#![no_std]
#![no_main]

use common::*;
use core::arch::asm;

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) -> ! {
    clear_screen();
    println!("Started protected mode");

    // Note that CPUID functionality is checked in stage-1
    if has_long_mode() {
        println!("We have long mode");
    } else {
        println!("No long mode!");
        hlt();
    }

    // TODO:
    // Set up paging
    // Load/update gdt to have long mode
    // Enter Long Mode
    hlt();
}

fn setup_paging() {
    // Zero out addresses used for paging
    let starts = [PML4T_START, PDPT_START, PDT_START, PT_START];
    for start in starts {
        for ii in 0..0x1000 {
            unsafe {
                start.add(ii).write(0);
            }
        }
    }
}

// Uses CPUID to check for long mode
fn has_long_mode() -> bool {
    let eax: u32;
    unsafe {
        asm!(
            "mov eax, 0x80000000",
            "cpuid",
            out("eax") eax
        );
    }

    if eax < 0x80000001 {
        println!("No long mode feature");
        return false;
    }

    let edx: u32;
    unsafe {
        asm!(
            "mov eax, 0x80000001",
            "cpuid",
            out("edx") edx
        );
    }

    edx & 1 << 29 != 0
}
