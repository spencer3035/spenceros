use core::arch::asm;

use common::println_vbe;
use proc_macros::define_keycodes;

pub fn next_scancode() -> u8 {
    while !has_scancode() {}

    let mut scancode: u8;
    unsafe {
        asm!(
            // Reads from the data register to get scancode
            "in al, 0x60",
             out("al") scancode,
        );
    };

    scancode
}

pub fn has_scancode() -> bool {
    let status: u8;
    unsafe {
        // Reads from the PS/2 controller status register
        asm!(
            "in al, 0x64",
             out("al") status,
        );
    };
    status & 1 != 0
}

#[derive(Debug)]
struct KeyEvent {
    code: KeyCode,
    is_down: bool,
}

fn wait_keycode() -> Option<KeyEvent> {
    get_next_keycode().ok()
}

pub fn wait_keypress() -> char {
    loop {
        let Some(kc) = wait_keycode() else {
            return '?';
        };

        if !kc.is_down {
            return kc.code.to_char().unwrap_or('?');
        }
    }
}

fn get_next_keycode() -> Result<KeyEvent, ()> {
    let code = next_scancode();
    let index = 0;
    get_next_keycode_impl(code, index)
}

fn get_next_keycode_impl(code: u8, index: u8) -> Result<KeyEvent, ()> {
    if has_next_scancode(code, index) {
        let code = next_scancode();
        get_next_keycode_impl(code, index + 1)
    } else {
        match keycode_from_index_and_code(code, index) {
            Some(val) => Ok(val),
            None => {
                println_vbe!("Didn't have mapping for code 0x{code:X} at depth {index}");
                Err(())
            }
        }
    }
}

fn has_next_scancode(code: u8, index: u8) -> bool {
    // has_next_scancode_impl(code, index)
    KeyCode::has_next(code, index)
}

fn keycode_from_index_and_code(code: u8, index: u8) -> Option<KeyEvent> {
    // keycode_from_index_and_code_impl(code, index)
    KeyCode::get_event(code, index)
}

macro_rules! def_keycodes {
    (
        $(
            ([$($code:literal),*], $name:ident, $dir:tt, $char:tt)
        ),*
    $(,)?
    ) => {
        fn has_next_scancode_impl(code: u8, index: u8) -> bool {
            false
            $(
                 // || ((count!($($code)*)) - 1 == index && code == last!($($code)*))
                 || ((count!($($code)*)) - 1 == index && code == last!($($code)*))
            )*
        }
        fn keycode_from_index_and_code_impl(code: u8, index: u8) -> Option<KeyEvent> {
            if false {
                None
            }
            $(
            else if ((count!($($code)*)) - 1 == index && code == last!($($code)*)) {
                Some(
                    KeyEvent {
                        code : KeyCode::$name,
                        is_down: def_keycodes!(@dir $dir)
                    })
            }
            )*
            else {
                None
            }
        }

        #[derive(Debug)]
        enum KeyCode {
            $($name ,)*
        }
        impl KeyCode {
            fn to_char(&self) -> Option<char> {
                match self {
                    $(KeyCode::$name => def_keycodes!(@to_char $char),)*
                }
            }
        }
    };
    (@matches $index:ident $code:ident $([$($first:literal, $rest:literal),*]),*) => {
        false $(
            ||
            )*
    };
    (@dir Down) => {true};
    (@dir Up) => {false};
    ( @to_char $ch:literal ) => {
        Some($ch)
    };
    ( @to_char None ) => {
        None
    };
}

macro_rules! count {
    () => (0u8);
    ($x:tt $($xs:tt)* ) => (1u8 + count!($($xs)*));
}

macro_rules! last {
    ($x:tt) => {
        $x
    };
    ($x:tt $($xs:tt)* ) => {
        last!($($xs)*)
    };
}

macro_rules! second_last_eq {
    () => { false };
    ($x:tt) => { false };
    ($val:literal, $x:tt $y:tt) => { $val == $x };
    ($val:literal, $x:tt $($xs:tt)* ) => {
        last!($val, $($xs)*)
    };
}

// def_keycodes!(
//     ([0x01], Kc1, Down, '2'),
//     ([0x02], Kc2, Down, '2'),
//     ([0x11], Kc1, Up, '1'),
//     ([0x12], Kc2, Up, '2'),
//     ([0xE0, 0x01], KcUp, Up, None),
//     ([0xE0, 0x11], KcUp, Down, None),
// );

// ( $code:literal, $name:ident, $char:literal) => {};
// ( @inner $code:literal, $name:ident) => {};

// def_keycodes!(
define_keycodes!(
    // Regular keycodes
    ([0x01], KcEsc, Up, None),
    ([0x02], Kc1, Up, '1'),
    ([0x03], Kc2, Up, '2'),
    ([0x04], Kc3, Up, '3'),
    ([0x05], Kc4, Up, '4'),
    ([0x06], Kc5, Up, '5'),
    ([0x07], Kc6, Up, '6'),
    ([0x08], Kc7, Up, '7'),
    ([0x09], Kc8, Up, '8'),
    ([0x0A], Kc9, Up, '9'),
    ([0x0B], Kc0, Up, '0'),
    ([0x0C], KcMinus, Up, '-'),
    ([0x0D], KcEquals, Up, '='),
    ([0x0E], KcBackspace, Up, None),
    ([0x0F], KcTab, Up, None),
    ([0x10], KcQ, Up, 'Q'),
    ([0x11], KcW, Up, 'W'),
    ([0x12], KcE, Up, 'E'),
    ([0x13], KcR, Up, 'R'),
    ([0x14], KcT, Up, 'T'),
    ([0x15], KcY, Up, 'Y'),
    ([0x16], KcU, Up, 'U'),
    ([0x17], KcI, Up, 'I'),
    ([0x18], KcO, Up, 'O'),
    ([0x19], KcP, Up, 'P'),
    ([0x1A], KcOpenSquare, Up, '['),
    ([0x1B], KcCloseSquare, Up, ']'),
    ([0x1C], KcEnter, Up, None),
    ([0x1D], KcLeftControl, Up, None),
    ([0x1E], KcA, Up, 'A'),
    ([0x1F], KcS, Up, 'S'),
    ([0x20], KcD, Up, 'D'),
    ([0x21], KcF, Up, 'F'),
    ([0x22], KcG, Up, 'G'),
    ([0x23], KcH, Up, 'H'),
    ([0x24], KcJ, Up, 'J'),
    ([0x25], KcK, Up, 'K'),
    ([0x26], KcL, Up, 'L'),
    ([0x27], KcSemiColon, Up, ';'),
    ([0x28], KcSingleQuote, Up, '\''),
    ([0x29], KcTick, Up, '`'),
    ([0x2A], KcLeftShift, Up, None),
    ([0x2B], KcBackSlash, Up, '\\'),
    ([0x2C], KcZ, Up, 'Z'),
    ([0x2D], KcX, Up, 'X'),
    ([0x2E], KcC, Up, 'C'),
    ([0x2F], KcV, Up, 'V'),
    ([0x30], KcB, Up, 'B'),
    ([0x31], KcN, Up, 'N'),
    ([0x32], KcM, Up, 'M'),
    ([0x33], KcComma, Up, ','),
    ([0x34], KcPeriod, Up, '.'),
    ([0x35], KcForwardSlash, Up, '/'),
    ([0x36], KcRightShift, Up, None),
    ([0x37], KcKpAst, Up, '*'),
    ([0x38], KcLeftAlt, Up, None),
    ([0x39], KcSpace, Up, None),
    ([0x3A], KcCapsLock, Up, None),
    ([0x3B], KcF1, Up, None),
    ([0x3C], KcF2, Up, None),
    ([0x3D], KcF3, Up, None),
    ([0x3E], KcF4, Up, None),
    ([0x3F], KcF5, Up, None),
    ([0x40], KcF6, Up, None),
    ([0x41], KcF7, Up, None),
    ([0x42], KcF8, Up, None),
    ([0x43], KcF9, Up, None),
    ([0x44], KcF10, Up, None),
    ([0x45], KcNumberLock, Up, None),
    ([0x46], KcScrollLock, Up, None),
    ([0x47], KcNp7, Up, '7'),
    ([0x48], KcNp8, Up, '8'),
    ([0x49], KcNp9, Up, '9'),
    ([0x4A], KcNpMinus, Up, '-'),
    ([0x4B], KcNp4, Up, '4'),
    ([0x4C], KcNp5, Up, '5'),
    ([0x4D], KcNp6, Up, '6'),
    ([0x4E], KcNpPlus, Up, '+'),
    ([0x4F], KcNp1, Up, '1'),
    ([0x50], KcNp2, Up, '2'),
    ([0x51], KcNp3, Up, '3'),
    ([0x52], KcNp0, Up, '0'),
    ([0x53], KcNpPeriod, Up, '.'),
    ([0x57], KcF11, Up, None),
    ([0x58], KcF12, Up, None),
    // Tuple keycodes
    ([0xE0, 0x10], KcMultiMediaTrackPrevious, Up, None),
    ([0xE0, 0x19], KcMultiMediaTrackNext, Up, None),
    ([0xE0, 0x1C], KcKpEnter, Up, None),
    ([0xE0, 0x1D], KcRightCtrl, Up, None),
    ([0xE0, 0x20], KcMultiMediaMute, Up, None),
    ([0xE0, 0x21], KcMultiMediaCalculator, Up, None),
    ([0xE0, 0x22], KcMultiMediaPlay, Up, None),
    ([0xE0, 0x24], KcMultiMediaStop, Up, None),
    ([0xE0, 0x2E], KcMultiMediaVolumeDown, Up, None),
    ([0xE0, 0x30], KcMultiMediaVolumeUp, Up, None),
    ([0xE0, 0x32], KcMultiMediaWwwHome, Up, None),
    ([0xE0, 0x35], KcKpForwardSlash, Up, '/'),
    ([0xE0, 0x38], KcRightAlt, Up, None),
    ([0xE0, 0x47], KcHome, Up, None),
    ([0xE0, 0x48], KcUp, Up, None),
    ([0xE0, 0x49], KcPageUp, Up, None),
    ([0xE0, 0x4B], KcLeft, Up, None),
    ([0xE0, 0x4D], KcRight, Up, None),
    ([0xE0, 0x4F], KcEnd, Up, None),
    ([0xE0, 0x50], KcDown, Up, None),
    ([0xE0, 0x51], KcPageDown, Up, None),
    ([0xE0, 0x52], KcInsert, Up, None),
    ([0xE0, 0x53], KcDelete, Up, None),
    ([0xE0, 0x5B], KcLeftGui, Up, None),
    ([0xE0, 0x5C], KcRightGui, Up, None),
    ([0xE0, 0x5D], KcApps, Up, None),
    ([0xE0, 0x5E], KcAcpiPower, Up, None),
    ([0xE0, 0x5F], KcAcpiSleep, Up, None),
    ([0xE0, 0x63], KcAcpiWake, Up, None),
    ([0xE0, 0x65], KcMultiMediaWwwSearch, Up, None),
    ([0xE0, 0x66], KcMultiMediaWwwFavorites, Up, None),
    ([0xE0, 0x67], KcMultiMediaWwwRefresh, Up, None),
    ([0xE0, 0x68], KcMultiMediaWwwStop, Up, None),
    ([0xE0, 0x69], KcMultiMediaWwwForward, Up, None),
    ([0xE0, 0x6A], KcMultiMediaWwwBack, Up, None),
    ([0xE0, 0x6B], KcMultiMediaMyComputer, Up, None),
);
