use static_assertions::const_assert;

use super::BiosInfo;
use core::mem::size_of;

// 0x0000 to 0x1000 is a no-go zone. It contains inturrupt vector information

// Pointers should not overlap and be documented how large the structures are

/// Start of the PML4T, takes up 0x1000 = 8 * 0x200 bytes
pub const PML4T_START: *mut [u64; 0x200] = 0x1000 as *mut [u64; 0x200];
/// Start of the PDPT,  takes up 0x1000 = 8 * 0x200 bytes
pub const PDPT_START: *mut [u64; 0x200] = 0x2000 as *mut [u64; 0x200];
/// Start of the PDT,   takes up 0x1000 = 8 * 0x200 bytes
pub const PDT_START: *mut [u64; 0x200] = 0x3000 as *mut [u64; 0x200];
/// Start of the PT,    takes up 0x1000 = 8 * 0x200 bytes
pub const PT_START: *mut [u64; 0x200] = 0x4000 as *mut [u64; 0x200];

/// Pointer to the bios info. Can't get exact size without unstable feature
pub const BIOS_INFO: *mut BiosInfo = 0x5000 as *mut BiosInfo;
/// Start of the memory map, each entry is 24 bytes, number of entries is not known at compiletimw,
/// but in the emulator it is 7 entries which would be 7*24=168 bytes
pub const MEMORY_MAP_START: *mut u8 = (0x5000 + size_of::<BiosInfo>()) as *mut u8;

/// Lowest address of the stack, grows down so BP should be set to STACK_END
pub const STACK_START: *mut u8 = 0x6000 as *mut u8;
/// Highest address of the stack, stack grows down so BP should be set to this value
pub const STACK_END: *mut u8 = 0x7000 as *mut u8;

/// The start of the first stage in memory, defined by BIOS
pub const STAGE_0_START: usize = 0x7c00;
/// Number of 512 byte sections stage 0 takes up
pub const STAGE_0_SECTIONS: usize = 1;
/// Number of 512 byte sections stage 1 takes up
pub const STAGE_1_SECTIONS: usize = 0x30;
/// Number of 512 byte sections stage 2 takes up
pub const STAGE_2_SECTIONS: usize = 0x10;
/// Number of 512 byte sections stage 3 takes up
pub const STAGE_3_SECTIONS: usize = 0x20;
/// End address of bootloader
pub const BOOTLOADER_END: usize = STAGE_0_START + 0x200 * (TOTAL_SECTORS);
const_assert!(BOOTLOADER_END < 0x20000);
/// End of real mode (16 bit) addresses
pub const REAL_MODE_END: usize = STAGE_0_START + 0x200 * (STAGE_0_SECTIONS + STAGE_1_SECTIONS);
const_assert!(REAL_MODE_END <= u16::MAX as usize);

/// Total number of boot sectors we need to read. Not including the 0th boot sector loaded into
/// memory from the bios.
pub const SECTORS_TO_READ: usize = STAGE_1_SECTIONS + STAGE_2_SECTIONS + STAGE_3_SECTIONS;
pub const TOTAL_SECTORS: usize = STAGE_0_SECTIONS + SECTORS_TO_READ;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stack_in_range() {
        assert!((STACK_START as usize) < (u16::MAX as usize));
        assert!((STACK_END as usize) < (u16::MAX as usize));
    }

    #[test]
    fn test_sectors_readable() {
        assert!(SECTORS_TO_READ <= u8::MAX as usize);
        assert!(
            STAGE_2_SECTIONS as u32 * 0x200 + (STAGE_0_START as u32) < 0xffff,
            "Address outside of 16 bit range"
        )
    }

    #[test]
    fn test_pages_aligned() {
        assert!(PML4T_START as u64 % 0x1000 == 0, "Page not 4096 aligned");
        assert!(PDPT_START as u64 % 0x1000 == 0, "Page not 4096 aligned");
        assert!(PDT_START as u64 % 0x1000 == 0, "Page not 4096 aligned");
        assert!(PT_START as u64 % 0x1000 == 0, "Page not 4096 aligned");
    }
}
