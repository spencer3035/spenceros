use crate::config::BIOS_INFO;

/// Info passed to the kernel
#[repr(C)]
pub struct BiosInfo {
    pub memory_map_start: *const u8,
    pub vba_is_init: bool,
    pub memory_map_count: u32,
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
