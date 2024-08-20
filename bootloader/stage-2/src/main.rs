#![no_std]
#![no_main]

use common::*;
use core::arch::asm;

#[repr(packed)]
#[derive(Debug)]
struct MemoryMapEntry {
    base_address: u64,
    length: u64,
    region_type: u32,
    attributes: u32,
}

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(count: u16) -> ! {
    clear_screen();
    println!("Started protected mode");

    let mut mmap_reader: *const MemoryMapEntry = MEMORY_MAP_START as *const MemoryMapEntry;

    //for ii in 0..count {
    //    unsafe {
    //        let val = mmap_reader.add(ii as usize).read();
    //        println!("{:#x?}", val);
    //    }
    //}

    // Note that CPUID functionality is checked in stage-1
    if has_long_mode() {
        println!("We have long mode");
    } else {
        println!("No long mode!");
        hlt();
    }

    hlt();
}

unsafe fn print_memory_addresses(mut start: *const u8) {
    let size = 16;
    //let num_lines = 20;
    //for ii in 0..num_lines {
    //    let as_arr = core::slice::from_raw_parts(start, size);
    //    print!("0x{:04x}: ", start as u64);
    //    println!("{as_arr:02x?}");
    //    start = start.add(16);
    //}
    let as_arr = core::slice::from_raw_parts(start, size);
    print!("0x{:04x}: ", start as u64);
    println!("{as_arr:02x?}");
}

/// Sets up paging at the configured addresses
unsafe fn setup_paging() {
    // TODO: Read and understand paging better:
    // https://wiki.osdev.org/Setting_Up_Paging

    // Zero out addresses used for paging
    let starts = [PML4T_START, PDPT_START, PDT_START, PT_START];
    for start in starts {
        for ii in 0..0x400 {
            unsafe {
                // Sets the following:
                // kernel-only mode
                // write enabled
                // not present
                start.add(ii).write(2);
            }
        }
    }

    // Point pages to next values
    unsafe {
        (PML4T_START as *mut u32).write(PDPT_START as u32 + 3);
        (PDPT_START as *mut u32).write(PDT_START as u32 + 3);
        (PDT_START as *mut u32).write(PT_START as u32 + 3);
    }

    // TODO: check above and fix

    // Enable paging
    unsafe {
        asm!(
            "mov eax, cr4",
            "or eax, 1 << 5", // PAE-bit is the 6th bit
            "mov cr4, eax",
        );
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
