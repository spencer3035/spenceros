use core::arch::asm;

use common::config::STACK_END;
use common::println_bios;
use common::{gdt::*, println_vbe};
use common::{print_bios, print_vbe};

use common::config::STACK_START;
use common::io::bios::{print_char, print_chars, print_hex, print_hex32};

// TODO: Make sure correct println is used
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use common::io::framebuffer::VbeDisplay;

    if VbeDisplay::is_init() {
        println_vbe!("PANIC: {info}");
    } else {
        println_bios!("PANIC: {info}");
    }
    loop {
        unsafe { asm!("hlt") }
    }
}

pub fn prompt_continue() {
    loop {
        print_bios!("Continue (y/n)? ");
        let ch = next_keypress();
        println_bios!("{ch}");
        if ch == 'y' {
            break;
        }
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
        asm!("mov {sp}, esp",  sp = out(reg_abcd) sp);
    }
    STACK_END as u32 - sp
}

#[inline(always)]
pub fn get_stack_left() -> u32 {
    let used = get_stack_used();
    let total = STACK_END as u32 - STACK_START as u32;
    total - used
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
    let ch = (ax & 0xFF) as u8 as char;
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
    let ch = (ax & 0xFF) as u8 as char;
    ch
}
