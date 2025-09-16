use core::ops::Deref;

use crate::{
    config::{FONT, VBE_DISPLAY_INFO},
    io::framebuffer::FramebufferInfo,
    static_variable::StaticVariable,
};

/// Light wrapper around font array
#[repr(transparent)]
pub struct Font {
    inner: [u8; 0x1000],
}

impl Deref for Font {
    type Target = [u8; 0x1000];
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for Font {
    fn default() -> Self {
        Self { inner: [0; 0x1000] }
    }
}

impl StaticVariable for Font {
    fn addr() -> *mut Self {
        FONT
    }
}

/// Info needed to use the VBE display
#[repr(C)]
pub struct VbeDisplayInfo {
    pub framebuffer: FramebufferInfo,
    pub font: *const Font,
    pub char_index: u32,
    pub is_init: bool,
}

impl Default for VbeDisplayInfo {
    fn default() -> Self {
        Self {
            framebuffer: FramebufferInfo::null(),
            font: core::ptr::null(),
            char_index: 0,
            is_init: false,
        }
    }
}

impl StaticVariable for VbeDisplayInfo {
    fn addr() -> *mut Self {
        VBE_DISPLAY_INFO
    }
}
