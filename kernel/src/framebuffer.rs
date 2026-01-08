use core::fmt::Write;

use bootloader_api::{BootInfo, info::FrameBufferInfo};

const CHAR_WIDTH: u16 = 8;
const CHAR_HEIGHT: u16 = 16;

#[derive(Debug)]
pub enum FrameBufferError {
    #[allow(dead_code)]
    NotEnoughBytesPerPixel(u8),
    NoFramebufferFound,
}

pub struct FrameBufferDisplay {
    pub char_index_x: usize,
    pub char_index_y: usize,
    pub info: FrameBufferInfo,
    pub buf: &'static mut [u8],
}

impl Write for FrameBufferDisplay {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char_impl(c)?;
        }
        Ok(())
    }
}

impl FrameBufferDisplay {
    pub fn new(boot_info: &'static mut BootInfo) -> Result<Self, FrameBufferError> {
        let framebuffer = boot_info
            .framebuffer
            .as_mut()
            .ok_or(FrameBufferError::NoFramebufferFound)?;
        let info = framebuffer.info();
        let buf = framebuffer.buffer_mut();
        let fb = Self {
            info,
            buf,
            char_index_x: 0,
            char_index_y: 0,
        };
        fb.check()?;
        Ok(fb)
    }

    pub fn backspace(&mut self) -> core::fmt::Result {
        if self.char_index_x > 0 {
            self.char_index_x -= 1;
        } else if self.char_index_y > 0 {
            self.char_index_y -= 1;
            let chars_per_col = self.width() / CHAR_WIDTH;
            self.char_index_x = chars_per_col as usize - 1;
        } else {
            // At beginning of screen
            return Ok(());
        }
        self.clear_char(self.char_index_x as u16, self.char_index_y as u16)?;

        Ok(())
    }

    fn write_char_impl(&mut self, ch: char) -> core::fmt::Result {
        match ch {
            '\n' => {
                self.char_index_y += 1;
                self.char_index_x = 0;
            }
            '\r' => {
                self.char_index_x = 0;
                return Ok(());
            }
            c => {
                let c = if c.is_ascii() { c as u8 } else { 137 };
                self.set_char(self.char_index_x as u16, self.char_index_y as u16, c)?;
                self.char_index_x += 1;
            }
        }

        let chars_per_col = self.width() / CHAR_WIDTH;
        let chars_per_row = self.height() / CHAR_HEIGHT;

        if self.char_index_x >= chars_per_col as usize {
            self.char_index_x = 0;
            self.char_index_y += 1;
        }

        if self.char_index_y >= chars_per_row as usize {
            self.shift_up(CHAR_HEIGHT);
            self.char_index_y = chars_per_row as usize - 1;
        }

        Ok(())
    }

    fn check(&self) -> Result<(), FrameBufferError> {
        match self.info.pixel_format {
            bootloader_api::info::PixelFormat::Rgb => {
                if self.info.bytes_per_pixel < 3 {
                    return Err(FrameBufferError::NotEnoughBytesPerPixel(
                        self.info.bytes_per_pixel as u8,
                    ));
                }
            }
            bootloader_api::info::PixelFormat::Bgr => {
                if self.info.bytes_per_pixel < 3 {
                    return Err(FrameBufferError::NotEnoughBytesPerPixel(
                        self.info.bytes_per_pixel as u8,
                    ));
                }
            }
            bootloader_api::info::PixelFormat::U8 => todo!(),
            bootloader_api::info::PixelFormat::Unknown {
                red_position: _,
                green_position: _,
                blue_position: _,
            } => todo!(),
            _ => todo!(),
        }

        Ok(())
    }
    fn shift_up_impl(&mut self, rows: u16) {
        let bytes_per_row = self.info.bytes_per_pixel * self.width() as usize;
        for row in 0..(self.height().saturating_sub(rows)) {
            let dst_offset = row as usize * self.info.stride * self.info.bytes_per_pixel;
            let src_offset = (row + rows) as usize * self.info.stride * self.info.bytes_per_pixel;

            self.buf.copy_within(src_offset..bytes_per_row, dst_offset);
        }

        // Set last `rows` rows to black
        for ii in 0..rows.min(self.height()) {
            let dst_offset = self.info.stride
                * self.info.bytes_per_pixel
                * self.height().saturating_sub(ii).saturating_sub(1) as usize;
            self.buf[dst_offset..].fill(0);
        }
    }

    fn get_pixel_address(&mut self, x: u16, y: u16) -> &mut [u8] {
        let pixel_offset = y as usize * self.info.stride + x as usize;
        let byte_offset = pixel_offset * self.info.bytes_per_pixel;
        let end = byte_offset + self.info.bytes_per_pixel;
        &mut self.buf[byte_offset..end]
    }

    /// Sets the given pixel a color, returns false if pixel is out of range
    fn set_pixel_impl(&mut self, x: u16, y: u16, color: &Color) -> core::fmt::Result {
        if x >= self.width() || y >= self.height() {
            // panic!("bad pixel position {x}, {y}");
            //return false;
            return Err(core::fmt::Error);
        }

        match self.info.pixel_format {
            bootloader_api::info::PixelFormat::Rgb => {
                let pixel = self.get_pixel_address(x, y);
                // TODO: Check mask
                pixel[0] = color.r;
                pixel[1] = color.g;
                pixel[2] = color.b;
            }
            bootloader_api::info::PixelFormat::Bgr => {
                let pixel = self.get_pixel_address(x, y);
                pixel[0] = color.b;
                pixel[1] = color.g;
                pixel[2] = color.r;
            }
            bootloader_api::info::PixelFormat::U8 => todo!(),
            bootloader_api::info::PixelFormat::Unknown {
                red_position: _,
                green_position: _,
                blue_position: _,
            } => todo!(),
            _ => todo!(),
        }

        Ok(())
    }

    /// Sets the given pixel a color, returns false if pixel is out of range
    fn clear_impl(&mut self) {
        let bytes_per_line = self.width() as usize * self.info.bytes_per_pixel;

        let mut ii = 0;
        for _jj in 0..self.height() {
            let end = ii + bytes_per_line;
            self.buf[ii..end].fill(0);
            ii += self.info.stride * self.info.bytes_per_pixel;
        }
    }
}

pub trait FrameBuffer {
    /// Gets number of pixels wide the screen is
    fn width(&self) -> u16;
    /// Gets number of pixels high the screen is
    fn height(&self) -> u16;
    /// Sets the given pixel the given color
    fn set_pixel(&mut self, x: u16, y: u16, c: &Color) -> core::fmt::Result;
    /// Clears the screen (sets to black)
    fn clear(&mut self);
    /// Gets font bitmap
    fn font(&self) -> &'static [u8; 0x1000];
    /// Shifts up rows by given number of pixels
    fn shift_up(&mut self, rows: u16);
    /// Sets a box given by two corners to the given color
    fn set_box(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        c: &Color,
    ) -> Result<(), core::fmt::Error> {
        for xx in x..(x + width) {
            for yy in y..(y + height) {
                self.set_pixel(xx, yy, c)?;
            }
        }

        Ok(())
    }
    /// Sets the characer at the given position (in units of characters)
    fn set_char(&mut self, x: u16, y: u16, c: u8) -> Result<(), core::fmt::Error> {
        if (x + 1) * CHAR_WIDTH > self.width() || (y + 1) * CHAR_HEIGHT > self.height() {
            // panic!("Bad char position {x},{y}");
            return Err(core::fmt::Error);
        }

        let offset = c as usize * 16;
        for ii in 0..16 {
            let mut mask = self.font()[offset + ii];
            let mut shift = 0;
            while mask != 0 {
                if mask & 1 != 0 {
                    let x_px = CHAR_WIDTH * x + 7 - shift;
                    let y_px = CHAR_HEIGHT * y + ii as u16;
                    self.set_pixel(x_px, y_px, &Color::WHITE)?;
                }
                shift += 1;
                mask >>= 1;
            }
        }

        Ok(())
    }
    fn clear_char(&mut self, x: u16, y: u16) -> core::fmt::Result {
        if (x + 1) * CHAR_WIDTH > self.width() || (y + 1) * CHAR_HEIGHT > self.height() {
            // panic!("Bad char position {x},{y}");
            return Err(core::fmt::Error);
        }

        let x_px = CHAR_WIDTH * x;
        let y_px = CHAR_HEIGHT * y;
        self.set_box(x_px, y_px, CHAR_WIDTH, CHAR_HEIGHT, &Color::BLACK)
    }
}

impl FrameBuffer for FrameBufferDisplay {
    fn width(&self) -> u16 {
        self.info.width as u16
    }
    fn height(&self) -> u16 {
        self.info.height as u16
    }
    fn set_pixel(&mut self, x: u16, y: u16, c: &Color) -> core::fmt::Result {
        self.set_pixel_impl(x, y, c)
    }
    fn clear(&mut self) {
        self.clear_impl()
    }
    fn font(&self) -> &'static [u8; 0x1000] {
        &crate::font::DECO_8X16_FONT
    }
    fn shift_up(&mut self, rows: u16) {
        self.shift_up_impl(rows);
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
