#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(const_trait_impl)]
#![deny(unsafe_op_in_unsafe_fn)]

//! This stage's purpose is to switch the BIOS into protected mode and get to the next stage.
//!
//! Currently it additionally sets up the VBE display mode because we are not longer able to use
//! BIOS interrupts once we enter protected mode.

use common::{
    config::BIOS_INFO,
    static_items::{
        bios_info::BiosInfo,
        mem::MemInfo,
        static_variable::StaticVariable,
        vbe_display::{Font, VbeDisplayInfo},
    },
};
use common::{println_bios, println_vbe};

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
        // BiosInfo::init();
        Font::init();
        VbeDisplayInfo::init();
        MemInfo::init();
    }
}

/// Main function, we force inline so that rust will clean up the stack
#[inline(never)]
fn main(disk_number: u16) {
    println_bios!("Starting stage 1");
    enable_a20();
    hint_bios_long_mode();
    init_static_values();

    if !has_cpuid() {
        panic!("CPUID not present");
    }

    // prompt_continue();
    init_graphical();
    println_vbe!("update bios is at 0x{:X}", update_bios_info as usize);
    println_vbe!("_start is at 0x{:X}", _start as usize);
    update_bios_info(disk_number);

    // Safety: This is the only mutable reference.
    // let memory_info = unsafe { MemInfo::get_mut() };
    // detect_memory(memory_info);
}

#[inline(never)]
fn update_bios_info(disk_number: u16) {
    // println_vbe!("trying to write = {}", disk_number);
    unsafe {
        {
            // let info: &BiosInfo = &*BIOS_INFO;
            // println_vbe!("addr = 0x{:X}", BIOS_INFO as usize);
            // println_vbe!("before = {}", info.disk_number);
        }
        {
            let info: &mut BiosInfo = &mut *BIOS_INFO;
            info.disk_number = disk_number;

            // let info = BiosInfo {
            //     // disk_number: 128,
            //     disk_number,
            // };
            // BIOS_INFO.write(info);
        }
        {
            // let info: &BiosInfo = &*BIOS_INFO;
            // println_vbe!("after = {}", info.disk_number);
        }
    }
}

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(disk_number: u16) {
    // utils::print_unsafe_fn_location(next_stage);
    main(disk_number);
    println_vbe!("disk_number : {disk_number}");
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
