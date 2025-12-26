#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(const_trait_impl)]
#![deny(unsafe_op_in_unsafe_fn)]

//! This stage's purpose is to switch the BIOS into protected mode and get to the next stage.
//!
//! Currently it additionally sets up the VBE display mode because we are not longer able to use
//! BIOS interrupts once we enter protected mode.

use core::arch::asm;
use core::ptr::addr_of;

use badfs::header::HeaderDef;
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

#[repr(align(16))]
struct Buffer<const LEN: usize> {
    buf: [u8; LEN],
}

impl<const LEN: usize> Buffer<LEN> {
    const fn new() -> Self {
        Self { buf: [0; LEN] }
    }
}

const BUFFER_SECTIONS: usize = 10;
const BUFFER_LEN: usize = 0x200 * BUFFER_SECTIONS;
static mut BUFFER: Buffer<BUFFER_LEN> = Buffer::new();

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
    let memory_address = unsafe { &raw mut BUFFER.buf as *mut [u8; BUFFER_LEN] as u16 };
    println_vbe!("memory address : 0x{:X}", memory_address);
    let num_blocks = 1;
    load_disk(disk_number, disk_start_block, memory_address, num_blocks);
    unsafe {
        if test_header_eq(
            addr_of!(BUFFER.buf) as *const () as *const HeaderDef,
            BADFS_HEADER as *const HeaderDef,
        ) {
            println_vbe!("headers equal 1");
        } else {
            println_vbe!("Headers not equal 1");
        }

        let far_addr = 0x60_0000;
        copy(memory_address as *const u8, far_addr as *mut u8, 0x200);

        if test_header_eq(
            memory_address as *const HeaderDef,
            far_addr as *const HeaderDef,
        ) {
            println_vbe!("headers equal 2");
        } else {
            println_vbe!("Headers not equal 2");
        }
    }
    println_vbe!("Stoping before stage 2");
    loop {}
    unsafe {
        next_stage();
    }
}

unsafe fn copy(src: *const u8, dst: *mut u8, len: usize) {
    for offset in 0..len {
        unsafe { *dst.add(offset) = *src.add(offset) }
    }
}

fn test_header_eq(a: *const HeaderDef, b: *const HeaderDef) -> bool {
    unsafe { &*a == &*b }
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
    start_block: u64,
    // Optional: 64-bit flat address of the transfer buffer.
    // This is used if blocks is 0xFFFF:0xFFFF
    // flat_address_low: u32,
    // flat_address_high: u32,
}

impl DiskAddressPacket {
    /// Create a new packet to transfer `blocks` blocks from `from` to `to`
    fn new(from_block: u64, to: u16, blocks: u16) -> Self {
        Self {
            size: 0x10,
            res: 0,
            blocks,
            transfer_buffer_segment: 0,
            transfer_buffer_offset: to,
            start_block: from_block,
        }
    }
}

struct DiskRw {
    disk_number: u16,
    buffer: Buffer<BUFFER_LEN>,
}

impl DiskRw {
    pub fn load_from_disk(
        &mut self,
        disk_addr: u64,
        mem_addr: u64,
        size: usize,
    ) -> Result<usize, ()> {
        const SECTION: u64 = 0x200;
        let disk_start_block = disk_addr / SECTION;
        let disk_start_offset = disk_addr % SECTION;
        let disk_end_addr = disk_addr + size as u64;
        let disk_end_block = disk_end_addr.div_ceil(SECTION);
        let disk_end_offset = disk_end_addr % SECTION;

        let mut num_to_read = disk_end_block - disk_start_block;
        let mut block_chunk_ii = disk_start_block;
        let mut is_first = true;
        let mut target_addr = mem_addr;
        loop {
            let sectors_read = if num_to_read > BUFFER_SECTIONS as u64 {
                // Load full buffer and copy
                self.fill_buffer(block_chunk_ii);
                num_to_read -= BUFFER_SECTIONS as u64;
                BUFFER_SECTIONS as u64
            } else if num_to_read > 0 {
                // Load partial buffer
                self.fill_buffer_partial(block_chunk_ii, num_to_read as u16);
                let tmp = num_to_read;
                num_to_read = 0;
                tmp
            } else {
                break;
            };
            block_chunk_ii += sectors_read;

            let is_last = num_to_read == 0;
            let start_addr = if is_first { disk_start_offset } else { 0 };
            let end_addr = if is_last {
                SECTION * (sectors_read - 1) + disk_end_offset
            } else {
                sectors_read * SECTION
            };

            let len = end_addr - start_addr;
            let src = start_addr as *const u8;
            unsafe {
                copy(src, target_addr as *mut u8, len as usize);
            }
            target_addr += len;

            is_first = false;
        }

        Ok((target_addr - mem_addr) as usize)
    }

    fn fill_buffer(&mut self, disk_start_block: u64) {
        let num_blocks = BUFFER_SECTIONS as u16;
        self.fill_buffer_partial(disk_start_block, num_blocks);
    }

    /// `num_blocks` should be less than `BUFFER_SECTIONS`
    fn fill_buffer_partial(&mut self, disk_start_block: u64, num_blocks: u16) {
        let memory_address = &mut self.buffer.buf as *mut u8 as u16;
        load_disk(
            self.disk_number,
            disk_start_block,
            memory_address,
            num_blocks,
        );
    }
}

fn load_disk(
    disk_number: u16,
    disk_start_block: u64,
    memory_address: u16,
    num_blocks: u16,
) -> Result<(), Int13hError> {
    // INTERRUP.B:3590 from inturrupt list
    let mut packet = DiskAddressPacket::new(disk_start_block, memory_address, num_blocks);

    // INT 13 op code for EXTENDED READ
    let mut ax: u16;
    // Drive Number
    let dl: u8 = disk_number as u8;
    // println_vbe!("dl = {dl}");
    // DS:SI disk address packet
    let si: *mut DiskAddressPacket = &mut packet;
    // println_vbe!("ds = 0x{:X}", ds as usize);

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

    if let Some(err) = Int13hError::from_ax(ax) {
        Err(err)
    } else {
        Ok(())
    }
}

#[repr(u8)]
enum Int13hError {
    Unknown,
    InvalidFunctionOrParam,           // 0x01
    AddressMarkNotFound,              // 0x02
    DiskWriteProtected,               // 0x03
    SectorNotFoundReadError,          // 0x04
    ResetFailed,                      // 0x05
    DataDidNotVerify,                 // 0x05
    DiskChanged,                      // 0x06
    DriveParameterActivityFailed,     // 0x07
    DMAOverrun,                       // 0x08
    DataBoundaryError,                // 0x09
    BadSectorDetected,                // 0x0A
    BadTrackDetected,                 // 0x0B
    UnsupportedTrackOrInvalidMedia,   // 0x0C
    InvalidNumberOfSectorsOnFormat,   // 0x0D
    ControlDataAddressMarkDetected,   // 0x0E
    DMAArbitrationLevelOutOfRange,    // 0x0F
    UncorrectableCRCOrECCErrorOnRead, // 0x10
    DataECCCorrected,                 // 0x11
    ControllerFailure,                // 0x20
    NoMediaInDrive,                   // 0x31
    IncorrectDriveTypeStoredInCMOS,   // 0x32
    SeekFailed,                       // 0x40
    Timeout,                          // 0x80
    DriveNotReady,                    // 0xAA
    VolumeNotLockedInDrive,           // 0xB0
    VolumeLockedInDrive,              // 0xB1
    VolumeNotRemovable,               // 0xB2
    VolumeInUse,                      // 0xB3
    LockCountExceeded,                // 0xB4
    ValidEjectRequestFailed,          // 0xB5
    VolumePresentButReadProtected,    // 0xB6
    UndefinedError,                   // 0xBB
    WriteFault,                       // 0xCC
    StatusRegisterError,              // 0xE0
    SenseOperationFailed,             // 0xFF
}

impl Int13hError {
    fn from_ax(ax: u16) -> Option<Self> {
        let ah = (ax >> 8) as u8;
        if ah == 0 {
            return None;
        }

        let err = match ah {
            0x01 => Self::InvalidFunctionOrParam,
            0x02 => Self::AddressMarkNotFound,
            0x03 => Self::DiskWriteProtected,
            0x04 => Self::SectorNotFoundReadError,
            0x05 => Self::ResetFailed,
            0x05 => Self::DataDidNotVerify,
            0x06 => Self::DiskChanged,
            0x07 => Self::DriveParameterActivityFailed,
            0x08 => Self::DMAOverrun,
            0x09 => Self::DataBoundaryError,
            0x0A => Self::BadSectorDetected,
            0x0B => Self::BadTrackDetected,
            0x0C => Self::UnsupportedTrackOrInvalidMedia,
            0x0D => Self::InvalidNumberOfSectorsOnFormat,
            0x0E => Self::ControlDataAddressMarkDetected,
            0x0F => Self::DMAArbitrationLevelOutOfRange,
            0x10 => Self::UncorrectableCRCOrECCErrorOnRead,
            0x11 => Self::DataECCCorrected,
            0x20 => Self::ControllerFailure,
            0x31 => Self::NoMediaInDrive,
            0x32 => Self::IncorrectDriveTypeStoredInCMOS,
            0x40 => Self::SeekFailed,
            0x80 => Self::Timeout,
            0xAA => Self::DriveNotReady,
            0xB0 => Self::VolumeNotLockedInDrive,
            0xB1 => Self::VolumeLockedInDrive,
            0xB2 => Self::VolumeNotRemovable,
            0xB3 => Self::VolumeInUse,
            0xB4 => Self::LockCountExceeded,
            0xB5 => Self::ValidEjectRequestFailed,
            0xB6 => Self::VolumePresentButReadProtected,
            0xBB => Self::UndefinedError,
            0xCC => Self::WriteFault,
            0xE0 => Self::StatusRegisterError,
            0xFF => Self::SenseOperationFailed,
            _ => Self::Unknown,
        };

        Some(err)
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
