#![no_std]
#![allow(ambiguous_glob_imports)]

#[cfg(all(feature = "real_mode", feature = "protected_mode"))]
compile_error!("feature \"real_mode\" and \"protected_mode\" cannot be enabled at the same time");

#[cfg(feature = "real_mode")]
pub mod real_mode;
#[cfg(feature = "real_mode")]
pub use real_mode::*;

#[cfg(feature = "protected_mode")]
pub mod protected_mode;
#[cfg(feature = "protected_mode")]
pub use protected_mode::io::*;
#[cfg(feature = "protected_mode")]
pub use protected_mode::*;

pub mod gdt;
use gdt::GdtPointer;

/// Number of 512 byte sections stage 1 takes up
pub const STAGE_1_SECTIONS: usize = 2;
/// Number of 512 byte sections stage 2 takes up
pub const STAGE_2_SECTIONS: usize = 0x10;

// Pointers to memory. These should not overlap and be documented how large each of the sections
// are needed

/// Lowest address of the stack, grows down so BP should be set to STACK_END
pub const STACK_START: *mut u8 = 0x0 as *mut u8;
/// Highest address of the stack, stack grows down so BP should be set to this value
pub const STACK_END: *mut u8 = 0x1000 as *mut u8;

/// Start of the PML4T, takes up 0x1000 bytes
pub const PML4T_START: *mut u8 = 0x1000 as *mut u8;
/// Start of the PDPT, takes up 0x1000 bytes
pub const PDPT_START: *mut u8 = 0x2000 as *mut u8;
/// Start of the PDT, takes up 0x1000 bytes
pub const PDT_START: *mut u8 = 0x3000 as *mut u8;
/// Start of the PT, takes up 0x1000 bytes
pub const PT_START: *mut u8 = 0x4000 as *mut u8;

/// Location of the GDT pointer, contains a u16 and u32, so 6 bytes
pub const GDT_POINTER: *mut GdtPointer = 0x80 as *mut GdtPointer;
/// Location of the GDT, size depends on how many tables
pub const GDT_START: *mut u8 = 0x100 as *mut u8;
