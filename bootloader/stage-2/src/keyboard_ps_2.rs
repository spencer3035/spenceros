use core::arch::asm;

use proc_macros::FromScancodes;

#[derive(Default)]
struct KeyboardDriver {
    shift_held: u8,
    ctrl_held: u8,
    gui_held: u8,
    alt_held: u8,
}

impl KeyboardDriver {
    #[allow(dead_code)]
    pub fn has_keypress(&self) -> bool {
        has_scancode()
    }

    #[allow(dead_code)]
    pub fn next_keypress(&mut self) -> KeyCode {
        let mut kc = wait_key_event();
        loop {
            if let Some(modi) = kc.code.is_modifier() {
                self.handle_modifier(modi, kc.is_down);
            } else {
                return kc.code;
            }
            kc = wait_key_event();
        }
    }

    fn handle_modifier(&mut self, modifier: Modifier, is_down: bool) {
        match modifier {
            Modifier::Shift => {
                if is_down {
                    if self.shift_held > 0 {
                        self.shift_held -= 1;
                    }
                } else {
                    self.shift_held += 1;
                }
            }
            Modifier::Control => {
                if is_down {
                    if self.ctrl_held > 0 {
                        self.ctrl_held -= 1;
                    }
                } else {
                    self.ctrl_held += 1;
                }
            }
            Modifier::Alt => {
                if is_down {
                    if self.alt_held > 0 {
                        self.alt_held -= 1;
                    }
                } else {
                    self.alt_held += 1;
                }
            }
            Modifier::Gui => {
                if is_down {
                    if self.gui_held > 0 {
                        self.gui_held -= 1;
                    }
                } else {
                    self.gui_held += 1;
                }
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
    pub is_down: bool,
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
    Some(KeyEvent { code: kc, is_down })
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
    /// Tries to conver the key to a character
    fn to_char(&self) -> Option<char>;
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
}

/// Possible keys that can be pressed
#[derive(FromScancodes, Debug, PartialEq, Eq)]
pub enum KeyCode {
    /// Escape
    #[scan(down=[0x01],up=[0x81])]
    KcEsc,
    #[scan(down=[0x02],up=[0x82],ch='1')]
    Kc1,
    #[scan(down=[0x03],up=[0x83],ch='2')]
    Kc2,
    #[scan(down=[0x04],up=[0x84],ch='3')]
    Kc3,
    #[scan(down=[0x05],up=[0x85],ch='4')]
    Kc4,
    #[scan(down=[0x06],up=[0x86],ch='5')]
    Kc5,
    #[scan(down=[0x07],up=[0x87],ch='6')]
    Kc6,
    #[scan(down=[0x08],up=[0x88],ch='7')]
    Kc7,
    #[scan(down=[0x09],up=[0x89],ch='8')]
    Kc8,
    #[scan(down=[0x0A],up=[0x8A],ch='9')]
    Kc9,
    #[scan(down=[0x0B],up=[0x8B],ch='0')]
    Kc0,
    #[scan(down=[0x0C],up=[0x8C],ch='-')]
    KcMinus,
    #[scan(down=[0x0D],up=[0x8D],ch='=')]
    KcEquals,
    #[scan(down=[0x0E],up=[0x8E])]
    KcBackspace,
    #[scan(down=[0x0F],up=[0x8F])]
    KcTab,
    #[scan(down=[0x10],up=[0x90],ch='Q')]
    KcQ,
    #[scan(down=[0x11],up=[0x91],ch='W')]
    KcW,
    #[scan(down=[0x12],up=[0x92],ch='E')]
    KcE,
    #[scan(down=[0x13],up=[0x93],ch='R')]
    KcR,
    #[scan(down=[0x14],up=[0x94],ch='T')]
    KcT,
    #[scan(down=[0x15],up=[0x95],ch='Y')]
    KcY,
    #[scan(down=[0x16],up=[0x96],ch='U')]
    KcU,
    #[scan(down=[0x17],up=[0x97],ch='I')]
    KcI,
    #[scan(down=[0x18],up=[0x98],ch='O')]
    KcO,
    #[scan(down=[0x19],up=[0x99],ch='P')]
    KcP,
    #[scan(down=[0x1A],up=[0x9A],ch='[')]
    KcOpenSquare,
    #[scan(down=[0x1B],up=[0x9B],ch=']')]
    KcCloseSquare,
    #[scan(down=[0x1C],up=[0x9C])]
    KcEnter,
    #[scan(down=[0x1D],up=[0x9D])]
    KcLeftControl,
    #[scan(down=[0x1E],up=[0x9E],ch='A')]
    KcA,
    #[scan(down=[0x1F],up=[0x9F],ch='S')]
    KcS,
    #[scan(down=[0x20],up=[0xA0],ch='D')]
    KcD,
    #[scan(down=[0x21],up=[0xA1],ch='F')]
    KcF,
    #[scan(down=[0x22],up=[0xA2],ch='G')]
    KcG,
    #[scan(down=[0x23],up=[0xA3],ch='H')]
    KcH,
    #[scan(down=[0x24],up=[0xA4],ch='J')]
    KcJ,
    #[scan(down=[0x25],up=[0xA5],ch='K')]
    KcK,
    #[scan(down=[0x26],up=[0xA6],ch='L')]
    KcL,
    #[scan(down=[0x27],up=[0xA7],ch=';')]
    KcSemiColon,
    #[scan(down=[0x28],up=[0xA8],ch='\'')]
    KcSingleQuote,
    #[scan(down=[0x29],up=[0xA9],ch='`')]
    KcTick,
    #[scan(down=[0x2A],up=[0xAA])]
    KcLeftShift,
    #[scan(down=[0x2B],up=[0xAB],ch='\\')]
    KcBackSlash,
    #[scan(down=[0x2C],up=[0xAC],ch='Z')]
    KcZ,
    #[scan(down=[0x2D],up=[0xAD],ch='X')]
    KcX,
    #[scan(down=[0x2E],up=[0xAE],ch='C')]
    KcC,
    #[scan(down=[0x2F],up=[0xAF],ch='V')]
    KcV,
    #[scan(down=[0x30],up=[0xB0],ch='B')]
    KcB,
    #[scan(down=[0x31],up=[0xB1],ch='N')]
    KcN,
    #[scan(down=[0x32],up=[0xB2],ch='M')]
    KcM,
    #[scan(down=[0x33],up=[0xB3],ch=',')]
    KcComma,
    #[scan(down=[0x34],up=[0xB4],ch='.')]
    KcPeriod,
    #[scan(down=[0x35],up=[0xB5],ch='/')]
    KcForwardSlash,
    #[scan(down=[0x36],up=[0xB6])]
    KcRightShift,
    // TODO: I use this to test error handling, uncomment when done
    // #[scan(down=[0x37],up=[0xB7],ch='*')]
    // KcKpAst,
    #[scan(down=[0x38],up=[0xB8])]
    KcLeftAlt,
    #[scan(down=[0x39],up=[0xB9],ch=' ')]
    KcSpace,
    #[scan(down=[0x3A],up=[0xBA])]
    KcCapsLock,
    #[scan(down=[0x3B],up=[0xBB])]
    KcF1,
    #[scan(down=[0x3C],up=[0xBC])]
    KcF2,
    #[scan(down=[0x3D],up=[0xBD])]
    KcF3,
    #[scan(down=[0x3E],up=[0xBE])]
    KcF4,
    #[scan(down=[0x3F],up=[0xBF])]
    KcF5,
    #[scan(down=[0x40],up=[0xC0])]
    KcF6,
    #[scan(down=[0x41],up=[0xC1])]
    KcF7,
    #[scan(down=[0x42],up=[0xC2])]
    KcF8,
    #[scan(down=[0x43],up=[0xC3])]
    KcF9,
    #[scan(down=[0x44],up=[0xC4])]
    KcF10,
    #[scan(down=[0x45],up=[0xC5])]
    KcNumberLock,
    #[scan(down=[0x46],up=[0xC6])]
    KcScrollLock,
    #[scan(down=[0x47],up=[0xC7],ch='7')]
    KcKp7,
    #[scan(down=[0x48],up=[0xC8],ch='8')]
    KcKp8,
    #[scan(down=[0x49],up=[0xC9],ch='9')]
    KcKp9,
    #[scan(down=[0x4A],up=[0xCA],ch='-')]
    KcKpMinus,
    #[scan(down=[0x4B],up=[0xCB],ch='4')]
    KcKp4,
    #[scan(down=[0x4C],up=[0xCC],ch='5')]
    KcKp5,
    #[scan(down=[0x4D],up=[0xCD],ch='6')]
    KcKp6,
    #[scan(down=[0x4E],up=[0xCE],ch='+')]
    KcKpPlus,
    #[scan(down=[0x4F],up=[0xCF],ch='1')]
    KcKp1,
    #[scan(down=[0x50],up=[0xD0],ch='2')]
    KcKp2,
    #[scan(down=[0x51],up=[0xD1],ch='3')]
    KcKp3,
    #[scan(down=[0x52],up=[0xD2],ch='0')]
    KcKp0,
    #[scan(down=[0x53],up=[0xD3],ch='.')]
    KcKpPeriod,
    #[scan(down=[0x57],up=[0xD7])]
    KcF11,
    #[scan(down=[0x58],up=[0xD8])]
    KcF12,
    #[scan(down=[0xE0,0x10],up=[0xE0,0x90])]
    KcMultiMediaTrackPrevious,
    #[scan(down=[0xE0,0x19],up=[0xE0,0x99])]
    KcMultiMediaTrackNext,
    #[scan(down=[0xE0,0x1C],up=[0xE0,0x9C])]
    KcKpEnter,
    #[scan(down=[0xE0,0x1D],up=[0xE0,0x9D])]
    KcRightCtrl,
    #[scan(down=[0xE0,0x20],up=[0xE0,0xA0])]
    KcMultiMediaMute,
    #[scan(down=[0xE0,0x21],up=[0xE0,0xA1])]
    KcMultiMediaCalculator,
    #[scan(down=[0xE0,0x22],up=[0xE0,0xA2])]
    KcMultiMediaPlay,
    #[scan(down=[0xE0,0x24],up=[0xE0,0xA4])]
    KcMultiMediaStop,
    #[scan(down=[0xE0,0x2E],up=[0xE0,0xAE])]
    KcMultiMediaVolumeDown,
    #[scan(down=[0xE0,0x30],up=[0xE0,0xB0])]
    KcMultiMediaVolumeUp,
    #[scan(down=[0xE0,0x32],up=[0xE0,0xB2])]
    KcMultiMediaWwwHome,
    #[scan(down=[0xE0,0x35],up=[0xE0,0xB5],ch='/')]
    KcKpForwardSlash,
    #[scan(down=[0xE0,0x38],up=[0xE0,0xB8])]
    KcRightAlt,
    #[scan(down=[0xE0,0x47],up=[0xE0,0xC7])]
    KcHome,
    #[scan(down=[0xE0,0x48],up=[0xE0,0xC8])]
    KcUp,
    #[scan(down=[0xE0,0x49],up=[0xE0,0xC9])]
    KcPageUp,
    #[scan(down=[0xE0,0x4B],up=[0xE0,0xCB])]
    KcLeft,
    #[scan(down=[0xE0,0x4D],up=[0xE0,0xCD])]
    KcRight,
    #[scan(down=[0xE0,0x4F],up=[0xE0,0xCF])]
    KcEnd,
    #[scan(down=[0xE0,0x50],up=[0xE0,0xD0])]
    KcDown,
    #[scan(down=[0xE0,0x51],up=[0xE0,0xD1])]
    KcPageDown,
    #[scan(down=[0xE0,0x52],up=[0xE0,0xD2])]
    KcInsert,
    #[scan(down=[0xE0,0x53],up=[0xE0,0xD3])]
    KcDelete,
    #[scan(down=[0xE0,0x5B],up=[0xE0,0xDB])]
    KcLeftGui,
    #[scan(down=[0xE0,0x5C],up=[0xE0,0xDC])]
    KcRightGui,
    #[scan(down=[0xE0,0x5D],up=[0xE0,0xDD])]
    KcApps,
    #[scan(down=[0xE0,0x5E],up=[0xE0,0xDE])]
    KcAcpiPower,
    #[scan(down=[0xE0,0x5F],up=[0xE0,0xDF])]
    KcAcpiSleep,
    #[scan(down=[0xE0,0x63],up=[0xE0,0xE3])]
    KcAcpiWake,
    #[scan(down=[0xE0,0x65],up=[0xE0,0xE5])]
    KcMultiMediaWwwSearch,
    #[scan(down=[0xE0,0x66],up=[0xE0,0xE6])]
    KcMultiMediaWwwFavorites,
    #[scan(down=[0xE0,0x67],up=[0xE0,0xE7])]
    KcMultiMediaWwwRefresh,
    #[scan(down=[0xE0,0x68],up=[0xE0,0xE8])]
    KcMultiMediaWwwStop,
    #[scan(down=[0xE0,0x69],up=[0xE0,0xE9])]
    KcMultiMediaWwwForward,
    #[scan(down=[0xE0,0x6A],up=[0xE0,0xEA])]
    KcMultiMediaWwwBack,
    #[scan(down=[0xE0,0x6B],up=[0xE0,0xEB])]
    KcMultiMediaMyComputer,
    #[scan(down=[0xE0,0x6C],up=[0xE0,0xEC])]
    KcMultiMediaEmail,
    #[scan(down=[0xE0,0x6D],up=[0xE0,0xED])]
    KcMultiMediaMediaSelect,
    #[scan(down=[0xE0,0x2A,0xE0,0x37],up=[0xE0,0xB7,0xE0,0xAA])]
    KcPrintScreen,
    #[scan(down=[0xE1,0x1D,0x45,0xE1,0x9D,0xC5],up=[])]
    KcPause,
}
