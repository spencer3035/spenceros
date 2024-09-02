#![cfg_attr(not(test), no_std)]

/// Config values for memory and sizes of files
pub mod config;
/// Global Descriptor Table logic
pub mod gdt;
/// Input/output
pub mod io {
    /// Bios writer that can be used in Real Mode
    pub mod bios;
    /// Framebuffer that can be used in Real Mode, Protected Mode, and Long Mode
    pub mod framebuffer;
    /// VGA Text Mode that can be used in Real Mode, Protected Mode, and Long Mode
    pub mod text_mode;
}

/// Info passed to the kernel
#[repr(C)]
pub struct BiosInfo {
    pub memory_map_start: *const u8,
    pub memory_map_count: usize,
    pub framebuffer: io::framebuffer::FramebufferInfo,
}
