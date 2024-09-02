mod vbe_impl;
use common::io::framebuffer::Screen;

/// Enters the best fit VBE mode
///
/// SAFETY: Writes to static variables, can't be used accross threads
pub fn init_graphical() {
    // pixel size: 1024x720
    // char  size: 160x45
    let mode = vbe_impl::init();
    Screen::init(mode);
    get_monitor_info();
}

fn get_monitor_info() {
    println!("Getting monitor info");
    loop {}
}
