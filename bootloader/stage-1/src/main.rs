#![no_std]
#![no_main]

use core::arch::asm;

use common::*;
pub mod gdt;
use gdt::*;

#[repr(packed)]
struct GdtPointer {
    size: u16,
    location: u32,
}

const GDT_POINTER: *mut GdtPointer = 0x80 as *mut GdtPointer;
const GDT_START: *mut u8 = 0x100 as *mut u8;

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) {
    println(b"Starting Real Mode");

    // TODO:
    // Check CPUID avaliable (for checking 64 bit mode)
    // Check 64 bit mode avaliable
    // Load GDT
    // Enter Long Mode directly
    if has_cpuid() {
        println(b"Has CPUID");
    } else {
        println(b"Doesn't have CPUID");
        hlt();
    }

    println(b"Writing GDT");
    write_protected_gdt();

    // Setup protected mode
    unsafe {
        asm!(
            "cli", // Disable inturrupts
            "lgdt [{gdt_location}]", // Load GDT
            "mov eax, cr0", // Set protection enable bit
            "or eax, 1",
            "mov cr0, eax",
            gdt_location = in(reg) GDT_POINTER
        );
    }
    //unsafe {
    //    asm!("mov ah, 0x0f", "mov al, 'f'", "mov [0xb8000], ax");
    //}

    // Perform long jump
    unsafe {
        let entry_point = 0x7c00 + 0x600;
        let val = 69;
        asm!(
            // align the stack
            "and esp, 0xffffff00",
            // push arguments
            "push {info:e}",
            // push entry point address
            "push {entry_point:e}",
            info = in(reg) val as u32,
            entry_point = in(reg) entry_point as u32,
        );
        // What does this do? I Think it just does a "long jump" one line down.
        asm!(
            // Perform long jump to next address? How do we know this is sector 0x8?
            // Also what is 2f? Should it be interpreted as 0x2f? Changing it to that breaks things
            "ljmp $0x8, $2f",
            // Label? How does this not conflict with below
            "2:",
            // Why do we need att_syntax?
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

            // enter endless loop in case third stage returns
            "2:",
            "jmp 2b",
            out(reg) _,
            out(reg) _,
        );
    }
    loop {}

    //println(b"Done");

    //if has_long_mode() {
    //    println(b"Has long mode");
    //} else {
    //    println(b"Doesn't have long mode");
    //}

    //hlt();
}

/// Writes GDT and retuns number of bytes written
fn write_protected_gdt() -> usize {
    let gdt_code = GdtEntry::new(0, u32::MAX, kernel_code_flags(), extra_flags_protected());
    let gdt_data = GdtEntry::new(0, u32::MAX, kernel_data_flags(), extra_flags_protected());

    let mut gdt_bytes = 0;
    for byte in GdtEntry::null()
        .bytes()
        .iter()
        .chain(gdt_code.bytes().iter())
        .chain(gdt_data.bytes().iter())
    {
        unsafe {
            GDT_START.add(gdt_bytes).write(*byte);
        }
        gdt_bytes += 1;
    }

    // Align to 4 byte boundary (Assumed GDT_START is on a 4 byte boundary)
    let mut offset = gdt_bytes;
    let gdt_bytes: u16 = gdt_bytes as u16;
    let gdt_ptr = GdtPointer {
        size: gdt_bytes - 1,
        location: GDT_START as u32,
    };

    unsafe {
        GDT_POINTER.write(gdt_ptr);
    }
    if offset % 4 != 0 {
        offset += 4 - offset % 4;
    }

    offset
}

// Uses CPUID to check for long mode
//fn has_long_mode() -> bool {
//    let eax: u16;
//    unsafe {
//        asm!(
//            "mov eax, 0x80000000",
//            "cpuid",
//            out("eax") eax
//        );
//    }
//
//    if eax < 0x80000001 {
//        println(b"No long mode feature");
//        return false;
//    }
//
//    let edx: u16;
//    unsafe {
//        asm!(
//            "mov eax, 0x80000001",
//            "cpuid",
//            out("edx") edx
//        );
//    }
//
//    edx & 1 << 29 != 0
//}

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
