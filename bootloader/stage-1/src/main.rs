#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(const_trait_impl)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::arch::asm;

use common::gdt::*;
use common::*;
use real_mode::hlt;

static GDT_PROTECTED: Gdt = Gdt::protected_mode();

use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC: {info}");
    hlt();
}

use vbe::enter_vbe_mode;
pub mod vbe;

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

    let count = unsafe { detect_memory() };
    unsafe {
        enter_vbe_mode();
    }

    panic!("Not ready for next stage");
    unsafe {
        load_gdt();
        next_stage(count);
    }
    panic!("Returned back to stage 1");
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
                "jc fail_asm",
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

#[inline(always)]
unsafe fn enable_a20() {
    // enable A20-Line via IO-Port 92, might not work on all motherboards
    // Check if A20 is enabled
    let al: u8;
    unsafe {
        asm!(
            "in {al}, 0x92",
            //"test al, 2",
            al = out(reg_byte) al
        );
    }

    if al != 2 {
        //println(b"A20 already enabled");
        return;
    }

    // Enable a20
    unsafe {
        asm!("or al, 2", "and al, 0xFE", "out 0x92, al",);
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
