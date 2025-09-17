#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(const_trait_impl)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::arch::asm;
use core::ptr::addr_of;

use common::config::STAGE_2_START;
use common::println_bios;
use common::println_vbe;
use common::static_items::{
    bios_info::BiosInfo,
    mem::MemInfo,
    static_variable::StaticVariable,
    vbe_display::{Font, VbeDisplayInfo},
};

use vbe::init_graphical;

pub mod gdt;
pub mod mem;
pub mod vbe;

#[allow(unused)]
mod utils;
use utils::*;

use crate::mem::detect_memory;

/// Initalize all the variables we want to populate
fn init_static_values() {
    // SAFETY: These should only be called once, we call them here
    unsafe {
        BiosInfo::init();
        Font::init();
        VbeDisplayInfo::init();
        MemInfo::init();
    }
}

/// Main function, we force inline so that rust will clean up the stack
#[inline(never)]
fn main(_disk_number: u16) {
    println_bios!("Stack used : 0x{:X}", get_stack_used());
    println_bios!("Starting stage 1");
    enable_a20();
    init_static_values();

    if !has_cpuid() {
        panic!("CPUID not present");
    }

    prompt_continue();
    init_graphical();

    println_vbe!("Detecting memory");
    // Safety: This is the only mutable reference.
    let memory_info = unsafe { MemInfo::get_mut() };
    detect_memory(memory_info);
}

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) {
    utils::print_unsafe_fn_location(next_stage);
    println_bios!("Stack used : 0x{:X}", get_stack_used());
    main(_disk_number);
    println_vbe!("About to enter next stage");
    unsafe {
        next_stage();
    }
}

use common::gdt::{Gdt, GdtPointer};

static mut GDT_POINTER: GdtPointer = GdtPointer::null();

#[inline(never)]
unsafe fn next_stage() {
    println_vbe!("Loading GDT");
    // Setup protected mode
    let gdt_addr = unsafe {
        Gdt::init();
        let gdt = Gdt::get_mut();
        *gdt = Gdt::protected_mode();
        gdt as *const Gdt
    };

    unsafe {
        GDT_POINTER = GdtPointer::new(
            gdt_addr,
            (Gdt::NUM_ENTRIES * size_of::<u64> as u16 - 1) as u16,
        );
    };
    unsafe {
        asm!(
            "cli",          // Disable inturrupts
            "lgdt [{}]",
             in(reg) addr_of!(GDT_POINTER),
             options(readonly, nostack, preserves_flags)
        );

        let mut cr0: u32;
        asm!(
            "mov {:e}, cr0", // Set protection enable bit
            out(reg) cr0,
            options(nomem,nostack,preserves_flags),
        );

        let cr0_protected = cr0 | 1;

        asm!(
            "mov cr0, {:e}",
            in(reg) cr0_protected,
            options(nostack,preserves_flags)
        );
    }
    // Perform long jump
    unsafe {
        asm!(
            // align the stack
            // "mov esp, ebp",
            "and esp, 0xffffff00",
            // push entry point address
            "push {entry_point:e}",
            entry_point = in(reg) STAGE_2_START as u32,
        );
        // println_vbe!("DONE");
        // TODO: Something seems to be broken with this
        // Perform a "long jump" to one line down.
        asm!(
            // reload segment registers
            // TODO: How do we know this is sector 0x8?
            // Note that 2f means jump (f)orward to the next local label "2:"
            "ljmp $0x08, $2f",
            // Relative label that we jump to
            "2:",
            options(att_syntax, nostack)
        );
        asm!(
            ".code32",

            "mov {0}, 0x10", // The 0x02 is the data segment
            "mov ds, {0}",
            "mov es, {0}",
            "mov ss, {0}",

            // jump to stage-2
            "pop {0}",
            "call {0}",

            // enter endless loop in case stage-2 returns
            "2:",
            "jmp 2b",
            out(reg) _,
        );
    }
}
