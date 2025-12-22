#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(const_trait_impl)]
#![deny(unsafe_op_in_unsafe_fn)]

//! This stage's purpose is to switch the BIOS into protected mode and get to the next stage.
//!
//! Currently it additionally sets up the VBE display mode because we are not longer able to use
//! BIOS interrupts once we enter protected mode.

use core::arch::asm;

use common::config::{
    BADFS_HEADER, SCRATCH, STAGE_0_START, STAGE_1_START, STAGE_2_START, STAGE_3_START,
};
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
        BiosInfo::init();
        Font::init();
        VbeDisplayInfo::init();
        MemInfo::init();
    }
}

/// Main function, we force inline so that rust will clean up the stack
#[inline(never)]
fn main(disk_number: u16) {
    enter_unreal();
    println_bios!("Starting stage 1");
    enable_a20();
    hint_bios_long_mode();
    init_static_values();

    if !has_cpuid() {
        panic!("CPUID not present");
    }

    init_graphical();
    update_bios_info(disk_number);

    // Safety: This is the only mutable reference.
    let memory_info = unsafe { MemInfo::get_mut() };
    detect_memory(memory_info);
}

fn enter_unreal() {
    todo!()
}

#[inline(never)]
fn update_bios_info(disk_number: u16) {
    unsafe {
        let info: &mut BiosInfo = &mut *BIOS_INFO;
        info.disk_number = disk_number;
    }
}

#[unsafe(link_section = ".start")]
#[unsafe(no_mangle)]
pub extern "C" fn _start(disk_number: u16) {
    // utils::print_unsafe_fn_location(next_stage);
    main(disk_number);
    println_vbe!("load_disk: 0x{:X}", load_disk as usize);
    let disk_start_block = 1;
    // let memory_address = STAGE_3_START as u32;
    // let memory_address = STAGE_0_START as u32;
    // This seems to only work if less than 0xFFFF.
    // Maybe it is segment offset? 0xFFFF:0xFFFF
    let memory_address = 0x4000;
    println_vbe!("memory address : 0x{:X}", memory_address);
    let num_blocks = 1;
    load_disk(disk_number, disk_start_block, memory_address, num_blocks);
    println_vbe!("Stoping before stage 2");
    loop {}
    unsafe {
        next_stage();
    }
}

#[repr(C, align(4))]
struct DiskAddressPacket {
    /// Size of this packet (0x10 or 0x18, depending on if last entry is present)
    size: u8,
    /// reserved
    res: u8,
    /// number of 0x200 (?) blocks to transfer
    blocks: u16,
    /// Transfer buffer
    transfer_buffer_offset: u16,
    transfer_buffer_segment: u16,
    /// Starting block number
    start_block_low: u32,
    start_block_high: u32,
    // Optional: 64-bit flat address of the transfer buffer.
    // This is used if blocks is 0xFFFF:0xFFFF
    // flat_address_low: u32,
    // flat_address_high: u32,
}

impl DiskAddressPacket {
    /// Create a new packet to transfer `blocks` blocks from `from` to `to`
    fn new(from_block: u64, to: u32, blocks: u16) -> Self {
        // Self {
        //     size: 0x10,
        //     res: 0,
        //     blocks,
        //     transfer_buffer: to,
        //     start_block: from_block,
        //     flat_address: 0,
        // }

        // For some reason, setting the transfer buffer to 0xFFFF_FFFF doesn't seem to work as
        // described and it won't read the flat_address
        // Self {
        //     size: 0x18,
        //     res: 0,
        //     blocks,
        //     transfer_buffer_offset: 0xFFFF,
        //     transfer_buffer_segment: 0xFFFF,
        //     start_block: from_block,
        //     flat_address_low: to,
        //     flat_address_high: 0,
        // }

        // For some reason, setting the transfer buffer to 0xFFFF_FFFF doesn't seem to work as
        // described and it won't read the flat_address
        Self {
            size: 0x18,
            res: 0,
            blocks,
            transfer_buffer_offset: 0xFFFF,
            transfer_buffer_segment: 0xFFFF,
            start_block_low: (from_block & 0xFFFF_FFFF) as u32,
            start_block_high: 0,
        }
    }
}

// INT 13 - IBM/MS INT 13 Extensions - EXTENDED READ
// 	AH = 42h
// 	DL = drive number
// 	DS:SI -> disk address packet (see #00272)
// Return: CF clear if successful
// 	    AH = 00h
// 	CF set on error
// 	    AH = error code (see #00234)
// 	    disk address packet's block count field set to number of blocks
// 	      successfully transferred
// SeeAlso: AH=02h,AH=41h"INT 13 Ext",AH=43h"INT 13 Ext"
// Format of disk address packet:
// Offset	Size	Description	(Table 00272)
//  00h	BYTE	size of packet (10h or 18h)
//  01h	BYTE	reserved (0)
//  02h	WORD	number of blocks to transfer (max 007Fh for Phoenix EDD)
//  04h	DWORD	-> transfer buffer
//  08h	QWORD	starting absolute block number
// 		(for non-LBA devices, compute as
// 		  (Cylinder*NumHeads + SelectedHead) * SectorPerTrack +
// 		  SelectedSector - 1
//  10h	QWORD	(EDD-3.0, optional) 64-bit flat address of transfer buffer;
// 		  used if DWORD at 04h is FFFFh:FFFFh

#[inline(never)]
fn load_disk(disk_number: u16, disk_start_block: u64, memory_address: u32, num_blocks: u16) {
    let hdr_addr_orig = BADFS_HEADER;
    let hdr_addr_new = memory_address as *mut [u8; 0x200];
    unsafe {
        println_vbe!("before:");
        println_vbe!("{:X?}", &(&(*hdr_addr_orig))[0..10]);
        println_vbe!("{:X?}", &(&(*hdr_addr_new))[0..10]);
    }
    // END test

    // INTERRUP.B:3590
    let mut packet = DiskAddressPacket::new(disk_start_block, memory_address, num_blocks);

    // INT 13 op code for EXTENDED READ
    let mut ax: u16;
    // Drive Number
    let dl: u8 = disk_number as u8;
    // println_vbe!("dl = {dl}");
    // DS:SI disk address packet
    let si: *mut DiskAddressPacket = &mut packet;
    // println_vbe!("ds = 0x{:X}", ds as usize);

    println_vbe!("packet addr = 0x{:X}", si as usize);

    unsafe {
        asm!(
            // Save SI so we don't mess up rust/LLVM
            "push si",
            "mov si, cx",
            // Do interrupt
            "mov ah, 0x42",
            "int 0x13",
            // Carry is set on error, but AH != 0 if there is an error, so we check that
            // Put si back
            "pop si",
            out("ax") ax,
            in("dl") dl,
            in("cx") si,
        );
    }

    // let is_error = (ax & 0x00FF) == 1;
    let is_error = (ax & 0xFF00) != 0;
    if is_error {
        panic!("error loading from disk (AH = 0x{:02X})", ax >> 8);
    }
    let blocks_transfered = packet.blocks;
    println_vbe!("transfered {} blocks", blocks_transfered);
    let hdr_addr_orig = BADFS_HEADER;
    // let hdr_addr_new = memory_address as *mut [u8; 0x200];
    let hdr_addr_new = memory_address as *mut [u8; 0x200];
    unsafe {
        println_vbe!("after:");
        println_vbe!("{:X?}", &(&(*hdr_addr_orig))[0..10]);
        println_vbe!("{:X?}", &(&(*hdr_addr_new))[0..10]);
        let hdr_orig = badfs::header::HeaderDef::read(&*hdr_addr_orig).unwrap();
        let hdr_new = badfs::header::HeaderDef::read(&*hdr_addr_new).unwrap();
        if hdr_new != hdr_orig {
            println_vbe!("headers not equal");
        } else {
            println_vbe!("headers equal");
        }
    }
}

enum Int13hResult {
    Success = 0x00,
    InvalidFunction = 0x01,
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
