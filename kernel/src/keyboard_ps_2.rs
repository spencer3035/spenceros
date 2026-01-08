use core::arch::asm;

use proc_macros::FromScancodes;

#[derive(Default)]
pub struct KeyboardDriver {
    shift_held: bool,
    ctrl_held: bool,
    gui_held: bool,
    alt_held: bool,
}

impl KeyboardDriver {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn has_event(&self) -> bool {
        has_scancode()
    }

    #[allow(dead_code)]
    pub fn next_char(&mut self) -> char {
        loop {
            let kc = self.next_keypress();
            if let Some(ch) = self.modify_key_to_char(kc) {
                return ch;
            }
        }
    }

    pub fn next_key_event(&mut self) -> KeyEvent {
        wait_key_event()
    }

    pub fn modify_key_to_char(&self, key: KeyCode) -> Option<char> {
        if self.shift_held {
            key.to_char_upper()
        } else {
            key.to_char_lower()
        }
    }

    #[allow(dead_code)]
    pub fn next_keypress(&mut self) -> KeyCode {
        let mut kc = self.next_key_event();
        loop {
            if let Some(modi) = kc.code.is_modifier() {
                self.handle_modifier(modi, kc.is_press);
            } else if kc.is_press {
                return kc.code;
            }
            kc = self.next_key_event();
        }
    }

    fn handle_modifier(&mut self, modifier: Modifier, is_down: bool) {
        match modifier {
            Modifier::Shift => {
                self.shift_held = is_down;
            }
            Modifier::Control => {
                self.ctrl_held = is_down;
            }
            Modifier::Alt => {
                self.alt_held = is_down;
            }
            Modifier::Gui => {
                self.gui_held = is_down;
            }
        }
    }
}

/// Waits until the next keyboard event and returns it
#[allow(dead_code)]
pub fn wait_key_event() -> KeyEvent {
    loop {
        if let Ok(val) = get_next_key_event() {
            return val;
        }
    }
}

#[derive(Debug)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub is_press: bool,
}

/// Gets the next keycode
fn get_next_key_event() -> Result<KeyEvent, ()> {
    let code = next_scancode();
    let index = 0;
    get_next_key_event_impl(code, index)
}

/// Handles polling for new scancodes until they are mapped into a proper key event
fn get_next_key_event_impl(code: u8, index: u8) -> Result<KeyEvent, ()> {
    if KeyCode::has_next(code, index) {
        let code = next_scancode();
        get_next_key_event_impl(code, index + 1)
    } else {
        match keycode_from_index_and_code(code, index) {
            Some(val) => Ok(val),
            None => Err(()),
        }
    }
}

fn keycode_from_index_and_code(code: u8, index: u8) -> Option<KeyEvent> {
    let (kc, is_down) = KeyCode::from_scancode_and_depth(code, index)?;
    Some(KeyEvent {
        code: kc,
        is_press: is_down,
    })
}

/// Blocks until we can read another scancode
fn next_scancode() -> u8 {
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

/// Checks if there is a pending scancode to be read, not blocking
fn has_scancode() -> bool {
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

pub trait FromScancodes: Sized {
    /// If the current code and index is terminal, assuming that all previous vales were terminal
    fn has_next(code: u8, index: u8) -> bool;
    /// Gets the scancode given the terminal code and the number of codes, as well as if it is a
    /// down press or not (_, true) is downpress, (_, false) is a release.
    fn from_scancode_and_depth(code: u8, index: u8) -> Option<(Self, bool)>;
    /// Tries to convert the key to an unshifted character
    fn to_char_lower(&self) -> Option<char>;
    /// Tries to convert the key to a shifted character
    fn to_char_upper(&self) -> Option<char>;
}

pub enum Modifier {
    Shift,
    Control,
    Alt,
    Gui,
}

impl KeyCode {
    /// Returns the kind of modifer key it is (if it is one)
    #[allow(dead_code)]
    pub fn is_modifier(&self) -> Option<Modifier> {
        if self.is_shift() {
            Some(Modifier::Shift)
        } else if self.is_ctrl() {
            Some(Modifier::Control)
        } else if self.is_alt() {
            Some(Modifier::Alt)
        } else if self.is_gui() {
            Some(Modifier::Gui)
        } else {
            None
        }
    }
    pub fn is_backspace(&self) -> bool {
        *self == KeyCode::KcBackspace
    }
    pub fn is_shift(&self) -> bool {
        *self == KeyCode::KcLeftShift || *self == KeyCode::KcRightShift
    }
    pub fn is_ctrl(&self) -> bool {
        *self == KeyCode::KcLeftControl || *self == KeyCode::KcRightCtrl
    }
    pub fn is_alt(&self) -> bool {
        *self == KeyCode::KcLeftAlt || *self == KeyCode::KcRightAlt
    }
    #[allow(dead_code)]
    pub fn is_enter(&self) -> bool {
        *self == KeyCode::KcEnter || *self == KeyCode::KcKpEnter
    }
    pub fn is_gui(&self) -> bool {
        *self == KeyCode::KcLeftGui || *self == KeyCode::KcRightGui
    }

    pub fn is_char(&self) -> bool {
        self.to_char_lower().is_some()
    }
}

/// Possible keys that can be pressed
#[derive(FromScancodes, Debug, PartialEq, Eq)]
pub enum KeyCode {
    /// Escape
    #[scan(press=[0x01],release=[0x81])]
    KcEsc,
    #[scan(press=[0x02],release=[0x82],lower='1',upper='!')]
    Kc1,
    #[scan(press=[0x03],release=[0x83],lower='2',upper='@')]
    Kc2,
    #[scan(press=[0x04],release=[0x84],lower='3',upper='#')]
    Kc3,
    #[scan(press=[0x05],release=[0x85],lower='4',upper='$')]
    Kc4,
    #[scan(press=[0x06],release=[0x86],lower='5',upper='%')]
    Kc5,
    #[scan(press=[0x07],release=[0x87],lower='6',upper='^')]
    Kc6,
    #[scan(press=[0x08],release=[0x88],lower='7',upper='&')]
    Kc7,
    #[scan(press=[0x09],release=[0x89],lower='8',upper='*')]
    Kc8,
    #[scan(press=[0x0A],release=[0x8A],lower='9',upper='(')]
    Kc9,
    #[scan(press=[0x0B],release=[0x8B],lower='0',upper=')')]
    Kc0,
    #[scan(press=[0x0C],release=[0x8C],lower='-',upper='_')]
    KcMinus,
    #[scan(press=[0x0D],release=[0x8D],lower='=',upper='+')]
    KcEquals,
    #[scan(press=[0x0E],release=[0x8E])]
    KcBackspace,
    #[scan(press=[0x0F],release=[0x8F])]
    KcTab,
    #[scan(press=[0x10],release=[0x90],lower='q',upper='Q')]
    KcQ,
    #[scan(press=[0x11],release=[0x91],lower='w',upper='W')]
    KcW,
    #[scan(press=[0x12],release=[0x92],lower='e',upper='E')]
    KcE,
    #[scan(press=[0x13],release=[0x93],lower='r',upper='R')]
    KcR,
    #[scan(press=[0x14],release=[0x94],lower='t',upper='T')]
    KcT,
    #[scan(press=[0x15],release=[0x95],lower='y',upper='Y')]
    KcY,
    #[scan(press=[0x16],release=[0x96],lower='u',upper='U')]
    KcU,
    #[scan(press=[0x17],release=[0x97],lower='i',upper='I')]
    KcI,
    #[scan(press=[0x18],release=[0x98],lower='o',upper='O')]
    KcO,
    #[scan(press=[0x19],release=[0x99],lower='p',upper='P')]
    KcP,
    #[scan(press=[0x1A],release=[0x9A],lower='[',upper='{')]
    KcOpenSquare,
    #[scan(press=[0x1B],release=[0x9B],lower=']',upper='}')]
    KcCloseSquare,
    #[scan(press=[0x1C],release=[0x9C],lower='\n',upper='\n')]
    KcEnter,
    #[scan(press=[0x1D],release=[0x9D])]
    KcLeftControl,
    #[scan(press=[0x1E],release=[0x9E],lower='a',upper='A')]
    KcA,
    #[scan(press=[0x1F],release=[0x9F],lower='s',upper='S')]
    KcS,
    #[scan(press=[0x20],release=[0xA0],lower='d',upper='D')]
    KcD,
    #[scan(press=[0x21],release=[0xA1],lower='f',upper='F')]
    KcF,
    #[scan(press=[0x22],release=[0xA2],lower='g',upper='G')]
    KcG,
    #[scan(press=[0x23],release=[0xA3],lower='h',upper='H')]
    KcH,
    #[scan(press=[0x24],release=[0xA4],lower='j',upper='J')]
    KcJ,
    #[scan(press=[0x25],release=[0xA5],lower='k',upper='K')]
    KcK,
    #[scan(press=[0x26],release=[0xA6],lower='l',upper='L')]
    KcL,
    #[scan(press=[0x27],release=[0xA7],lower=';',upper=':')]
    KcSemiColon,
    #[scan(press=[0x28],release=[0xA8],lower='\'',upper='"')]
    KcSingleQuote,
    #[scan(press=[0x29],release=[0xA9],lower='`',upper='~')]
    KcTick,
    #[scan(press=[0x2A],release=[0xAA])]
    KcLeftShift,
    #[scan(press=[0x2B],release=[0xAB],lower='\\',upper='|')]
    KcBackSlash,
    #[scan(press=[0x2C],release=[0xAC],lower='z',upper='Z')]
    KcZ,
    #[scan(press=[0x2D],release=[0xAD],lower='x',upper='X')]
    KcX,
    #[scan(press=[0x2E],release=[0xAE],lower='c',upper='C')]
    KcC,
    #[scan(press=[0x2F],release=[0xAF],lower='v',upper='V')]
    KcV,
    #[scan(press=[0x30],release=[0xB0],lower='b',upper='B')]
    KcB,
    #[scan(press=[0x31],release=[0xB1],lower='n',upper='N')]
    KcN,
    #[scan(press=[0x32],release=[0xB2],lower='m',upper='M')]
    KcM,
    #[scan(press=[0x33],release=[0xB3],lower=',',upper='<')]
    KcComma,
    #[scan(press=[0x34],release=[0xB4],lower='.',upper='>')]
    KcPeriod,
    #[scan(press=[0x35],release=[0xB5],lower='/',upper='?')]
    KcForwardSlash,
    #[scan(press=[0x36],release=[0xB6])]
    KcRightShift,
    // TODO: I use this to test error handling, uncomment when done
    // #[scan(press=[0x37],release=[0xB7],lower='*',upper='')]
    // KcKpAst,
    #[scan(press=[0x38],release=[0xB8])]
    KcLeftAlt,
    #[scan(press=[0x39],release=[0xB9],lower=' ',upper=' ')]
    KcSpace,
    #[scan(press=[0x3A],release=[0xBA])]
    KcCapsLock,
    #[scan(press=[0x3B],release=[0xBB])]
    KcF1,
    #[scan(press=[0x3C],release=[0xBC])]
    KcF2,
    #[scan(press=[0x3D],release=[0xBD])]
    KcF3,
    #[scan(press=[0x3E],release=[0xBE])]
    KcF4,
    #[scan(press=[0x3F],release=[0xBF])]
    KcF5,
    #[scan(press=[0x40],release=[0xC0])]
    KcF6,
    #[scan(press=[0x41],release=[0xC1])]
    KcF7,
    #[scan(press=[0x42],release=[0xC2])]
    KcF8,
    #[scan(press=[0x43],release=[0xC3])]
    KcF9,
    #[scan(press=[0x44],release=[0xC4])]
    KcF10,
    #[scan(press=[0x45],release=[0xC5])]
    KcNumberLock,
    #[scan(press=[0x46],release=[0xC6])]
    KcScrollLock,
    #[scan(press=[0x47],release=[0xC7],lower='7')]
    KcKp7,
    #[scan(press=[0x48],release=[0xC8],lower='8')]
    KcKp8,
    #[scan(press=[0x49],release=[0xC9],lower='9')]
    KcKp9,
    #[scan(press=[0x4A],release=[0xCA],lower='-')]
    KcKpMinus,
    #[scan(press=[0x4B],release=[0xCB],lower='4')]
    KcKp4,
    #[scan(press=[0x4C],release=[0xCC],lower='5')]
    KcKp5,
    #[scan(press=[0x4D],release=[0xCD],lower='6')]
    KcKp6,
    #[scan(press=[0x4E],release=[0xCE],lower='+')]
    KcKpPlus,
    #[scan(press=[0x4F],release=[0xCF],lower='1')]
    KcKp1,
    #[scan(press=[0x50],release=[0xD0],lower='2')]
    KcKp2,
    #[scan(press=[0x51],release=[0xD1],lower='3')]
    KcKp3,
    #[scan(press=[0x52],release=[0xD2],lower='0')]
    KcKp0,
    #[scan(press=[0x53],release=[0xD3],lower='.')]
    KcKpPeriod,
    #[scan(press=[0x57],release=[0xD7])]
    KcF11,
    #[scan(press=[0x58],release=[0xD8])]
    KcF12,
    #[scan(press=[0xE0,0x10],release=[0xE0,0x90])]
    KcMultiMediaTrackPrevious,
    #[scan(press=[0xE0,0x19],release=[0xE0,0x99])]
    KcMultiMediaTrackNext,
    #[scan(press=[0xE0,0x1C],release=[0xE0,0x9C],lower='\n',upper='\n')]
    KcKpEnter,
    #[scan(press=[0xE0,0x1D],release=[0xE0,0x9D])]
    KcRightCtrl,
    #[scan(press=[0xE0,0x20],release=[0xE0,0xA0])]
    KcMultiMediaMute,
    #[scan(press=[0xE0,0x21],release=[0xE0,0xA1])]
    KcMultiMediaCalculator,
    #[scan(press=[0xE0,0x22],release=[0xE0,0xA2])]
    KcMultiMediaPlay,
    #[scan(press=[0xE0,0x24],release=[0xE0,0xA4])]
    KcMultiMediaStop,
    #[scan(press=[0xE0,0x2E],release=[0xE0,0xAE])]
    KcMultiMediaVolumeDown,
    #[scan(press=[0xE0,0x30],release=[0xE0,0xB0])]
    KcMultiMediaVolumeUp,
    #[scan(press=[0xE0,0x32],release=[0xE0,0xB2])]
    KcMultiMediaWwwHome,
    #[scan(press=[0xE0,0x35],release=[0xE0,0xB5],lower='/',upper='/')]
    KcKpForwardSlash,
    #[scan(press=[0xE0,0x38],release=[0xE0,0xB8])]
    KcRightAlt,
    #[scan(press=[0xE0,0x47],release=[0xE0,0xC7])]
    KcHome,
    #[scan(press=[0xE0,0x48],release=[0xE0,0xC8])]
    KcUp,
    #[scan(press=[0xE0,0x49],release=[0xE0,0xC9])]
    KcPageUp,
    #[scan(press=[0xE0,0x4B],release=[0xE0,0xCB])]
    KcLeft,
    #[scan(press=[0xE0,0x4D],release=[0xE0,0xCD])]
    KcRight,
    #[scan(press=[0xE0,0x4F],release=[0xE0,0xCF])]
    KcEnd,
    #[scan(press=[0xE0,0x50],release=[0xE0,0xD0])]
    KcDown,
    #[scan(press=[0xE0,0x51],release=[0xE0,0xD1])]
    KcPageDown,
    #[scan(press=[0xE0,0x52],release=[0xE0,0xD2])]
    KcInsert,
    #[scan(press=[0xE0,0x53],release=[0xE0,0xD3])]
    KcDelete,
    #[scan(press=[0xE0,0x5B],release=[0xE0,0xDB])]
    KcLeftGui,
    #[scan(press=[0xE0,0x5C],release=[0xE0,0xDC])]
    KcRightGui,
    #[scan(press=[0xE0,0x5D],release=[0xE0,0xDD])]
    KcApps,
    #[scan(press=[0xE0,0x5E],release=[0xE0,0xDE])]
    KcAcpiPower,
    #[scan(press=[0xE0,0x5F],release=[0xE0,0xDF])]
    KcAcpiSleep,
    #[scan(press=[0xE0,0x63],release=[0xE0,0xE3])]
    KcAcpiWake,
    #[scan(press=[0xE0,0x65],release=[0xE0,0xE5])]
    KcMultiMediaWwwSearch,
    #[scan(press=[0xE0,0x66],release=[0xE0,0xE6])]
    KcMultiMediaWwwFavorites,
    #[scan(press=[0xE0,0x67],release=[0xE0,0xE7])]
    KcMultiMediaWwwRefresh,
    #[scan(press=[0xE0,0x68],release=[0xE0,0xE8])]
    KcMultiMediaWwwStop,
    #[scan(press=[0xE0,0x69],release=[0xE0,0xE9])]
    KcMultiMediaWwwForward,
    #[scan(press=[0xE0,0x6A],release=[0xE0,0xEA])]
    KcMultiMediaWwwBack,
    #[scan(press=[0xE0,0x6B],release=[0xE0,0xEB])]
    KcMultiMediaMyComputer,
    #[scan(press=[0xE0,0x6C],release=[0xE0,0xEC])]
    KcMultiMediaEmail,
    #[scan(press=[0xE0,0x6D],release=[0xE0,0xED])]
    KcMultiMediaMediaSelect,
    #[scan(press=[0xE0,0x2A,0xE0,0x37],release=[0xE0,0xB7,0xE0,0xAA])]
    KcPrintScreen,
    #[scan(press=[0xE1,0x1D,0x45,0xE1,0x9D,0xC5],release=[])]
    KcPause,
}
