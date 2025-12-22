use crate::{config::BIOS_INFO, static_items::static_variable::StaticVariable};

/// Info passed to the kernel
#[repr(C)]
#[derive(Default)]
pub struct BiosInfo {
    pub disk_number: u16,
    // pub memory_map_start: *const u8,
    // pub memory_map_count: u32,
}

impl StaticVariable for BiosInfo {
    fn addr() -> *mut BiosInfo {
        BIOS_INFO
    }
}

// impl Default for BiosInfo {
//     fn default() -> Self {
//         Self {
//             memory_map_start: core::ptr::null(),
//             memory_map_count: 0,
//         }
//     }
// }
