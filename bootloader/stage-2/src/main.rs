#![no_std]
#![no_main]

use common::*;

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) -> ! {
    clear_screen();

    let mut writer = Writer::default();
    for ii in 0usize..25 {
        writer.print_string(b"We have finally done it : ");
        writer.print_hex(ii as u16);
        writer.print_char(b'\n');
    }

    writer.print_string(b"here we go");
    loop {}
}

//impl core::fmt::Write for Writer {
//    fn write_str(&mut self, s: &str) -> core::fmt::Result {
//    }
//}

const TEXT_ADDRESS: *mut u16 = 0xB8000 as *mut u16;
const MAX_LINES: usize = 25;
const MAX_COLUMNS: usize = 80;
const BUFFER_SIZE: usize = MAX_LINES * MAX_COLUMNS;

fn clear_screen() {
    let value: u16 = (0x0f << 8) | b' ' as u16;
    for ii in 0..BUFFER_SIZE {
        unsafe {
            TEXT_ADDRESS.add(ii).write(value);
        }
    }
}

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

#[derive(Default)]
struct Writer {
    color: TextColor,
    index: usize,
}

fn write_char_row_col_color(c: u8, row: usize, col: usize, color: &TextColor) {
    let value = color.get_u16(c);
    let offset = col + MAX_COLUMNS * row;
    unsafe {
        TEXT_ADDRESS.add(offset).write(value);
    }
}

impl Writer {
    /// Prints a single character to screen
    fn print_char(&mut self, c: u8) {
        if c != b'\n' {
            let col = self.index % MAX_COLUMNS;
            let line = self.index / MAX_COLUMNS;
            write_char_row_col_color(c, line, col, &self.color);
            self.index += 1;
        } else {
            // Move to beginning of next line
            self.index = self.index - (self.index % MAX_COLUMNS) + MAX_COLUMNS;
        }

        if self.index >= BUFFER_SIZE {
            self.scroll();
        }
    }

    /// Scrolls the lines up by one and sets cursor to beginning of last line
    fn scroll(&mut self) {
        self.index = MAX_COLUMNS * (MAX_LINES - 1);

        for ii in 0..(BUFFER_SIZE - MAX_COLUMNS) {
            unsafe {
                TEXT_ADDRESS
                    .add(ii)
                    .write(TEXT_ADDRESS.add(ii + MAX_COLUMNS).read());
            }
        }

        (0..MAX_COLUMNS)
            .map(|x| x + BUFFER_SIZE - MAX_COLUMNS)
            .for_each(|ii| unsafe {
                TEXT_ADDRESS
                    .add(ii)
                    .write(TEXT_ADDRESS.add(ii + MAX_COLUMNS).read());
            });
    }

    fn print_string(&mut self, str: &[u8]) {
        for c in str.iter() {
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
                high_hexit - 10 + b'a'
            };

            val <<= 4;
            self.print_char(to_print);
        }
    }
}
