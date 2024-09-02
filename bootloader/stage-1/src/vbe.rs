mod vbe_impl;
use core::{
    fmt::Write,
    sync::atomic::{AtomicBool, AtomicU16, AtomicUsize, Ordering},
};

use common::println_bios;
use vbe_impl::set_bitmap_font_from_bios;

static mut FRAME_BUFFER: Option<FramebufferInfo> = None;
static mut FONT: Option<[u8; 0x1000]> = None;

pub trait FrameBuffer {
    /// Gets number of pixels wide the screen is
    fn width(&self) -> u16;
    /// Gets number of pixels high the screen is
    fn height(&self) -> u16;
    /// Sets the given pixel the given color
    fn set_pixel(&self, x: u16, y: u16, c: &Color) -> bool;
    /// Gets font bitmap
    fn font(&self) -> Option<&'static [u8; 0x1000]>;
    /// Sets the characer at the given position (in units of characters
    fn set_char(&self, x: u16, y: u16, c: u8) -> bool {
        if (x + 1) * 8 > self.width() || (y + 1) * 16 > self.height() {
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
                    let x_px = 8 * x + 7 - shift;
                    let y_px = 16 * y + ii as u16;
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
    fn font(&self) -> Option<&'static [u8; 0x1000]> {
        unsafe { FONT.as_ref() }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FramebufferInfo {
    mode_id: u16,
    /// Number of bytes to get next horizontal row
    bytes_per_scan_line: u16,
    /// How many pixels wide the screen is
    width: u16,
    /// How many pixels high the screen is
    height: u16,
    /// How many bits per pixes, should be 4 or 6
    bits_per_pixel: u8,
    /// Start address of the framebuffer
    framebuffer: *mut u8,
    // TODO: Put color mask
}

impl FramebufferInfo {
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
}

static CHAR_INDEX: AtomicUsize = AtomicUsize::new(0);

pub struct Screen;

impl FrameBuffer for Screen {
    fn width(&self) -> u16 {
        unsafe { FRAME_BUFFER.as_ref().unwrap().width() }
    }
    fn height(&self) -> u16 {
        unsafe { FRAME_BUFFER.as_ref().unwrap().height() }
    }
    fn set_pixel(&self, x: u16, y: u16, c: &Color) -> bool {
        unsafe { FRAME_BUFFER.as_ref().unwrap().set_pixel(x, y, c) }
    }
    fn font(&self) -> Option<&'static [u8; 0x1000]> {
        unsafe { FONT.as_ref() }
    }
}

impl Write for Screen {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char_impl(c);
        }

        Ok(())
    }
}

impl Screen {
    pub fn reset(&self) {
        CHAR_INDEX.store(0, Ordering::Relaxed);
        // Set everything to dark gray
        for x in 0..Screen.width() {
            for y in 0..Screen.height() {
                Screen.set_pixel(x, y, &Color::new(0, 0, 0));
            }
        }
    }
    fn width_char(&self) -> u16 {
        Screen.width() / 8
    }
    fn height_char(&self) -> u16 {
        Screen.height() / 16
    }
    fn write_char_impl(&self, c: char) {
        let mut char_idx = CHAR_INDEX.load(Ordering::Acquire);
        if char_idx >= self.width_char() as usize * self.height_char() as usize {
            char_idx -= self.width_char() as usize;
            CHAR_INDEX.store(char_idx, Ordering::Release);
            panic!("Scrolling not implemented");
        }

        if c == '\n' {
            char_idx += self.width_char() as usize - char_idx % self.width_char() as usize;
        } else {
            let y = char_idx as u16 / self.width_char();
            let x = char_idx as u16 % self.width_char();
            if c.is_ascii() {
                Screen.set_char(x, y, c as u8);
            } else {
                Screen.set_char(x, y, b'?');
            }
            char_idx += 1;
        }

        CHAR_INDEX.store(char_idx, Ordering::Release);
    }
}

/// Enters the best fit VBE mode
///
/// SAFETY: Writes to static variables, can't be used accross threads
pub fn init_graphical() -> Screen {
    unsafe {
        FONT = Some([0; 0x1000]);
        set_bitmap_font_from_bios(FONT.as_mut().unwrap());
    }

    //for ii in 0..16 {
    //    unsafe {
    //        println_bios!("{:08b}", FONT.as_ref().unwrap()[b'?' as usize * 16 + ii]);
    //    }
    //}

    //loop {}

    let mode = vbe_impl::init();
    unsafe {
        FRAME_BUFFER = Some(mode);
    }

    // Set everything to dark gray
    for x in 0..Screen.width() {
        for y in 0..Screen.height() {
            Screen.set_pixel(x, y, &Color::new(20, 20, 20));
        }
    }

    // 1024x720
    for x in 0..Screen.width() {
        let y = 400;
        //let (width, height, depth) = (1280, 720, 24);
        Screen.set_pixel(x, y, &Color::new(0, 0xff, 0));
    }

    // 160x45
    //write!(Screen, "{}x{}", Screen.width_char(), Screen.height_char());
    //loop {}

    for ii in 0..(720 + 1) {
        let v = ii % 10;
        write!(Screen, "{v}12345678_").unwrap();
    }
    //write!(Screen, "Here is a long string\n with some new linese \n");
    loop {}
}
