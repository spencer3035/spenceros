use core::arch::asm;

/// Prints a single characetr to the screen
#[inline]
pub fn print_char(c: u8) {
    let ax = c as u16 | 0x0e00;
    unsafe {
        asm!(
            "int 0x10",
            in("ax") ax,
        );
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        #[allow(unused_imports)]
        use core::fmt::Write as _;
        write!($crate::real_mode::BiosWriter, $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        #[allow(unused_imports)]
        use core::fmt::Write as _;
        writeln!($crate::real_mode::BiosWriter, $($arg)*);
    }};
}

pub struct BiosWriter;

impl core::fmt::Write for BiosWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print_str(s);
        Ok(())
    }
}

fn print_str(s: &str) {
    for c in s.chars() {
        if c.is_ascii() {
            if c == '\n' {
                let ax = 0x0e00 | b'\r' as u16;
                unsafe { asm!("int 0x10", in("ax") ax) }
                let ax = 0x0e00 | b'\n' as u16;
                unsafe { asm!("int 0x10", in("ax") ax) }
            } else {
                print_char(c as u8);
            }
        } else {
            print_char(b'?')
        }
    }
}

/// Prints a value in hex, prepending 0x
pub fn print_hex32(mut num: u32) {
    print_char(b'0');
    print_char(b'x');
    let mut num_hexits = 0;
    loop {
        let hexit = num & 0x0F;
        unsafe {
            asm!("push {0:x}", in(reg) hexit);
        }
        num_hexits += 1;
        num = num >> 4;
        if num == 0 {
            break;
        }
    }

    while num_hexits > 0 {
        let hexit: i16;
        unsafe {
            asm!("pop {0:x}", out(reg) hexit);
        }
        let value = if hexit <= 9 {
            hexit as u8 + b'0'
        } else {
            hexit as u8 - 10 + b'a'
        };
        print_char(value);
        num_hexits -= 1;
    }
}

/// Prints a value in hex, prepending 0x
pub fn print_hex(mut num: u16) {
    print_char(b'0');
    print_char(b'x');
    let mut num_hexits = 0;
    loop {
        let hexit = num & 0x0F;
        unsafe {
            asm!("push {0:x}", in(reg) hexit);
        }
        num_hexits += 1;
        num = num >> 4;
        if num == 0 {
            break;
        }
    }

    while num_hexits > 0 {
        let hexit: i16;
        unsafe {
            asm!("pop {0:x}", out(reg) hexit);
        }
        let value = if hexit <= 9 {
            hexit as u8 + b'0'
        } else {
            hexit as u8 - 10 + b'a'
        };
        print_char(value);
        num_hexits -= 1;
    }
}

/// Prints a decimal value to the screen
pub fn print_dec(mut num: u16) {
    let mut num_digits = 0;

    loop {
        let digit: u16 = num % 10;
        unsafe {
            asm!("push {0:x}", in(reg) digit);
        }
        num_digits += 1;
        num /= 10;
        if num == 0 {
            break;
        }
    }

    while num_digits > 0 {
        let digit: i16;
        unsafe {
            asm!("pop {0:x}", out(reg) digit);
        }
        let value = digit as u8 + b'0';
        print_char(value);
        num_digits -= 1;
    }
}

/// Prints a slice of characters to screen with \c\r at the end
#[inline]
pub fn printline(chars: &[u8]) {
    print_chars(chars);
    print_chars(b"\r\n");
}

/// Prints a slice of characters to screen in BIOS
pub fn print_chars(chars: &[u8]) {
    for val in chars.iter() {
        print_char(*val);
    }
}

/// Prints '![char]' where [char] should be the top element on the stack when this is called
///
/// Should not be called with jump commands from assembly. Will not work unless called
pub fn fail(code: &[u8]) -> ! {
    print_chars(b"Fail: ");
    printline(code);
    hlt()
}

/// Prints '![char]' where [char] should be the top element on the stack when this is called
///
/// Should not be called with jump commands from assembly. Will not work unless called
#[no_mangle]
pub extern "C" fn fail_asm(code: &u8) -> ! {
    print_chars(b"Fail: ");
    print_char(*code);
    hlt()
}

/// Displays "Halt." and Halts CPU
pub fn hlt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
