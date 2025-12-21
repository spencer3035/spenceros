use crate::{
    gdt::Gdt,
    static_items::{
        bios_info::BiosInfo,
        mem::MemInfo,
        vbe_display::{Font, VbeDisplayInfo},
    },
};

/// Helper to define the memory layout.
///
/// The syntax goes like this
///
/// ```ignore
/// layout!(
///     /// Doc comment for VAR_NAME_1
///     0x1000 => VAR_NAME_1:  VarType1,
///     /// Doc comment for VAR_NAME_2
///     0x2000 => VAR_NAME_2:  VarType2,
/// );
/// ```
macro_rules! layout {
    (
        // Capture group for comma separates entries
        $(
            // attribues (for doc comments)
            $(#[$attrs:meta])*
            // address (number literal) => Identifier : Type
            $addr:literal => $name:ident : $type:ty
        ),*
        // Optional comma at end of list
        $(,)?
    ) => {
        // Define constants
        $(
            // This is just a marker, so allow 0 as a pointer
            #[allow(clippy::zero_ptr)]
            $(#[$attrs])*
            pub const $name: *mut $type = $addr as *mut $type;
        )*


        /// Total number of entries
        #[cfg(test)]
        // Kind of cheeky way to get `count = 0 + 1 + 1 + 1 ...;`. Will break if addr is not a
        // numeric literal
        const TOTAL_MEM_ENTRIES : usize = 0 $(+ ($addr + 1) / ($addr + 1))*;

        /// Structure with all start/ends for testing
        #[cfg(test)]
        pub const MAP : test::MemoryMap = test::MemoryMap::new([
            $(
                test::MapEntry::new(
                    $addr,
                    $addr + ::core::mem::size_of::<$type>(),
                    stringify!($name)),
            )*]
        );
    };
}

// - Pointers should not overlap and be documented how large the structures are

layout!(
    /// Reserved, do not use
    0x0000 => _REAL_MODE_IVT : [u8; 0x400],
    /// Reserved, do not use
    0x0400 => _BIOS_DATA_AREA : [u8; 0x100],
    /// VBE Display info, used to print stuff to screen in VBE mode
    0x3000 => VBE_DISPLAY_INFO:  VbeDisplayInfo,
    /// Font for printing in VBE mode
    0x4000 => FONT: Font,
    /// Pointer to the bios info.
    0x5000 => BIOS_INFO: BiosInfo,
    /// Information about system's memory
    0x5010 => MEM_INFO : MemInfo,
    /// Global Descriptor Table (defines memory map)
    0x5418 => GDT_TABLE : Gdt,
    /// Lowest address of the stack.
    ///
    /// The stack grows down so BP should be set to STACK_END (the last address) on boot
    0x6000 => STACK: [u8; STACK_END - 0x6000],
    /// Start of stage 0 in memory
    0x7c00 => STAGE_0_START: [u8; STAGE_0_SECTIONS * 0x200],
    /// BadFS Header
    0x7e00 => BADFS_HEADER: [u8; BADFS_HEADER_SECTIONS * 0x200],
    /// Start of stage 1 in memory
    0x8000 => STAGE_1_START: [u8; STAGE_1_SECTIONS * 0x200],
    /// Start of stage 2 in memory
    0xc200 => STAGE_2_START: [u8; STAGE_2_SECTIONS * 0x200],
    /// Start of stage 3 in memory
    // NOTE: The 16 bit address limit is currently here
    0x10200 => STAGE_3_START: [u8; STAGE_3_SECTIONS * 0x200],
    /// Start of the PML4T, takes up 0x1000 = 8 * 0x200 bytes
    0x0002_0000 => PML4T_START: [u64; 0x200],
    /// Start of the PDPT,  takes up 0x1000 = 8 * 0x200 bytes
    0x0002_1000 => PDPT_START: [u64; 0x200],
    /// Start of the PDT,   takes up 0x1000 = 8 * 0x200 bytes
    0x0002_2000 => PDT_START: [u64; 0x200],
    /// Start of the PT,    takes up 0x1000 = 8 * 0x200 bytes
    0x0002_3000 => PT_START: [u64; 0x200],
    /// Reserved, do not use
    0x0008_0000 => _BIOS_DATA_AREA_EXTENDED : [u8; 0x2_0000],
    /// Reserved, do not use
    0x000A_0000 => _VIDEO_DISPLAY_MEM : [u8; 0x2_0000],
    /// Reserved, do not use
    0x000C_0000 => _VIDEO_BIOS : [u8; 0x8000],
    /// Reserved, do not use
    0x000C_8000 => _BIOS_EXPANSIONS : [u8; 0x2_8000],
    /// Reserved, do not use
    0x000F_0000 => _MOTHERBOARD_BIOS : [u8; 0x2_000],
);

pub const STACK_END: usize = 0x7c00;

/// Number of 512 byte sections stage 0 takes up
pub const STAGE_0_SECTIONS: usize = 1;
/// Number of 512 byte sections the BadFS header takes up
pub const BADFS_HEADER_SECTIONS: usize = 1;
/// Number of 512 byte sections stage 1 takes up
pub const STAGE_1_SECTIONS: usize = 0x21;
/// Number of 512 byte sections stage 2 takes up
pub const STAGE_2_SECTIONS: usize = 0x20;
/// Number of 512 byte sections stage 3 takes up
pub const STAGE_3_SECTIONS: usize = 0x8;
/// Total number of boot sectors we need to read. Not including the 0th boot sector loaded into
/// memory from the bios.
pub const SECTORS_TO_READ: usize =
    STAGE_1_SECTIONS + BADFS_HEADER_SECTIONS + STAGE_2_SECTIONS + STAGE_3_SECTIONS;

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    pub struct MapEntry {
        low: usize,
        high: usize,
        name: &'static str,
    }

    #[cfg(test)]
    impl MapEntry {
        pub const fn new(low: usize, high: usize, name: &'static str) -> Self {
            MapEntry { low, high, name }
        }
    }

    #[cfg(test)]
    impl core::fmt::Display for MapEntry {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "{:25}: [0x{:0>6X}, 0x{:0>6X}]",
                self.name, self.low, self.high
            )
        }
    }

    #[cfg(test)]
    pub struct MemoryMap {
        // START, END, NAME
        entries: [MapEntry; TOTAL_MEM_ENTRIES],
    }

    #[cfg(test)]
    impl MemoryMap {
        pub const fn new(entries: [MapEntry; TOTAL_MEM_ENTRIES]) -> Self {
            MemoryMap { entries }
        }

        fn check(&self) {
            for window in self.entries.windows(2) {
                let curr = &window[0];
                let next = &window[1];
                assert!(
                    curr.high <= next.low,
                    "address overlap: {curr} overlaps with {next}"
                );
            }
        }
    }

    #[cfg(test)]
    impl core::fmt::Display for MemoryMap {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            for entry in self.entries.iter() {
                writeln!(f, "{entry}")?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_no_memory_overlap() {
        println!("{MAP}");
        MAP.check();
    }

    #[test]
    fn test_stages_contiguous() {
        assert_eq!(
            BADFS_HEADER as usize,
            STAGE_0_START as usize + STAGE_0_SECTIONS * 0x200,
            "Expected 0x{:X} to start at 0x{:X}",
            BADFS_HEADER as usize,
            STAGE_0_START as usize + STAGE_0_SECTIONS * 0x200
        );
        assert_eq!(
            STAGE_1_START as usize,
            BADFS_HEADER as usize + BADFS_HEADER_SECTIONS * 0x200,
            "Expected 0x{:X} to start at 0x{:X}",
            STAGE_1_START as usize,
            BADFS_HEADER as usize + BADFS_HEADER_SECTIONS * 0x200
        );
        assert_eq!(
            STAGE_2_START as usize,
            STAGE_1_START as usize + STAGE_1_SECTIONS * 0x200,
            "Expected 0x{:X} to start at 0x{:X}",
            STAGE_2_START as usize,
            STAGE_1_START as usize + STAGE_1_SECTIONS * 0x200
        );
        assert_eq!(
            STAGE_3_START as usize,
            STAGE_2_START as usize + STAGE_2_SECTIONS * 0x200,
            "Expected 0x{:X} to start at 0x{:X}",
            STAGE_3_START as usize,
            STAGE_2_START as usize + STAGE_2_SECTIONS * 0x200,
        );
    }

    #[test]
    fn test_stack_in_range() {
        assert!((STACK as usize) < (u16::MAX as usize));
        // I'm not 100% sure that this is safe, the pointer points to a valid address, but it is
        // not initalized. It seems to give the correct information and it is only in a test, so
        // what's the worst that can happen?
        let stack_end = unsafe { STACK as usize + STACK.as_ref().unwrap().len() };
        assert!(stack_end < (u16::MAX as usize));
    }

    #[test]
    fn test_real_mode_limitations() {
        // Assembly call is limited to (u8::MAX + 1) / 2 sectors it can read. It uses the lower half of a register
        assert!(
            SECTORS_TO_READ <= (u8::MAX as usize + 1) / 2,
            "can't read 0x{:X} sectors",
            SECTORS_TO_READ
        );

        // This might not be required, but I don't trust rust to handle segmentation in 16 bit mode
        // properly
        let real_mode_end: usize =
            STAGE_0_START as usize + 0x200 * (STAGE_0_SECTIONS + STAGE_1_SECTIONS);
        assert!(real_mode_end <= u16::MAX as usize);
    }

    #[test]
    fn test_pages_aligned() {
        assert!(PML4T_START as u64 % 0x1000 == 0, "Page not 4096 aligned");
        assert!(PDPT_START as u64 % 0x1000 == 0, "Page not 4096 aligned");
        assert!(PDT_START as u64 % 0x1000 == 0, "Page not 4096 aligned");
        assert!(PT_START as u64 % 0x1000 == 0, "Page not 4096 aligned");
    }
}
