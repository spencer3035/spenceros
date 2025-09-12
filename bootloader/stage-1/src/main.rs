#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(const_trait_impl)]
#![deny(unsafe_op_in_unsafe_fn)]
// TODO: Remove
#![allow(unused)]

use core::arch::asm;

use common::config::{MEMORY_MAP_START, STACK_END, STACK_START};
use common::gdt::*;

static GDT_PROTECTED: Gdt = Gdt::protected_mode();

macro_rules! println {
    ($($args:tt)*) => {
            common::println_bios!($($args)*)
    };
}

#[allow(unused_macros)]
macro_rules! print {
    ($($args:tt)*) => {
        common::print_bios!($($args)*);
    };
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {info}");
    loop {
        unsafe { asm!("hlt") }
    }
}

use common::io::bios::{print_char, print_chars, print_hex, print_hex32};
use vbe::init_graphical;
pub mod vbe;

#[inline(always)]
fn print_stack_used() {
    let mut sp: u32;
    unsafe {
        asm!("mov {:e}, esp",  out(reg) sp);
    }
    // let total = STACK_END as u32 - STACK_START as u32;
    let used = STACK_END as u32 - sp;
    println!("USED: 0x{used:X}");
}

fn print_fn_location(f: fn()) {
    let addr: usize = f as *const () as usize;
    println!("fn: 0x{addr:X}");
}
fn poll_keypress() -> Option<char> {
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
    if ch != '\0' && ch.is_ascii_control() {
        // println!("CONTROL: {}", ch.escape_default());
    }
    if key_present == 0 {
        None
    } else {
        Some(next_keypress())
    }
}

fn next_keypress() -> char {
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
    if ch == '\r' {
        '\n'
    } else {
        ch
    }
}

struct StringOverflow;

impl core::fmt::Display for StringOverflow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "String Overflow, max size = {}", STATIC_STRING_LENGTH)
    }
}

const STATIC_STRING_LENGTH: usize = 255;
struct StaticString {
    chars: [char; STATIC_STRING_LENGTH],
    len: usize,
}

impl StaticString {
    const fn new() -> Self {
        StaticString {
            chars: ['\0'; STATIC_STRING_LENGTH],
            len: 0,
        }
    }

    /// Returns error if overflow and the number of characters left if it succeeds
    fn push(&mut self, ch: char) -> Result<usize, StringOverflow> {
        if self.len >= STATIC_STRING_LENGTH {
            Err(StringOverflow)
        } else {
            self.chars[self.len] = ch;
            self.len += 1;
            Ok(STATIC_STRING_LENGTH - self.len)
        }
    }

    fn clear(&mut self) {
        self.len = 0;
    }
}

impl core::fmt::Display for StaticString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.len > 0 {
            for ch in self.chars[0..self.len].iter() {
                write!(f, "{}", ch)?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

fn prompt_continue() -> bool {
    print!("Continue (y/n)? ");
    let ch = next_keypress();
    ch == 'y'
}

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) {
    println!("Starting stage 1");

    unsafe {
        enable_a20();
    }

    if !has_cpuid() {
        panic!("CPUID not present");
    }

    let mut s = StaticString::new();

    while !prompt_continue() {}

    println!("DONE");

    loop {}

    // let count = unsafe { detect_memory() };
    // init_graphical();
    // panic!("Not ready for next stage");
    // loop {}
    // unsafe {
    //     load_gdt();
    //     next_stage(count);
    // }
}

/// Detects memory using int 0x15 with eax = 0xE820, returns number of entries read
unsafe fn detect_memory() -> u16 {
    let int15_ax = 0xE820;
    let magic_number = 0x534d4150;
    let mem_address: u16 = MEMORY_MAP_START as u16;

    // Registers
    let mut eax = int15_ax;
    let mut di = mem_address;
    let mut ebx = 0;
    let mut ecx = 24;
    let mut edx = magic_number;

    let mut count = 0;
    loop {
        count += 1;
        unsafe {
            asm!(
                "int 0x15",
                // TODO: Put this check outside and remove "fail_asm" call. This doesn't even work
                // really
                "jnc 2f",
                "mov eax, 1",
                "2:",
                // https://wiki.osdev.org/Detecting_Memory_(x86)#BIOS_Function:_INT_0x15,_EAX_=_0xE820
                // If success:
                // Carry is clear
                // check EAX is magic number
                // EBX is nonzero, should be preserved to next call
                // CL has number of bytes stored (probably 20)
                // If end:
                // ebx == 0 or carry flag is set
                inout("di") di,
                inout("eax") eax,
                inout("ebx") ebx,
                inout("ecx") ecx,
                inout("edx") edx,
            );
        }

        if eax != magic_number {
            panic!("bad eax mem");
        }

        if ecx != 20 {
            panic!("mem offset bad");
        }

        // TODO: Also check carry is clear
        if ebx == 0 {
            break;
        }

        di += 24;
        eax = int15_ax;
        ecx = 24;
    }

    // Reference: https://wiki.osdev.org/Detecting_Memory_(x86)
    // TODO: Increment di, reset eax and ecx, until ebx==0 or carry is set
    count
}

unsafe fn next_stage(count: u16) {
    // Perform long jump
    unsafe {
        let entry_point = 0x7c00 + 0x600;
        asm!(
            // align the stack
            "and esp, 0xffffff00",
            // push arguments
            "push {info:e}",
            // push entry point address
            "push {entry_point:e}",
            info = in(reg) count as u32,
            entry_point = in(reg) entry_point as u32,
        );
        // Perform a "long jump" to one line down.
        asm!(
            // TODO: How do we know this is sector 0x8?
            // Note that 2f means jump (f)orward to the next local label "2:"
            "ljmp $0x08, $2f",
            // Relative label that we jump to
            "2:",
            options(att_syntax)
        );
        asm!(
            ".code32",

            // reload segment registers
            "mov {0}, 0x10",
            "mov ds, {0}",
            "mov es, {0}",
            "mov ss, {0}",

            // jump to stage-2
            "pop {1}",
            "call {1}",

            // enter endless loop in case stage-2 returns
            "2:",
            "jmp 2b",
            out(reg) _,
            out(reg) _,
        );
    }
}

/// Disables interrupts and loads GDT
#[inline(always)]
unsafe fn load_gdt() {
    // Setup protected mode
    GDT_PROTECTED.load();
    unsafe {
        asm!(
            "cli",          // Disable inturrupts
            "mov eax, cr0", // Set protection enable bit
            "or eax, 1",
            "mov cr0, eax",
        );
    }
}

/// Check if A20 is enabled
#[inline(always)]
unsafe fn enable_a20() {
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
        println!("A20 already enabled");
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
fn has_cpuid() -> bool {
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
