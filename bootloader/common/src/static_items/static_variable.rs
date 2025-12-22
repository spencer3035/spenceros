pub trait StaticVariable: Sized + Default {
    /// Address the value is stored at
    fn addr() -> *mut Self;

    /// Init the static variable with the default information
    ///
    /// # Safety
    ///
    /// This function should only be called once. It is also not thread safe
    unsafe fn init() {
        unsafe {
            *Self::addr() = Self::default();
        }
    }

    /// Gets mutable reference to contained data
    ///
    /// # Safety
    ///
    /// The following two conditions need to be met:
    /// - [Self::init()] has been called to initialize the memory
    /// - Need to manually enforce borrowing rules. Only one mutable reference can exist at a time
    unsafe fn get_mut() -> &'static mut Self {
        unsafe { Self::addr().as_mut().unwrap() }
    }

    /// Gets reference to contained data
    ///
    /// # Safety
    ///
    /// The following two conditions need to be met:
    /// - [Self::init()] has been called to initialize the memory
    /// - Need to manually enforce borrowing rules.
    unsafe fn get() -> &'static Self {
        unsafe { Self::addr().as_ref().unwrap() }
    }
}
