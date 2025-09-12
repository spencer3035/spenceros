#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(const_trait_impl)]
#![deny(unsafe_op_in_unsafe_fn)]
// TODO: Remove
#![allow(unused)]

use core::arch::asm;

use common::config::MEMORY_MAP_START;
use common::println_bios;
use common::static_string::StaticString;
use common::{gdt::*, println_vbe};
use common::{print_bios, print_vbe};

use common::io::bios::{print_char, print_chars, print_hex, print_hex32};
use vbe::init_graphical;

pub mod mem;
pub mod vbe;

static GDT_PROTECTED: Gdt = Gdt::protected_mode();

mod utils;
use utils::*;

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) {
    enable_a20();

    if !has_cpuid() {
        panic!("CPUID not present");
    }

    println_bios!("About to change graphical modes,");
    prompt_continue();
    init_graphical();
    println_vbe!("Finished graphical");
    println_vbe!("STACK USED: 0x{:X}", get_stack_used());
    let mut s = StaticString::new();
    loop {
        let c = utils::next_keypress();

        if c == '\r' {
            println_vbe!("{s}");
            s.clear();
        } else {
            s.push(c);
        }
    }
    loop {}
    // let count = unsafe { mem::detect_memory() };
    // panic!("Not ready for next stage");
    // loop {}
    // unsafe {
    //     load_gdt();
    //     next_stage(count);
    // }
}

unsafe fn next_stage(count: u16) {
    // Perform long jump
    unsafe {
        let entry_point = 0x7c00 + 0x600;
        asm!(
            // align the stack
            "and esp, 0xffffff00",
            // push arguments
            "push {info:e}",
            // push entry point address
            "push {entry_point:e}",
            info = in(reg) count as u32,
            entry_point = in(reg) entry_point as u32,
        );
        // Perform a "long jump" to one line down.
        asm!(
            // TODO: How do we know this is sector 0x8?
            // Note that 2f means jump (f)orward to the next local label "2:"
            "ljmp $0x08, $2f",
            // Relative label that we jump to
            "2:",
            options(att_syntax)
        );
        asm!(
            ".code32",

            // reload segment registers
            "mov {0}, 0x10",
            "mov ds, {0}",
            "mov es, {0}",
            "mov ss, {0}",

            // jump to stage-2
            "pop {1}",
            "call {1}",

            // enter endless loop in case stage-2 returns
            "2:",
            "jmp 2b",
            out(reg) _,
            out(reg) _,
        );
    }
}

/// Disables interrupts and loads GDT
#[inline(always)]
unsafe fn load_gdt() {
    // Setup protected mode
    GDT_PROTECTED.load();
    unsafe {
        asm!(
            "cli",          // Disable inturrupts
            "mov eax, cr0", // Set protection enable bit
            "or eax, 1",
            "mov cr0, eax",
        );
    }
}
