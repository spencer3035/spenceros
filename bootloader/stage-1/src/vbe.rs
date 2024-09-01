mod vbe_impl;

#[derive(Debug, Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    const fn white() -> Self {
        Color {
            r: 0xff,
            g: 0xff,
            b: 0xff,
        }
    }

    const fn red() -> Self {
        Color {
            r: 0xff,
            g: 0,
            b: 0,
        }
    }

    const fn green() -> Self {
        Color {
            r: 0,
            g: 0xff,
            b: 0,
        }
    }

    const fn blue() -> Self {
        Color {
            r: 0,
            g: 0,
            b: 0xff,
        }
    }
}

pub struct Screen {
    width: u16,
    height: u16,
    depth: u8,
    line_bytes: u16,
    framebuffer: *mut u8,
    font: &'static [u8; 0x1000],
}

impl Screen {
    pub fn print_char(&self, c: u8) {
        let offset = c as usize * 16;
        for ii in 0..16 {
            let mut mask = self.font[offset + ii];
            let mut shift = 0;
            while mask != 0 {
                if mask & 1 != 0 {
                    self.set_pixel(8 - shift, ii as u16, &Color::white());
                }
                shift += 1;
                mask >>= 1;
            }
        }
    }

    #[inline]
    fn get_pixel_address(&self, x: u16, y: u16) -> *mut u8 {
        let y_offset = y as usize * self.line_bytes as usize;
        let x_offset = x as usize * (self.depth as usize / 8);
        let offset = y_offset + x_offset;

        unsafe { self.framebuffer.add(offset) }
    }

    /// Sets the given pixel a color, panics if x or y is out of range
    pub fn set_pixel(&self, x: u16, y: u16, color: &Color) {
        if x >= self.width || y > self.height {
            panic!("requested pixel out of range: ({x},{y})");
        }

        let addr = self.get_pixel_address(x, y);
        unsafe {
            match self.depth {
                24 => {
                    addr.add(0).write(color.b);
                    addr.add(1).write(color.g);
                    addr.add(2).write(color.r);
                }
                // TODO: Implement 32 bpp and 16 bpp modes
                n => panic!("{n} bits per pixel not supported"),
            }
        }
    }
}

/// Enters the best fit VBE mode
///
/// SAFETY: Writes to static variables, can't be used accross threads
pub unsafe fn init_graphical() -> Screen {
    let screen = vbe_impl::init();
    screen.print_char(b'!');

    loop {}
}
