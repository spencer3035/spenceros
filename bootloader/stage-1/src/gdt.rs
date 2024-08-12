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
    AccessFlags(PRESENT | PRIV_0 | CODE_DATA_DESCRIPTOR | EXECUTABLE | READ_WRITE)
}

#[inline]
pub const fn kernel_data_flags() -> AccessFlags {
    AccessFlags(PRESENT | PRIV_0 | CODE_DATA_DESCRIPTOR | READ_WRITE)
}

#[inline]
pub const fn extra_flags_protected() -> ExtraFlags {
    ExtraFlags(GRANULARITY | PROTECTED_MODE)
}

#[inline]
pub const fn extra_flags_long() -> ExtraFlags {
    ExtraFlags(GRANULARITY | LONG_MODE)
}

pub struct ExtraFlags(u8);
pub struct AccessFlags(u8);

pub struct GdtEntry {
    base: u32,
    limit: u32,
    access_flags: AccessFlags,
    // Only bottom 4 bits should be set in this
    extra_flags: ExtraFlags,
}

impl GdtEntry {
    /// Makes new gdt entry with the given values
    pub fn new(base: u32, limit: u32, access_flags: AccessFlags, extra_flags: ExtraFlags) -> Self {
        GdtEntry {
            base,
            limit,
            access_flags,
            extra_flags,
        }
    }
    /// Gets null GdtEntry
    pub const fn null() -> Self {
        GdtEntry {
            base: 0,
            limit: 0,
            access_flags: AccessFlags(0),
            extra_flags: ExtraFlags(0),
        }
    }

    pub fn bytes(&self) -> [u8; 8] {
        let mut target = [0u8; 8];

        // Encode limit
        target[0] = (self.limit & 0xff) as u8;
        target[1] = (self.limit >> 8) as u8;
        target[6] = (self.limit >> 16) as u8 & 0x0f;

        // Encode base
        target[2] = (self.base & 0xFF) as u8;
        target[3] = (self.base >> 8) as u8;
        target[4] = (self.base >> 16) as u8;
        target[7] = (self.base >> 24) as u8;

        target[5] = self.access_flags.0;
        target[6] |= self.extra_flags.0 << 4;

        target
    }
}
