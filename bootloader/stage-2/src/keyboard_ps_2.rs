use core::arch::asm;

pub(crate) fn wait_keypress() -> char {
    while !has_keypres() {}

    let scancode: u8;
    unsafe {
        asm!(
            "in al, 0x60",
             out("al") scancode,
        );
    };

    if scancode & 1 << 7 != 0 {
        // This is a key release, not a press
        wait_keypress()
    } else if let Some(kc) = KeyCode::from_scancode_1(scancode) {
        kc.char()
    } else {
        '?'
    }
}

pub(crate) fn has_keypres() -> bool {
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

enum KeyCode {
    Kc1,
    Kc2,
    Kc3,
    Kc4,
    Kc5,
    Kc6,
    Kc7,
    Kc8,
    Kc9,
    Kc0,
}

impl KeyCode {
    fn from_scancode_1(code: u8) -> Option<Self> {
        match code & 0xEF {
            0x2 => Some(KeyCode::Kc1),
            0x3 => Some(KeyCode::Kc2),
            0x4 => Some(KeyCode::Kc3),
            0x5 => Some(KeyCode::Kc4),
            0x6 => Some(KeyCode::Kc5),
            0x7 => Some(KeyCode::Kc6),
            0x8 => Some(KeyCode::Kc7),
            0x9 => Some(KeyCode::Kc8),
            0xa => Some(KeyCode::Kc9),
            0xb => Some(KeyCode::Kc0),
            _ => None,
        }
    }

    fn char(&self) -> char {
        match self {
            KeyCode::Kc1 => '1',
            KeyCode::Kc2 => '2',
            KeyCode::Kc3 => '3',
            KeyCode::Kc4 => '4',
            KeyCode::Kc5 => '5',
            KeyCode::Kc6 => '6',
            KeyCode::Kc7 => '7',
            KeyCode::Kc8 => '8',
            KeyCode::Kc9 => '9',
            KeyCode::Kc0 => '0',
        }
    }
}
