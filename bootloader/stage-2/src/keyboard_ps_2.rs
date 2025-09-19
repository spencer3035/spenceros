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

        println_vbe!("{kc:?}");

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

define_keycodes!(
    // Simple keycodes down
    ([0x01], KcEsc, Down, None),
    ([0x02], Kc1, Down, '1'),
    ([0x03], Kc2, Down, '2'),
    ([0x04], Kc3, Down, '3'),
    ([0x05], Kc4, Down, '4'),
    ([0x06], Kc5, Down, '5'),
    ([0x07], Kc6, Down, '6'),
    ([0x08], Kc7, Down, '7'),
    ([0x09], Kc8, Down, '8'),
    ([0x0A], Kc9, Down, '9'),
    ([0x0B], Kc0, Down, '0'),
    ([0x0C], KcMinus, Down, '-'),
    ([0x0D], KcEquals, Down, '='),
    ([0x0E], KcBackspace, Down, None),
    ([0x0F], KcTab, Down, None),
    ([0x10], KcQ, Down, 'Q'),
    ([0x11], KcW, Down, 'W'),
    ([0x12], KcE, Down, 'E'),
    ([0x13], KcR, Down, 'R'),
    ([0x14], KcT, Down, 'T'),
    ([0x15], KcY, Down, 'Y'),
    ([0x16], KcU, Down, 'U'),
    ([0x17], KcI, Down, 'I'),
    ([0x18], KcO, Down, 'O'),
    ([0x19], KcP, Down, 'P'),
    ([0x1A], KcOpenSquare, Down, '['),
    ([0x1B], KcCloseSquare, Down, ']'),
    ([0x1C], KcEnter, Down, None),
    ([0x1D], KcLeftControl, Down, None),
    ([0x1E], KcA, Down, 'A'),
    ([0x1F], KcS, Down, 'S'),
    ([0x20], KcD, Down, 'D'),
    ([0x21], KcF, Down, 'F'),
    ([0x22], KcG, Down, 'G'),
    ([0x23], KcH, Down, 'H'),
    ([0x24], KcJ, Down, 'J'),
    ([0x25], KcK, Down, 'K'),
    ([0x26], KcL, Down, 'L'),
    ([0x27], KcSemiColon, Down, ';'),
    ([0x28], KcSingleQuote, Down, '\''),
    ([0x29], KcTick, Down, '`'),
    ([0x2A], KcLeftShift, Down, None),
    ([0x2B], KcBackSlash, Down, '\\'),
    ([0x2C], KcZ, Down, 'Z'),
    ([0x2D], KcX, Down, 'X'),
    ([0x2E], KcC, Down, 'C'),
    ([0x2F], KcV, Down, 'V'),
    ([0x30], KcB, Down, 'B'),
    ([0x31], KcN, Down, 'N'),
    ([0x32], KcM, Down, 'M'),
    ([0x33], KcComma, Down, ','),
    ([0x34], KcPeriod, Down, '.'),
    ([0x35], KcForwardSlash, Down, '/'),
    ([0x36], KcRightShift, Down, None),
    ([0x37], KcKpAst, Down, '*'),
    ([0x38], KcLeftAlt, Down, None),
    ([0x39], KcSpace, Down, None),
    ([0x3A], KcCapsLock, Down, None),
    ([0x3B], KcF1, Down, None),
    ([0x3C], KcF2, Down, None),
    ([0x3D], KcF3, Down, None),
    ([0x3E], KcF4, Down, None),
    ([0x3F], KcF5, Down, None),
    ([0x40], KcF6, Down, None),
    ([0x41], KcF7, Down, None),
    ([0x42], KcF8, Down, None),
    ([0x43], KcF9, Down, None),
    ([0x44], KcF10, Down, None),
    ([0x45], KcNumberLock, Down, None),
    ([0x46], KcScrollLock, Down, None),
    ([0x47], KcKp7, Down, '7'),
    ([0x48], KcKp8, Down, '8'),
    ([0x49], KcKp9, Down, '9'),
    ([0x4A], KcKpMinus, Down, '-'),
    ([0x4B], KcKp4, Down, '4'),
    ([0x4C], KcKp5, Down, '5'),
    ([0x4D], KcKp6, Down, '6'),
    ([0x4E], KcKpPlus, Down, '+'),
    ([0x4F], KcKp1, Down, '1'),
    ([0x50], KcKp2, Down, '2'),
    ([0x51], KcKp3, Down, '3'),
    ([0x52], KcKp0, Down, '0'),
    ([0x53], KcKpPeriod, Down, '.'),
    ([0x57], KcF11, Down, None),
    ([0x58], KcF12, Down, None),
    // Simple keycodes down
    ([0x81], KcEsc, Up, None),
    ([0x82], Kc1, Up, '1'),
    ([0x83], Kc2, Up, '2'),
    ([0x84], Kc3, Up, '3'),
    ([0x85], Kc4, Up, '4'),
    ([0x86], Kc5, Up, '5'),
    ([0x87], Kc6, Up, '6'),
    ([0x88], Kc7, Up, '7'),
    ([0x89], Kc8, Up, '8'),
    ([0x8A], Kc9, Up, '9'),
    ([0x8B], Kc0, Up, '0'),
    ([0x8C], KcMinus, Up, '-'),
    ([0x8D], KcEquals, Up, '='),
    ([0x8E], KcBackspace, Up, None),
    ([0x8F], KcTab, Up, None),
    ([0x90], KcQ, Up, 'Q'),
    ([0x91], KcW, Up, 'W'),
    ([0x92], KcE, Up, 'E'),
    ([0x93], KcR, Up, 'R'),
    ([0x94], KcT, Up, 'T'),
    ([0x95], KcY, Up, 'Y'),
    ([0x96], KcU, Up, 'U'),
    ([0x97], KcI, Up, 'I'),
    ([0x98], KcO, Up, 'O'),
    ([0x99], KcP, Up, 'P'),
    ([0x9A], KcOpenSquare, Up, '['),
    ([0x9B], KcCloseSquare, Up, ']'),
    ([0x9C], KcEnter, Up, None),
    ([0x9D], KcLeftControl, Up, None),
    ([0x9E], KcA, Up, 'A'),
    ([0x9F], KcS, Up, 'S'),
    ([0xA0], KcD, Up, 'D'),
    ([0xA1], KcF, Up, 'F'),
    ([0xA2], KcG, Up, 'G'),
    ([0xA3], KcH, Up, 'H'),
    ([0xA4], KcJ, Up, 'J'),
    ([0xA5], KcK, Up, 'K'),
    ([0xA6], KcL, Up, 'L'),
    ([0xA7], KcSemiColon, Up,';'),
    ([0xA8], KcSingleQuote, Up, '\''),
    ([0xA9], KcTick, Up, '`'),
    ([0xAA], KcLeftShift, Up, None),
    ([0xAB], KcBackSlash, Up, '\\'),
    ([0xAC], KcZ, Up, 'Z'),
    ([0xAD], KcX, Up, 'X'),
    ([0xAE], KcC, Up, 'C'),
    ([0xAF], KcV, Up, 'V'),
    ([0xB0], KcB, Up, 'B'),
    ([0xB1], KcN, Up, 'N'),
    ([0xB2], KcM, Up, 'M'),
    ([0xB3], KcComma, Up,','),
    ([0xB4], KcPeriod, Up,'.'),
    ([0xB5], KcForwardSlash, Up,'/'),
    ([0xB6], KcRightShift, Up, None),
    ([0xB7], KcKpAst, Up, None),
    ([0xB8], KcLeftAlt, Up, None),
    ([0xB9], KcSpace, Up, ' '),
    ([0xBA], KcCapsLock, Up, None),
    ([0xBB], KcF1, Up, None),
    ([0xBC], KcF2, Up, None),
    ([0xBD], KcF3, Up, None),
    ([0xBE], KcF4, Up, None),
    ([0xBF], KcF5, Up, None),
    ([0xC0], KcF6, Up, None),
    ([0xC1], KcF7, Up, None),
    ([0xC2], KcF8, Up, None),
    ([0xC3], KcF9, Up, None),
    ([0xC4], KcF10, Up, None),
    ([0xC5], KcNumberLock, Up, None),
    ([0xC6], KcScrollLock, Up, None),
    ([0xC7], KcKp7, Up, '7'),
    ([0xC8], KcKp8, Up, '8'),
    ([0xC9], KcKp9, Up, '9'),
    ([0xCA], KcKpMinus, Up, '-'),
    ([0xCB], KcKp4, Up, '4'),
    ([0xCC], KcKp5, Up, '5'),
    ([0xCD], KcKp6, Up, '6'),
    ([0xCE], KcKpPlus, Up, '+'),
    ([0xCF], KcKp1, Up, '1'),
    ([0xD0], KcKp2, Up, '2'),
    ([0xD1], KcKp3, Up, '3'),
    ([0xD2], KcKp0, Up, '0'),
    ([0xD3], KcKpPeriod, Up, '.'),
    ([0xD7], KcF11, Up,None),
    ([0xD8], KcF12, Up,None),
    // Complex keycodes down
    ([0xE0, 0x10], KcMultiMediaTrackPrevious, Down, None),
    ([0xE0, 0x19], KcMultiMediaTrackNext,  Down, None),
    ([0xE0, 0x1C], KcKpEnter, Down, None),
    ([0xE0, 0x1D], KcRightCtrl, Down, None),
    ([0xE0, 0x20], KcMultiMediaMute, Down, None),
    ([0xE0, 0x21], KcMultiMediaCalculator, Down, None),
    ([0xE0, 0x22], KcMultiMediaPlay, Down, None),
    ([0xE0, 0x24], KcMultiMediaStop, Down, None),
    ([0xE0, 0x2E], KcMultiMediaVolumeDown, Down, None),
    ([0xE0, 0x30], KcMultiMediaVolumeUp, Down, None),
    ([0xE0, 0x32], KcMultiMediaWwwHome, Down, None),
    ([0xE0, 0x35], KcKpForwardSlash,  Down, '/'),
    ([0xE0, 0x38], KcRightAlt, Down, None),
    ([0xE0, 0x47], KcHome,  Down, None),
    ([0xE0, 0x48], KcUp, Down, None),
    ([0xE0, 0x49], KcPageUp, Down, None),
    ([0xE0, 0x4B], KcLeft, Down, None),
    ([0xE0, 0x4D], KcRight, Down, None),
    ([0xE0, 0x4F], KcEnd, Down, None),
    ([0xE0, 0x50], KcDown, Down, None),
    ([0xE0, 0x51], KcPageDown Down, None),
    ([0xE0, 0x52], KcInsert, Down, None),
    ([0xE0, 0x53], KcDelete, Down, None),
    ([0xE0, 0x5B], KcLeftGui, Down, None),
    ([0xE0, 0x5C], KcRightGui, Down, None),
    ([0xE0, 0x5D], KcApps, Down, None),
    ([0xE0, 0x5E], KcAcpiPower, Down, None),
    ([0xE0, 0x5F], KcAcpiSleep, Down, None),
    ([0xE0, 0x63], KcAcpiWake, Down, None),
    ([0xE0, 0x65], KcMultiMediaWwwSearch, Down, None),
    ([0xE0, 0x66], KcMultiMediaWwwFavorites, Down, None),
    ([0xE0, 0x67], KcMultiMediaWwwRefresh, Down, None),
    ([0xE0, 0x68], KcMultiMediaWwwStop, Down, None),
    ([0xE0, 0x69], KcMultiMediaWwwForward, Down, None),
    ([0xE0, 0x6A], KcMultiMediaWwwBack, Down, None),
    ([0xE0, 0x6B], KcMultiMediaMyComputer, Down, None),
    ([0xE0, 0x6C], KcMultiMediaEmail, Down, None),
    ([0xE0, 0x6D], KcMultiMediaMediaSelect, Down, None),
    // Complex keycodes up
    ([0xE0, 0x90], KcMultiMediaTrackPrevious, Up, None),
    ([0xE0, 0x99], KcMultiMediaTrackNext, Up, None),
    ([0xE0, 0x9C], KcKpEnter Up, None),
    ([0xE0, 0x9D], KcRightCtrl, Up, None),
    ([0xE0, 0xA0], KcMultiMediaMute, Up, None),
    ([0xE0, 0xA1], KcMultiMediaCalculator, Up, None),
    ([0xE0, 0xA2], KcMultiMediaPlay, Up, None),
    ([0xE0, 0xA4], KcMultiMediaStop, Up, None),
    ([0xE0, 0xAE], KcMultiMediaVolumeDown, Up, None),
    ([0xE0, 0xB0], KcMultiMediaVolumeUp, Up, None),
    ([0xE0, 0xB2], KcMultiMediaWwwHome, Up, None),
    ([0xE0, 0xB5], KcKpForwardSlash, Up, '/'),
    ([0xE0, 0xB8], KcRightAlt, Up, None),
    ([0xE0, 0xC7], KcHome, Up, None),
    ([0xE0, 0xC8], KcUp, Up, None),
    ([0xE0, 0xC9], KcPageUp, Up, None),
    ([0xE0, 0xCB], KcLeft, Up, None),
    ([0xE0, 0xCD], KcRight, Up, None),
    ([0xE0, 0xCF], KcEnd, Up, None),
    ([0xE0, 0xD0], KcDown, Up, None),
    ([0xE0, 0xD1], KcPageDown, Up, None),
    ([0xE0, 0xD2], KcInsert, Up, None),
    ([0xE0, 0xD3], KcDelete, Up, None),
    ([0xE0, 0xDB], KcLeftGui, Up, None),
    ([0xE0, 0xDC], KcRightGui, Up, None),
    ([0xE0, 0xDD], KcApps, Up, None),
    ([0xE0, 0xDE], KcAcpiPower, Up, None),
    ([0xE0, 0xDF], KcAcpiSleep, Up, None),
    ([0xE0, 0xE3], KcAcpiWake, Up, None),
    ([0xE0, 0xE5], KcMultiMediaWwwSearch, Up, None),
    ([0xE0, 0xE6], KcMultiMediaWwwFavorites,  Up, None),
    ([0xE0, 0xE7], KcMultiMediaWwwRefresh,  Up, None),
    ([0xE0, 0xE8], KcMultiMediaWwwStop,  Up, None),
    ([0xE0, 0xE9], KcMultiMediaWwwForward,  Up, None),
    ([0xE0, 0xEA], KcMultiMediaWwwBack, Up, None),
    ([0xE0, 0xEB], KcMultiMediaMyComputer,  Up, None),
    ([0xE0, 0xEC], KcMultiMediaEmail, Up, None),
    ([0xE0, 0xED], KcMultiMediaMediaSelect,  Up, None),
    // Special children
    ([0xE0, 0xB7, 0xE0, 0xAA], KcPrintScreen, Up, None),
    ([0xE0, 0x2A, 0xE0, 0x37], KcPrintScreen, Down, None),
    ([0xE1, 0x1D, 0x45, 0xE1, 0x9D, 0xC5], KcPause, Down, None),
);
