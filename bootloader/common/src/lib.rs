#![cfg_attr(not(test), no_std)]

use crate::{
    config::{BIOS_INFO, VBE_DISPLAY_INFO},
    io::framebuffer::Font,
};

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

pub mod static_string {
    pub struct StringOverflow;

    impl core::fmt::Display for StringOverflow {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "String Overflow, max size = {}", STATIC_STRING_LENGTH)
        }
    }

    const STATIC_STRING_LENGTH: usize = 255;
    pub struct StaticString {
        chars: [char; STATIC_STRING_LENGTH],
        len: usize,
    }

    impl StaticString {
        pub const fn new() -> Self {
            StaticString {
                chars: ['\0'; STATIC_STRING_LENGTH],
                len: 0,
            }
        }

        /// Returns error if overflow and the number of characters left if it succeeds
        pub fn push(&mut self, ch: char) -> Result<usize, StringOverflow> {
            if self.len >= STATIC_STRING_LENGTH {
                Err(StringOverflow)
            } else {
                self.chars[self.len] = ch;
                self.len += 1;
                Ok(STATIC_STRING_LENGTH - self.len)
            }
        }

        pub fn clear(&mut self) {
            self.len = 0;
        }
    }

    impl Default for StaticString {
        fn default() -> Self {
            Self::new()
        }
    }

    impl core::fmt::Display for StaticString {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            if self.len > 0 {
                for ch in self.chars[0..self.len].iter() {
                    write!(f, "{}", ch)?;
                }
                Ok(())
            } else {
                Ok(())
            }
        }
    }
}

/// Info passed to the kernel
#[repr(C)]
pub struct BiosInfo {
    pub memory_map_start: *const u8,
    pub vba_is_init: bool,
    pub memory_map_count: u32,
}

#[repr(C)]
pub struct VbeDisplayInfo {
    pub framebuffer: io::framebuffer::FramebufferInfo,
    pub font: *const Font,
    pub char_index: u32,
    pub is_init: bool,
}

impl VbeDisplayInfo {
    const fn null() -> Self {
        Self {
            framebuffer: io::framebuffer::FramebufferInfo::null(),
            font: 0 as *const Font,
            char_index: 0,
            is_init: false,
        }
    }

    /// Init the with null information
    ///
    /// # Safety
    ///
    /// This function should only be called once. It is also not thread safe
    #[allow(dead_code)]
    #[inline(never)]
    pub unsafe fn init() {
        unsafe {
            *VBE_DISPLAY_INFO = VbeDisplayInfo::null();
        }
    }

    /// Gets mutable reference
    ///
    /// # Safety
    ///
    /// The following two conditions need to be met:
    /// - [Self::init()] has been called to initialize the memory
    /// - Need to manually enforce borrowing rules. Only one mutable reference can exist at a time
    pub unsafe fn get_mut() -> &'static mut Self {
        {
            VBE_DISPLAY_INFO.as_mut().unwrap()
        }
    }

    /// Gets reference
    ///
    /// # Safety
    ///
    /// The following two conditions need to be met:
    /// - [Self::init()] has been called to initialize the memory
    /// - Need to manually enforce borrowing rules.
    pub unsafe fn get() -> &'static Self {
        {
            VBE_DISPLAY_INFO.as_ref().unwrap()
        }
    }
}

impl BiosInfo {
    const fn null() -> Self {
        BiosInfo {
            memory_map_start: 0 as *const u8,
            vba_is_init: false,
            memory_map_count: 0,
        }
    }

    /// Init the bios info with null information
    ///
    /// # Safety
    ///
    /// This function should only be called once. It is also not thread safe
    #[allow(dead_code)]
    #[inline(never)]
    pub unsafe fn init() {
        unsafe {
            *BIOS_INFO = BiosInfo::null();
        }
    }

    /// Gets mutable reference to info
    ///
    /// # Safety
    ///
    /// The following two conditions need to be met:
    /// - [Self::init()] has been called to initialize the memory
    /// - Need to manually enforce borrowing rules. Only one mutable reference can exist at a time
    pub unsafe fn get_mut() -> &'static mut Self {
        {
            BIOS_INFO.as_mut().unwrap()
        }
    }

    /// Gets reference to info
    ///
    /// # Safety
    ///
    /// The following two conditions need to be met:
    /// - [Self::init()] has been called to initialize the memory
    /// - Need to manually enforce borrowing rules.
    pub unsafe fn get() -> &'static Self {
        {
            BIOS_INFO.as_ref().unwrap()
        }
    }
}
