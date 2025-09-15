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

/// Information that we pass to the kernel/OS
pub mod bios_info;
/// Static string methods
pub mod static_string;
/// Information about VBE display
pub mod vbe_display;
