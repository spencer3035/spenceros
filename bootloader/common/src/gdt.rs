use core::arch::asm;

use crate::{config::GDT_TABLE, static_items::static_variable::StaticVariable};

#[derive(Debug)]
#[repr(transparent)]
pub struct GdtEntry(u64);

impl GdtEntry {
    #[inline]
    pub const fn null() -> GdtEntry {
        GdtEntry(0)
    }

    #[inline]
    pub const fn code_32() -> GdtEntry {
        GdtEntry::new(0, 0xFFFFF, kernel_code_flags(), extra_flags_protected())
    }

    #[inline]
    pub const fn data_32() -> GdtEntry {
        GdtEntry::new(0, 0xFFFFF, kernel_data_flags(), extra_flags_protected())
    }

    #[inline]
    pub const fn data_64() -> GdtEntry {
        GdtEntry::new(0, 0xFFFFF, kernel_data_flags(), extra_flags_long())
    }

    #[inline]
    pub const fn code_64() -> GdtEntry {
        GdtEntry::code_32()
    }

    #[inline]
    pub const fn new(
        base: u32,
        limit: u32,
        access_flags: AccessFlags,
        extra_flags: ExtraFlags,
    ) -> Self {
        let mut target = [0u8; 8];

        // Encode limit
        target[0] = (limit & 0xff) as u8;
        target[1] = (limit >> 8) as u8;
        target[6] = (limit >> 16) as u8 & 0x0f;

        // Encode base
        target[2] = (base & 0xFF) as u8;
        target[3] = (base >> 8) as u8;
        target[4] = (base >> 16) as u8;
        target[7] = (base >> 24) as u8;

        target[5] = access_flags.0;
        target[6] |= extra_flags.0 << 4;

        GdtEntry(u64::from_le_bytes(target))
    }
}

// Access flags:

/// Flag indicating the segment is valid
#[allow(dead_code)]
const PRESENT: u8 = 1 << 7;
/// Ring 0
#[allow(dead_code)]
const PRIV_0: u8 = 0;
/// Ring 1
#[allow(dead_code)]
const PRIV_1: u8 = 1 << 5;
// Ring 2
#[allow(dead_code)]
const PRIV_2: u8 = 2 << 5;
// Ring 3
#[allow(dead_code)]
const PRIV_3: u8 = 3 << 5;
/// Set if code or data segment
#[allow(dead_code)]
const CODE_DATA_DESCRIPTOR: u8 = 1 << 4;
/// Set if a code section
#[allow(dead_code)]
const EXECUTABLE: u8 = 1 << 3;
/// If code segment, flag indicates the section can be executed by lower level sections
/// If data segment, flag indicates growing down
#[allow(dead_code)]
const DIRECTION_CONFORMING: u8 = 1 << 2;
/// If code segment, data can be read
/// If data segment, data can be written
#[allow(dead_code)]
const READ_WRITE: u8 = 1 << 1;
// If the segment has been accessed by the CPU.
#[allow(dead_code)]
const ACCESSED: u8 = 1 << 1;

// Extra flags:

/// Indicates limit is scales BY 4KiB
#[allow(dead_code)]
const GRANULARITY: u8 = 1 << 3;
/// Indicates section is in protected mode. Mutually exclusive with LONG_MODE
#[allow(dead_code)]
const PROTECTED_MODE: u8 = 1 << 2;
/// Indicates section is in long mode. Mutually exclusive with PROTECTED_MODE
#[allow(dead_code)]
const LONG_MODE: u8 = 1 << 1;

#[inline]
pub const fn kernel_extra_flags() -> AccessFlags {
    AccessFlags(GRANULARITY | PROTECTED_MODE)
}

#[inline]
pub const fn kernel_code_flags() -> AccessFlags {
    AccessFlags(PRESENT | PRIV_0 | CODE_DATA_DESCRIPTOR | EXECUTABLE | READ_WRITE | ACCESSED)
}

#[inline]
pub const fn kernel_data_flags() -> AccessFlags {
    AccessFlags(PRESENT | PRIV_0 | CODE_DATA_DESCRIPTOR | READ_WRITE | ACCESSED)
}

#[inline]
pub const fn extra_flags_protected() -> ExtraFlags {
    ExtraFlags(GRANULARITY | PROTECTED_MODE)
}

#[inline]
pub const fn extra_flags_long() -> ExtraFlags {
    ExtraFlags(GRANULARITY | LONG_MODE)
}

#[derive(Debug)]
pub struct ExtraFlags(u8);
#[derive(Debug)]
pub struct AccessFlags(u8);

/// What the gdt looks like in memory.
///
/// Uses 6 bytes when in 32 bit protected mode and 10 bytes when in 64 bit long mode
// Gets written directly to memory
#[allow(dead_code)]
#[repr(C, packed)]
pub struct Gdt {
    null: GdtEntry,
    code: GdtEntry,
    data: GdtEntry,
}

impl Default for Gdt {
    fn default() -> Self {
        Self {
            null: GdtEntry::null(),
            code: GdtEntry::null(),
            data: GdtEntry::null(),
        }
    }
}

impl StaticVariable for Gdt {
    fn addr() -> *mut Self {
        GDT_TABLE
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct GdtPointer {
    pub num_entries: u16,
    pub base_address: *const Gdt,
    // TODO: Is this needed? It seems like no
    // We conditionally pad the struct so that it will always be a valid width for 64 bit mode
    // #[cfg(target_pointer_width = "32")]
    // _pad: [u8; 4],
}

unsafe impl Sync for GdtPointer {}

impl GdtPointer {
    pub const fn new(base_address: &Gdt, num_entries: u16) -> Self {
        Self {
            num_entries,
            base_address,
        }
    }

    pub const fn null() -> Self {
        Self {
            num_entries: 0,
            base_address: core::ptr::null(),
        }
    }
}

impl Gdt {
    pub const NUM_ENTRIES: usize = 3;

    pub const fn protected_mode() -> Gdt {
        Gdt {
            null: GdtEntry::null(),
            code: GdtEntry::code_32(),
            data: GdtEntry::data_32(),
        }
    }

    pub const fn long_mode() -> Gdt {
        Gdt {
            null: GdtEntry::null(),
            code: GdtEntry::code_64(),
            data: GdtEntry::data_64(),
        }
    }

    /// Disable cli and load gdt
    ///
    /// # Safety
    /// Need to understand the consequences of loading a GDT and disabling inturrupts
    pub unsafe fn disable_cli_and_load(&self) {
        unsafe {
            asm!(
                "cli",
                "lgdt [{}]",
                in(reg) self,
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}
