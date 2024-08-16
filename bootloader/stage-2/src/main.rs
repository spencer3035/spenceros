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

    println!("MEM:");
    let esp: u32;
    let ebp: u32;
    unsafe {
        asm!(
        "mov {esp}, esp",
        "mov {ebp}, ebp",
        esp = out(reg) esp,
        ebp = out(reg) ebp,
        );
    }

    println!("sp = 0x{esp:x}");
    println!("bp = 0x{ebp:x}");
    loop {}

    // TODO: Set sp/bp and verify that the memory is set as expected. We could also just print the
    // sp/bp values...
    // This doesn't work
    //unsafe {
    //    asm!(
    //    "mov eax, {stack_start}",
    //    "mov esp, eax",
    //    "mov ebp, 0",
    //    "push 7",
    //    stack_start = in(reg) STACK_END,
    //    )
    //}
    let start = STACK_END;
    let mut val = 0;
    //for ii in 0..30 {
    //    unsafe { start.add(ii).write(0xf) }
    //    val += 1;
    //}
    unsafe {
        //start.add(0).write(0x12);
        //start.add(1).write(0x34);
        //start.add(2).write(0xff);
        print_memory_addresses(STACK_END.sub(16));
    }

    // TODO:
    // Set up paging
    // Load/update gdt to have long mode
    // Enter Long Mode
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
