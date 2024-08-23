use core::sync::atomic::{AtomicUsize, Ordering};

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        #[allow(unused_imports)]
        use ::core::fmt::Write as _;
        write!($crate::protected_mode::io::Writer::default(),$($arg)*).unwrap();
    };
}

#[macro_export]
macro_rules! println {
    () => {
        #[allow(unused_imports)]
        use ::core::fmt::Write as _;
        print!("\n");
    };
    ($($arg:tt)*) => {
        #[allow(unused_imports)]
        use ::core::fmt::Write as _;
        print!($($arg)*);
        print!("\n");
    };
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

pub fn clear_screen() {
    let value: u16 = (0x0f << 8) | b' ' as u16;
    for ii in 0..BUFFER_SIZE {
        unsafe {
            TEXT_ADDRESS.add(ii).write(value);
        }
    }

    unsafe {
        WRITE_INDEX.store(0, Ordering::Relaxed);
    }
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
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

pub struct TextColor {
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
    /// Gets a new text color with given forground and background colors
    pub fn new(foreground: Color, background: Color) -> Self {
        TextColor {
            fg: foreground,
            bg: background,
        }
    }
    fn get_u16(&self, char: u8) -> u16 {
        (self.color() as u16) << 8 | (char as u16)
    }
    fn color(&self) -> u8 {
        self.fg.val() | self.bg.val() << 4
    }
}

#[derive(Default)]
pub struct Writer {
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
    /// Sets the color of the text
    #[allow(dead_code)]
    pub fn set_color(&mut self, color: TextColor) {
        self.color = color;
    }
    /// Prints a single character to screen
    fn print_char(&mut self, c: u8) {
        let mut index = WRITE_INDEX.load(Ordering::Acquire);

        if index >= BUFFER_SIZE {
            self.scroll();
            index = MAX_COLUMNS * (MAX_LINES - 1);
        }

        if c != b'\n' {
            let col = index % MAX_COLUMNS;
            let line = index / MAX_COLUMNS;
            write_char_row_col_color(c, line, col, &self.color);
            index += 1;
        } else {
            // Move to beginning of next line
            index = index - (index % MAX_COLUMNS) + MAX_COLUMNS;
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
