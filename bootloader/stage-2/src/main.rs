#![no_std]
#![no_main]

use common::*;

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) -> ! {
    let mut writer = Writer::default();
    writer.print_string(b"We have finally done it");
    writer.flush();
    loop {}
}

const TEXT_ADDRESS: *mut u16 = 0xB8000 as *mut u16;
const MAX_LINES: u16 = 25;
const MAX_COLUMNS: u16 = 80;
const BUFFER_SIZE: usize = (MAX_COLUMNS * MAX_LINES) as usize;

#[allow(dead_code)]
#[repr(u8)]
enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

impl Color {
    #[inline]
    fn val(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

struct TextColor {
    fg: Color,
    bg: Color,
}

impl Default for TextColor {
    fn default() -> Self {
        TextColor {
            fg: Color::White,
            bg: Color::Black,
        }
    }
}

impl TextColor {
    fn get_u16(&self, char: u8) -> u16 {
        (self.color() as u16) << 8 | (char as u16)
    }
    fn color(&self) -> u8 {
        self.fg.val() | self.bg.val() << 4
    }
    fn new(foreground: Color, background: Color) -> Self {
        TextColor {
            fg: foreground,
            bg: background,
        }
    }
}

struct Writer {
    color: TextColor,
    index: usize,
    buffer: [u16; BUFFER_SIZE],
}

impl Default for Writer {
    fn default() -> Self {
        Writer {
            color: TextColor::default(),
            index: 0,
            buffer: [0; BUFFER_SIZE],
        }
    }
}

fn write_char_row_col_color(c: u8, row: u16, col: u16, color: &TextColor) {
    let value = color.get_u16(c);
    let offset = col + MAX_COLUMNS * row;
    unsafe {
        TEXT_ADDRESS.add(offset as usize).write(value);
    }
}

impl Writer {
    fn flush(&self) {
        for ii in 0..BUFFER_SIZE {
            unsafe {
                TEXT_ADDRESS.add(ii).write(self.buffer[ii]);
            }
        }
    }
    fn print_char(&mut self, c: u8) {
        //write_char_row_col_color(c, self.row, self.col, &self.color);
        if c != b'\n' {
            self.buffer[self.index] = self.color.get_u16(c);
            self.index += 1;
        } else {
            self.index = self.index - (self.index % MAX_COLUMNS as usize) + MAX_COLUMNS as usize;
        }

        if self.index >= BUFFER_SIZE {
            // TODO: Scroll
            write_char_row_col_color(b'P', 0, 0, &TextColor::new(Color::Red, Color::White));
            self.buffer.rotate_left(MAX_COLUMNS as usize);
            for ii in 0..MAX_COLUMNS {
                let ii: usize = ((MAX_LINES - 1) * MAX_COLUMNS + ii).into();
                self.buffer[ii] = self.color.get_u16(b' ');
            }
        }
    }

    fn print_string(&mut self, str: &[u8]) {
        for (ii, c) in str.iter().enumerate() {
            self.print_char(*c);
        }
    }

    fn print_hex(&mut self, mut val: u16) {
        let num_hexits = 4;
        self.print_char(b'0');
        self.print_char(b'x');
        for _ in 0..num_hexits {
            let high_hexit = ((val & 0xf000) >> 12) as u8;
            let to_print = if high_hexit <= 9 {
                high_hexit + b'0'
            } else {
                high_hexit + b'a'
            };

            val <<= 4;
            self.print_char(to_print);
        }
    }
}
