use crate::{
    config::VBE_DISPLAY_INFO,
    io::framebuffer::{Font, FramebufferInfo},
};

#[repr(C)]
pub struct VbeDisplayInfo {
    pub framebuffer: FramebufferInfo,
    pub font: *const Font,
    pub char_index: u32,
    pub is_init: bool,
}

impl VbeDisplayInfo {
    const fn null() -> Self {
        Self {
            framebuffer: FramebufferInfo::null(),
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
