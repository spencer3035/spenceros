use core::{arch::asm, fmt::Debug};

#[allow(unused)]
pub static GDT: Gdt = Gdt::long_mode();
#[allow(unused)]
pub static GDT_POINTER: GdtPointer = GdtPointer::new(&GDT, Gdt::NUM_ENTRIES as u16);

#[derive(Clone)]
#[repr(transparent)]
pub struct GdtEntry(u64);

impl Debug for GdtEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GdtEntry")
            .field("base", &self.base())
            .field("limit", &self.limit())
            .field("access", &self.access_bytes())
            .field("extra", &self.extra_flags())
            .finish()
    }
}

impl GdtEntry {
    #[inline]
    const fn null() -> GdtEntry {
        GdtEntry(0)
    }

    #[inline]
    const fn data_segment_kernel() -> GdtEntry {
        let extra_flags = ExtraFlags(GRANULARITY | PROTECTED_MODE);
        let access_flags = AccessFlags::data_segment_kernel();
        GdtEntry::new(0, 0xFFFFF, access_flags, extra_flags)
    }

    #[inline]
    const fn code_segment_kernel() -> GdtEntry {
        let extra_flags = ExtraFlags(GRANULARITY | LONG_MODE);
        let access_flags = AccessFlags::code_segment_kernel();
        GdtEntry::new(0, 0xFFFFF, access_flags, extra_flags)
    }

    fn access_bytes(&self) -> AccessFlags {
        let access_flags = self.0.to_le_bytes()[5];
        AccessFlags(access_flags)
    }

    fn extra_flags(&self) -> ExtraFlags {
        ExtraFlags(self.0.to_le_bytes()[6] >> 4)
    }

    fn base(&self) -> u32 {
        let bytes = self.0.to_le_bytes();
        let base_low = bytes[2] as u32;
        let base_mid1 = bytes[3] as u32;
        let base_mid2 = bytes[4] as u32;
        let base_high = bytes[7] as u32;
        base_low | (base_mid1 << 8) | (base_mid2 << 16) | (base_high << 24)
    }

    fn limit(&self) -> u32 {
        let bytes = self.0.to_le_bytes();
        let limit_low = bytes[0] as u32;
        let limit_mid = bytes[1] as u32;
        let limit_high = (bytes[6] & 0x0f) as u32;
        limit_low | (limit_mid << 8) | (limit_high << 16)
    }

    #[inline]
    const fn new(
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
/// Indicates section is in long mode code segment. Mutually exclusive with PROTECTED_MODE
#[allow(dead_code)]
const LONG_MODE: u8 = 1 << 1;

#[inline]
const fn kernel_code_flags() -> AccessFlags {
    AccessFlags(PRESENT | PRIV_0 | CODE_DATA_DESCRIPTOR | EXECUTABLE | READ_WRITE | ACCESSED)
}

#[inline]
const fn kernel_data_flags() -> AccessFlags {
    AccessFlags(PRESENT | PRIV_0 | CODE_DATA_DESCRIPTOR | READ_WRITE | ACCESSED)
}

#[inline]
const fn extra_flags_long() -> ExtraFlags {
    ExtraFlags(GRANULARITY | LONG_MODE)
}

#[derive(Debug)]
struct ExtraFlags(u8);
#[derive(Debug)]
struct AccessFlags(u8);

impl AccessFlags {
    const fn code_segment_kernel() -> Self {
        Self(PRESENT | PRIV_0 | CODE_DATA_DESCRIPTOR | EXECUTABLE | READ_WRITE | ACCESSED)
    }

    const fn data_segment_kernel() -> Self {
        Self(PRESENT | PRIV_0 | CODE_DATA_DESCRIPTOR | READ_WRITE | ACCESSED)
    }
}

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

impl Debug for Gdt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Gdt")
            .field("null", &GdtEntry(self.null.0))
            .field("code", &GdtEntry(self.code.0))
            .field("data", &GdtEntry(self.data.0))
            .finish()
    }
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

#[derive(Debug)]
#[repr(C, packed)]
pub struct GdtPointer {
    num_entries: u16,
    base_address: *const Gdt,
    // TODO: Is this needed? It seems like no
    // We conditionally pad the struct so that it will always be a valid width for 64 bit mode
    // #[cfg(target_pointer_width = "32")]
    // _pad: [u8; 4],
}

unsafe impl Sync for GdtPointer {}

impl GdtPointer {
    const fn new(base_address: &Gdt, num_entries: u16) -> Self {
        Self {
            num_entries,
            base_address,
        }
    }
}

impl Gdt {
    const NUM_ENTRIES: usize = 3;

    const fn long_mode() -> Gdt {
        Gdt {
            null: GdtEntry::null(),
            code: GdtEntry::code_segment_kernel(),
            data: GdtEntry::data_segment_kernel(),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gdt() {
        let gdt = Gdt::long_mode();
        let code = gdt.code;
        let data = gdt.data;
        println!("0xA = 0b{:08b}", 0xA);
        println!("0xC = 0b{:08b}", 0xC);
        assert_eq!(code.clone().extra_flags().0, 0xA);
        assert_eq!(data.clone().extra_flags().0, 0xC);
    }
}
