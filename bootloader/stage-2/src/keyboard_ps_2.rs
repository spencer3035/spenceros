use core::arch::asm;

use common::println_vbe;

pub fn wait_scancode() -> u8 {
    while !has_keypres() {}

    let mut scancode: u8;
    unsafe {
        asm!(
            "in al, 0x60",
             out("al") scancode,
        );
    };

    scancode
}

pub fn wait_keypress() -> char {
    let mut scancode = wait_scancode();
    // If the keypress is down/press or up/release
    if scancode == 0xE0 {
        // Get next scancode
        scancode = wait_scancode();
        // TODO: handle is_down here too. Try to clean up logic a bit
        return KeyCode::from_scancode_e0(scancode)
            .and_then(|kc| {
                println_vbe!("E0 {kc:?}");
                kc.to_char()
            })
            .unwrap_or('_');
    }
    let is_down = scancode & 0b1000_0000 == 0;
    if !is_down {
        // This is a key release, not a press
        wait_keypress()
    } else if let Some(kc) = KeyCode::from_scancode_1(scancode) {
        println_vbe!("{kc:?}");
        kc.to_char().unwrap_or('_')
    } else {
        '?'
    }
}

pub fn has_keypres() -> bool {
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

macro_rules! def_keycodes {
    (
        $( ($code:literal, $name:ident, $char:tt )  ),* ,
        $( {0xE0, $code_e0:literal, $name_e0:ident, $char_e0:tt }  ),* $(,)?
    ) => {
        #[derive(Debug)]
        enum KeyCode {
            $($name ,)*
            $($name_e0 ,)*
        }
        impl KeyCode {
            fn from_scancode_1(code: u8) -> Option<Self> {
                match code & 0b0111_1111 {
                    $($code => Some(KeyCode::$name),)*
                    _ => None,
                }
            }

            fn from_scancode_e0(code: u8) -> Option<Self> {
                match code & 0b0111_1111 {
                    $($code_e0 => Some(KeyCode::$name_e0),)*
                    _ => None,
                }
            }

            fn to_char(&self) -> Option<char> {
                match self {
                    $(KeyCode::$name => def_keycodes!(@to_char $char),)*
                    $(KeyCode::$name_e0 => def_keycodes!(@to_char $char_e0),)*
                }
            }
        }
    };
    ( @to_char $ch:literal ) => { Some($ch) };
    ( @to_char None ) => { None };
}

// ( $code:literal, $name:ident, $char:literal) => {};
// ( @inner $code:literal, $name:ident) => {};

def_keycodes!(
    // Regular keycodes
    (0x01, KcEsc, None),
    (0x02, Kc1, '1'),
    (0x03, Kc2, '2'),
    (0x04, Kc3, '3'),
    (0x05, Kc4, '4'),
    (0x06, Kc5, '5'),
    (0x07, Kc6, '6'),
    (0x08, Kc7, '7'),
    (0x09, Kc8, '8'),
    (0x0A, Kc9, '9'),
    (0x0B, Kc0, '0'),
    (0x0C, KcMinus, '-'),
    (0x0D, KcEquals, '='),
    (0x0E, KcBackspace, None),
    (0x0F, KcTab, None),
    (0x10, KcQ, 'Q'),
    (0x11, KcW, 'W'),
    (0x12, KcE, 'E'),
    (0x13, KcR, 'R'),
    (0x14, KcT, 'T'),
    (0x15, KcY, 'Y'),
    (0x16, KcU, 'U'),
    (0x17, KcI, 'I'),
    (0x18, KcO, 'O'),
    (0x19, KcP, 'P'),
    (0x1A, KcOpenSquare, '['),
    (0x1B, KcCloseSquare, ']'),
    (0x1C, KcEnter, None),
    (0x1D, KcLeftControl, None),
    (0x1E, KcA, 'A'),
    (0x1F, KcS, 'S'),
    (0x20, KcD, 'D'),
    (0x21, KcF, 'F'),
    (0x22, KcG, 'G'),
    (0x23, KcH, 'H'),
    (0x24, KcJ, 'J'),
    (0x25, KcK, 'K'),
    (0x26, KcL, 'L'),
    (0x27, KcSemiColon, ';'),
    (0x28, KcSingleQuote, '\''),
    (0x29, KcTick, '`'),
    (0x2A, KcLeftShift, None),
    (0x2B, KcBackSlash, '\\'),
    (0x2C, KcZ, 'Z'),
    (0x2D, KcX, 'X'),
    (0x2E, KcC, 'C'),
    (0x2F, KcV, 'V'),
    (0x30, KcB, 'B'),
    (0x31, KcN, 'N'),
    (0x32, KcM, 'M'),
    (0x33, KcComma, ','),
    (0x34, KcPeriod, '.'),
    (0x35, KcForwardSlash, '/'),
    (0x36, KcRightShift, None),
    (0x37, KcKpAst, '*'),
    (0x38, KcLeftAlt, None),
    (0x39, KcSpace, None),
    (0x3A, KcCapsLock, None),
    (0x3B, KcF1, None),
    (0x3C, KcF2, None),
    (0x3D, KcF3, None),
    (0x3E, KcF4, None),
    (0x3F, KcF5, None),
    (0x40, KcF6, None),
    (0x41, KcF7, None),
    (0x42, KcF8, None),
    (0x43, KcF9, None),
    (0x44, KcF10, None),
    (0x45, KcNumberLock, None),
    (0x46, KcScrollLock, None),
    (0x47, KcNp7, '7'),
    (0x48, KcNp8, '8'),
    (0x49, KcNp9, '9'),
    (0x4A, KcNpMinus, '-'),
    (0x4B, KcNp4, '4'),
    (0x4C, KcNp5, '5'),
    (0x4D, KcNp6, '6'),
    (0x4E, KcNpPlus, '+'),
    (0x4F, KcNp1, '1'),
    (0x50, KcNp2, '2'),
    (0x51, KcNp3, '3'),
    (0x52, KcNp0, '0'),
    (0x53, KcNpPeriod, '.'),
    (0x57, KcF11, None),
    (0x58, KcF12, None),
    // Tuple keycodes
    {0xE0, 0x10, KcMultiMediaTrackPrevious, None},
    {0xE0, 0x19, KcMultiMediaTrackNext, None},
    {0xE0, 0x1C, KcKpEnter, None},
    {0xE0, 0x1D, KcRightCtrl, None},
    {0xE0, 0x20, KcMultiMediaMute, None},
    {0xE0, 0x21, KcMultiMediaCalculator, None},
    {0xE0, 0x22, KcMultiMediaPlay, None},
    {0xE0, 0x24, KcMultiMediaStop, None},
    {0xE0, 0x2E, KcMultiMediaVolumeDown, None},
    {0xE0, 0x30, KcMultiMediaVolumeUp, None},
    {0xE0, 0x32, KcMultiMediaWwwHome, None},
    {0xE0, 0x35, KcKpForwardSlash, '/'},
    {0xE0, 0x38, KcRightAlt, None},
    {0xE0, 0x47, KcHome, None},
    {0xE0, 0x48, KcUp, None},
    {0xE0, 0x49, KcPageUp, None},
    {0xE0, 0x4B, KcLeft, None},
    {0xE0, 0x4D, KcRight, None},
    {0xE0, 0x4F, KcEnd, None},
    {0xE0, 0x50, KcDown, None},
    {0xE0, 0x51, KcPageDown, None},
    {0xE0, 0x52, KcInsert, None},
    {0xE0, 0x53, KcDelete, None},
    {0xE0, 0x5B, KcLeftGui, None},
    {0xE0, 0x5C, KcRightGui, None},
    {0xE0, 0x5D, KcApps, None},
    {0xE0, 0x5E, KcAcpiPower, None},
    {0xE0, 0x5F, KcAcpiSleep, None},
    {0xE0, 0x63, KcAcpiWake, None},
    {0xE0, 0x65, KcMultiMediaWwwSearch, None},
    {0xE0, 0x66, KcMultiMediaWwwFavorites, None},
    {0xE0, 0x67, KcMultiMediaWwwRefresh, None},
    {0xE0, 0x68, KcMultiMediaWwwStop, None},
    {0xE0, 0x69, KcMultiMediaWwwForward, None},
    {0xE0, 0x6A, KcMultiMediaWwwBack, None},
    {0xE0, 0x6B, KcMultiMediaMyComputer, None},
);
