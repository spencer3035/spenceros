#![no_std]

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
pub const STAGE_2_SECTIONS: usize = 0x2000 / 0x200;
/// Location of the GDT pointer
pub const GDT_POINTER: *mut GdtPointer = 0x80 as *mut GdtPointer;
/// Location of the GDT
pub const GDT_START: *mut u8 = 0x100 as *mut u8;
