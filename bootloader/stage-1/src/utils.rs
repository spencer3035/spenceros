use core::arch::asm;

use common::config::STACK_END;
use common::io::framebuffer::VbeDisplay;
use common::{print_bios, print_vbe};
use common::{println_bios, println_vbe};

use common::config::STACK;

// TODO: Make sure correct println is used
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use common::println_vbe;

    match VbeDisplay::is_init() {
        true => {
            println_vbe!("PANIC: {info}");
        }
        false => {
            println_bios!("PANIC: {info}");
        }
    }
    loop {
        unsafe { asm!("hlt") }
    }
}

pub fn prompt_continue() {
    loop {
        if VbeDisplay::is_init() {
            print_vbe!("Continue (y/n)? ");
        } else {
            print_bios!("Continue (y/n)? ");
        }
        let ch = next_keypress();
        if VbeDisplay::is_init() {
            println_vbe!("{ch}\n");
        } else {
            println_bios!("{ch}\n");
        }
        if ch == 'y' {
            break;
        }
    }
}

/// This tells the bios that we are targeting long mode (64 bit) as our primary mode of operation.
/// Has no other side effects
pub fn hint_bios_long_mode() {
    // This is a pretty poorly document INT
    // See: https://f.osdev.org/viewtopic.php?f=1&t=20445&start=0
    unsafe {
        asm!(
            // Which variant of int0x15
            "mov ax, 0xec00",
            // 2 is long mode
            "mov bl, 0x02",
            // Do INT
            "int 0x15"
        );
    }
}

/// Check if A20 is enabled
pub fn enable_a20() {
    // enable A20-Line via IO-Port 92, might not work on all motherboards
    let al: u8;
    unsafe {
        asm!(
            "in {al}, 0x92",
            //"test al, 2",
            al = out(reg_byte) al
        );
    }

    if al != 2 {
        println_bios!("A20 already enabled");
        return;
    }

    // Enable a20
    unsafe {
        asm!(
        "or {al}, 2",
        "and al, 0xFE",
        "out 0x92, al",
        al = in(reg_byte) al
        );
    }
}

/// Checks if CPUID exists or not
#[inline(always)]
pub fn has_cpuid() -> bool {
    let has_id: u16;
    unsafe {
        asm!(
        // Set bit 21
        "pushfd",
        "pop eax",
        "mov ecx, eax",
        "xor eax, 1 << 21",
        "push eax",
        "popfd",

        // Check bit 21 is set
        "pushfd",
        "pop eax",
        "xor eax, ecx",
        "shr eax, 21",
        "and eax, 1",
        out("eax") has_id
        );
    }

    has_id != 0
}

#[inline(always)]
pub fn get_stack_used() -> u32 {
    let mut sp: u32;
    unsafe {
        asm!("mov {sp:e}, esp",  sp = out(reg_abcd) sp);
    }
    STACK_END as u32 - sp
}

#[inline(always)]
pub fn get_stack_left() -> u32 {
    let used = get_stack_used();
    let total = STACK_END as u32 - STACK as u32;
    total - used
}

pub fn print_unsafe_fn_location(f: unsafe fn()) {
    let addr: usize = f as *const () as usize;
    println_bios!("fn: 0x{addr:X}");
}

pub fn print_fn_location(f: fn()) {
    let addr: usize = f as *const () as usize;
    println_bios!("fn: 0x{addr:X}");
}

pub fn poll_keypress() -> Option<char> {
    // INT 16 ; AH = 1
    // OUT:
    // ZF set if no key pressed
    // ZF clear if key avaliable
    let mut ax: u16 = 0x0100;
    let mut key_present: u16 = 0;
    unsafe {
        asm!(
            "int 0x16",
            "jz 2f", // No key event
            "mov dx, 1",
            "2:",
            inout("ax") ax,
            inout("dx") key_present,
        );
    }
    let _ch = (ax & 0xFF) as u8 as char;
    if key_present == 0 {
        None
    } else {
        Some(next_keypress())
    }
}

pub fn next_keypress() -> char {
    // INT 16 ; AH = 0
    // OUT AL = ascii character
    let mut ax: u16 = 0x0000;
    unsafe {
        asm!(
            "int 0x16",
            inout("ax") ax,
        );
    }
    (ax & 0xFF) as u8 as char
}
