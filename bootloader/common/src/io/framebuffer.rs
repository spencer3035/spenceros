use core::fmt::Write;

use crate::{
    config::FONT,
    static_items::static_variable::StaticVariable,
    static_items::vbe_display::{Font, VbeDisplayInfo},
};

#[macro_export]
macro_rules! println_vbe {
    ($($args:tt)*) => {
        if $crate::io::framebuffer::VbeDisplay::is_init() {
            use core::fmt::Write as _;
            if let Err(e) = writeln!($crate::io::framebuffer::VbeDisplay, $($args)*) {
                // Fall back on bios printing. We want to avoid potential double panics
                panic!("write error : {e}");
            }
        } else {
            panic!("Screen is not init");
        }
    };
}

#[macro_export]
macro_rules! print_vbe {
    ($($args:tt)*) => {
        if $crate::io::framebuffer::VbeDisplay::is_init() {
            use core::fmt::Write as _;
            if let Err(e) = write!($crate::io::framebuffer::VbeDisplay, $($args)*) {
                // Fall back on bios printing. We want to avoid potential double panics
                panic!("write error : {e}");
            }
        } else {
            panic!("Screen is not init");
        }
    };
}

const CHAR_WIDTH: u16 = 8;
const CHAR_HEIGHT: u16 = 16;

pub trait FrameBuffer {
    /// Gets number of pixels wide the screen is
    fn width(&self) -> u16;
    /// Gets number of pixels high the screen is
    fn height(&self) -> u16;
    /// Sets the given pixel the given color
    fn set_pixel(&self, x: u16, y: u16, c: &Color) -> bool;
    /// Clears the screen (sets to black)
    fn clear(&self);
    /// Gets font bitmap
    fn font(&self) -> Option<&'static Font>;
    /// Shifts up rows by given number of pixels
    fn shift_up(&self, rows: u16);
    /// Sets the characer at the given position (in units of characters
    fn set_char(&self, x: u16, y: u16, c: u8) -> bool {
        if (x + 1) * CHAR_WIDTH > self.width() || (y + 1) * CHAR_HEIGHT > self.height() {
            panic!("Bad char position {x},{y}");
            //return false;
        }

        let font = match self.font() {
            Some(f) => f,
            None => panic!("No font set"),
            //return false,
        };

        let offset = c as usize * 16;
        for ii in 0..16 {
            let mut mask = font[offset + ii];
            let mut shift = 0;
            while mask != 0 {
                if mask & 1 != 0 {
                    let x_px = CHAR_WIDTH * x + 7 - shift;
                    let y_px = CHAR_HEIGHT * y + ii as u16;
                    self.set_pixel(x_px, y_px, &Color::WHITE);
                }
                shift += 1;
                mask >>= 1;
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub const RED: Color = Color {
        r: 0xff,
        g: 0,
        b: 0,
    };

    pub const GREEN: Color = Color {
        r: 0,
        g: 0xff,
        b: 0,
    };

    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 0xff,
    };

    pub const WHITE: Color = Color {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    };

    pub const BLACK: Color = Color {
        r: 0x0,
        g: 0x0,
        b: 0x0,
    };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

impl FrameBuffer for FramebufferInfo {
    fn width(&self) -> u16 {
        self.width
    }
    fn height(&self) -> u16 {
        self.height
    }
    fn set_pixel(&self, x: u16, y: u16, c: &Color) -> bool {
        self.set_pixel_impl(x, y, c)
    }
    fn clear(&self) {
        self.clear_impl()
    }
    fn font(&self) -> Option<&'static Font> {
        unsafe { FONT.as_ref() }
    }
    fn shift_up(&self, rows: u16) {
        self.shift_up_impl(rows);
    }
}
#[derive(Debug, Clone)]
#[repr(C)]
pub struct FramebufferInfo {
    pub mode_id: u16,
    /// Number of bytes to get next horizontal row
    pub bytes_per_scan_line: u16,
    /// How many pixels wide the screen is
    pub width: u16,
    /// How many pixels high the screen is
    pub height: u16,
    /// How many bits per pixes, should be 4 or 6
    pub bits_per_pixel: u8,
    /// Start address of the framebuffer
    pub framebuffer: *mut u8,
    // TODO: Put color mask
}

/// Safety: We don't use threads, this is only needed because we store a pointer internally
unsafe impl Sync for FramebufferInfo {}

impl FramebufferInfo {
    /// Gets null, invalid frame buffer. Can be used for construction
    pub const fn null() -> FramebufferInfo {
        FramebufferInfo {
            mode_id: 0,
            bytes_per_scan_line: 0,
            width: 0,
            height: 0,
            bits_per_pixel: 0,
            framebuffer: core::ptr::null_mut(),
        }
    }
    /// Checks if the framebuffer is valid. Should be checked before returning/passing a
    /// framebuffer around
    #[must_use]
    pub fn is_valid(&self) -> bool {
        // TODO: Convert to error instead of bool
        // TODO: Add more checks
        self.bytes_per_scan_line != 0
            && self.width != 0
            && self.height != 0
            && self.bits_per_pixel != 0
            && self.framebuffer as usize != 0
    }

    fn shift_up_impl(&self, rows: u16) {
        let bytes_per_row = self.bits_per_pixel as usize * self.width() as usize / 8;
        for row in 0..(self.height().saturating_sub(rows)) {
            let dst_offset = row as usize * self.bytes_per_scan_line as usize;
            let src_offset = (row + rows) as usize * self.bytes_per_scan_line as usize;

            unsafe {
                let dst: *mut u8 = self.framebuffer.add(dst_offset);
                let src: *mut u8 = self.framebuffer.add(src_offset);
                core::ptr::copy_nonoverlapping(src, dst, bytes_per_row);
            }
        }

        // Set last `rows` rows to black
        for ii in 0..rows.min(self.height) {
            let dst_offset = self.bytes_per_scan_line as usize
                * self.height.saturating_sub(ii).saturating_sub(1) as usize;
            unsafe {
                let dst = self.framebuffer.add(dst_offset);
                let slice = core::slice::from_raw_parts_mut(dst, bytes_per_row);
                slice.fill(0);
            }
        }
    }

    #[inline]
    fn get_pixel_address(&self, x: u16, y: u16) -> *mut u8 {
        let y_offset = y as usize * self.bytes_per_scan_line as usize;
        let x_offset = x as usize * (self.bits_per_pixel as usize / 8);
        let offset = y_offset + x_offset;

        unsafe { self.framebuffer.add(offset) }
    }

    /// Sets the given pixel a color, returns false if pixel is out of range
    fn set_pixel_impl(&self, x: u16, y: u16, color: &Color) -> bool {
        if x >= self.width || y >= self.height {
            panic!("bad pixel position {x}, {y}");
            //return false;
        }

        let addr = self.get_pixel_address(x, y);
        unsafe {
            match self.bits_per_pixel {
                24 => {
                    // TODO: Check mask
                    addr.add(0).write(color.b);
                    addr.add(1).write(color.g);
                    addr.add(2).write(color.r);
                }
                // TODO: Implement 32 bpp and 16 bpp modes
                n => panic!("{n} bits per pixel not supported"),
            }
        }

        true
    }

    /// Sets the given pixel a color, returns false if pixel is out of range
    fn clear_impl(&self) {
        let mut addr = self.get_pixel_address(0, 0);

        let bytes_per_line = self.width() as usize * self.bits_per_pixel as usize / 8;

        for _jj in 0..self.height() {
            unsafe {
                let slice = core::slice::from_raw_parts_mut(addr, bytes_per_line);
                slice.fill(0);
                addr = addr.add(self.bytes_per_scan_line as usize);
            }
        }
    }
}

pub struct VbeDisplay;

// TODO: This current implementation is terrible in terms of safety. Both get_mut and get can
// probably overlap. We should figure out a better way to structure things
impl FrameBuffer for VbeDisplay {
    fn width(&self) -> u16 {
        unsafe { VbeDisplayInfo::get().framebuffer.width() }
    }
    fn height(&self) -> u16 {
        unsafe { VbeDisplayInfo::get().framebuffer.height() }
    }
    fn set_pixel(&self, x: u16, y: u16, c: &Color) -> bool {
        unsafe { VbeDisplayInfo::get().framebuffer.set_pixel(x, y, c) }
    }
    fn font(&self) -> Option<&'static Font> {
        unsafe { FONT.as_ref() }
    }
    fn shift_up(&self, rows: u16) {
        unsafe { VbeDisplayInfo::get().framebuffer.shift_up(rows) }
    }
    fn clear(&self) {
        unsafe { VbeDisplayInfo::get().framebuffer.clear() }
    }
}

impl Write for VbeDisplay {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char_impl(c);
        }
        Ok(())
    }
}

impl VbeDisplay {
    /// Init's the screen
    pub fn init() {
        // This seems to "wake up" the screen so later prints work. It may be worth investigating
        // why some prints do not display correctly without this
        unsafe {
            VbeDisplayInfo::get_mut().is_init = true;
        }
        VbeDisplay.clear();
    }
    pub fn is_init() -> bool {
        unsafe { VbeDisplayInfo::get().is_init }
    }
    pub fn reset(&self) {
        unsafe { VbeDisplayInfo::get_mut().char_index = 0 }
        // Set everything to dark gray
        for x in 0..VbeDisplay.width() {
            for y in 0..VbeDisplay.height() {
                VbeDisplay.set_pixel(x, y, &Color::new(0, 0, 0));
            }
        }
    }
    fn width_char(&self) -> u16 {
        VbeDisplay.width() / CHAR_WIDTH
    }
    fn height_char(&self) -> u16 {
        VbeDisplay.height() / CHAR_HEIGHT
    }
    fn write_char_impl(&self, c: char) {
        // TODO: This is super subject to race conditions if used across multiple threads
        let mut char_idx: u32 = unsafe { VbeDisplayInfo::get().char_index };
        if char_idx >= self.width_char() as u32 * self.height_char() as u32 {
            char_idx = (self.height_char() as u32 - 1) * self.width_char() as u32;
            self.shift_up(16);
        }

        if c == '\n' {
            char_idx += self.width_char() as u32 - char_idx % self.width_char() as u32;
        } else {
            let y = char_idx as u16 / self.width_char();
            let x = char_idx as u16 % self.width_char();
            if c.is_ascii() {
                VbeDisplay.set_char(x, y, c as u8);
            } else {
                VbeDisplay.set_char(x, y, b'?');
            }
            char_idx += 1;
        }

        unsafe { VbeDisplayInfo::get_mut().char_index = char_idx };
    }
}
