#![no_std]
#![no_main]

use core::arch::asm;
use core::fmt::Write;
use core::sync::atomic::{AtomicUsize, Ordering};

use core::panic::PanicInfo;

macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        write!(Writer::default(),$($arg)*).unwrap();
    };
}

macro_rules! println {
    () => {
        print!("\n");
    };
    ($($arg:tt)*) => {{
        print!($($arg)*);
        print!("\n");
    }};
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic : {info}");
    loop {
        unsafe { asm!("hlt") }
    }
}

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) -> ! {
    clear_screen();

    for ii in 0usize..25 {
        println!("bruh, we have finally done it {ii:x}");
    }
    loop {}
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            if c.is_ascii() {
                self.print_char(c as u8);
            } else {
                self.print_char(b'?');
            }
        }

        Ok(())
    }
}

const TEXT_ADDRESS: *mut u16 = 0xB8000 as *mut u16;
const MAX_LINES: usize = 25;
const MAX_COLUMNS: usize = 80;
const BUFFER_SIZE: usize = MAX_LINES * MAX_COLUMNS;
static WRITE_INDEX: AtomicUsize = AtomicUsize::new(0);

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
}

fn write_char_row_col_color(c: u8, row: usize, col: usize, color: &TextColor) {
    let value = color.get_u16(c);
    let offset = col + MAX_COLUMNS * row;
    unsafe {
        TEXT_ADDRESS.add(offset).write(value);
    }
}

impl Writer {
    fn new(color: TextColor) -> Self {
        Writer { color }
    }
    /// Prints a single character to screen
    fn print_char(&mut self, c: u8) {
        let mut index = WRITE_INDEX.load(Ordering::Acquire);
        if c != b'\n' {
            let col = index % MAX_COLUMNS;
            let line = index / MAX_COLUMNS;
            write_char_row_col_color(c, line, col, &self.color);
            index += 1;
        } else {
            // Move to beginning of next line
            index = index - (index % MAX_COLUMNS) + MAX_COLUMNS;
        }

        if index >= BUFFER_SIZE {
            self.scroll();
            index = MAX_COLUMNS * (MAX_LINES - 1);
        }

        WRITE_INDEX.store(index, Ordering::Release);
    }

    /// Scrolls the lines up by one and sets cursor to beginning of last line
    fn scroll(&mut self) {
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
}
