mod vbe_impl;

/// Enters the best fit VBE mode
///
/// SAFETY: Writes to static variables, can't be used accross threads
pub fn init_graphical() {
    vbe_impl::init();
}
