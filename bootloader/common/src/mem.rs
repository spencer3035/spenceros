use crate::{config::MEM_INFO, static_variable::StaticVariable};

#[repr(C)]
pub struct MemInfo {
    num_entries: u8,
    table: [MemEntry; Self::NUM_ENTRIES],
}

impl MemInfo {
    /// The max number of entries that can be stored
    pub const NUM_ENTRIES: usize = 64;

    /// Get null table for initalization
    pub const fn null() -> Self {
        Self {
            num_entries: 0,
            table: [const { MemEntry::null() }; Self::NUM_ENTRIES],
        }
    }
}

impl Default for MemInfo {
    fn default() -> Self {
        Self {
            num_entries: 0,
            table: [const { MemEntry::null() }; Self::NUM_ENTRIES],
        }
    }
}

impl StaticVariable for MemInfo {
    fn addr() -> *mut Self {
        MEM_INFO
    }
}

/// An entry denoting a location in ram and a length of avaliable memory
#[repr(C)]
pub struct MemEntry {
    /// The physical address of the memory
    physical_address: u64,
    /// The size (in bytes) the memory is avaliable at
    length: u64,
}

impl MemEntry {
    pub const fn null() -> Self {
        Self {
            physical_address: 0,
            length: 0,
        }
    }
}
