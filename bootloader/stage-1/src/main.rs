#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(const_trait_impl)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::arch::asm;

use common::config::STAGE_2_START;
use common::println_bios;
use common::static_items::{
    bios_info::BiosInfo,
    mem::MemInfo,
    static_variable::StaticVariable,
    vbe_display::{Font, VbeDisplayInfo},
};
use common::{gdt::*, println_vbe};

use vbe::init_graphical;

pub mod mem;
pub mod vbe;

static GDT_PROTECTED: Gdt = Gdt::protected_mode();

#[allow(unused)]
mod utils;
use utils::*;

use crate::mem::detect_memory;

/// Initalize all the variables we want to populate
// We force inline to save space on the stack as this could take up lots of space
#[inline(never)]
fn init_static_values() {
    // SAFETY: These should only be called once, we call them here
    unsafe {
        BiosInfo::init();
        Font::init();
        VbeDisplayInfo::init();
        MemInfo::init();
    }
}

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) {
    println_bios!("Starting stage 1");
    enable_a20();
    init_static_values();

    if !has_cpuid() {
        panic!("CPUID not present");
    }

    init_graphical();
    println_vbe!("Detecting memory");
    detect_memory();
    println_vbe!("DONE");
    loop {}
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
        asm!(
            // align the stack
            "and esp, 0xffffff00",
            // push arguments
            "push {info:e}",
            // push entry point address
            "push {entry_point:e}",
            info = in(reg) count as u32,
            entry_point = in(reg) STAGE_2_START as u32,
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
