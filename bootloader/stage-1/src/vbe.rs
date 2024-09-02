mod vbe_impl;
use common::framebuffer::Screen;

/// Enters the best fit VBE mode
///
/// SAFETY: Writes to static variables, can't be used accross threads
pub fn init_graphical() {
    let mode = vbe_impl::init();
    unsafe {
        Screen::init(mode);
    }

    // pixel size: 1024x720
    // char  size: 160x45
}
