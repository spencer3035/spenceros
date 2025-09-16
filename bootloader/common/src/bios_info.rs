use crate::{config::BIOS_INFO, static_variable::StaticVariable};

/// Info passed to the kernel
#[repr(C)]
pub struct BiosInfo {
    pub memory_map_start: *const u8,
    pub vba_is_init: bool,
    pub memory_map_count: u32,
}

impl StaticVariable for BiosInfo {
    fn addr() -> *mut BiosInfo {
        BIOS_INFO
    }
}

impl Default for BiosInfo {
    fn default() -> Self {
        Self {
            memory_map_start: core::ptr::null(),
            vba_is_init: false,
            memory_map_count: 0,
        }
    }
}
