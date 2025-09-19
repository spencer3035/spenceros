#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(const_trait_impl)]
#![deny(unsafe_op_in_unsafe_fn)]

use common::println_bios;
use common::static_items::{
    bios_info::BiosInfo,
    mem::MemInfo,
    static_variable::StaticVariable,
    vbe_display::{Font, VbeDisplayInfo},
};

use vbe::init_graphical;

mod mem;
mod protected_mode;
mod vbe;

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
    println_bios!("Starting stage 1");
    enable_a20();
    init_static_values();

    if !has_cpuid() {
        panic!("CPUID not present");
    }

    // prompt_continue();
    init_graphical();

    // Safety: This is the only mutable reference.
    let memory_info = unsafe { MemInfo::get_mut() };
    detect_memory(memory_info);
}

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) {
    utils::print_unsafe_fn_location(next_stage);
    main(_disk_number);
    unsafe {
        next_stage();
    }
}

#[inline(never)]
unsafe fn next_stage() {
    unsafe {
        protected_mode::disable_interrupts();
        protected_mode::load_protected_gdt();
        protected_mode::set_protected_flag();
        protected_mode::jump_next_stage();
    }
}
