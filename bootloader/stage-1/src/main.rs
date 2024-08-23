#![no_std]
#![no_main]

use core::arch::asm;

use common::gdt::*;
use common::*;

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_disk_number: u16) {
    unsafe {
        enable_a20();
    }

    if !has_cpuid() {
        fail(b"Doesn't have CPUID");
    }

    let count = unsafe { detect_memory() };

    unsafe {
        write_protected_gdt();
        load_gdt();
        next_stage(count);
    }

    fail(b"return from protected");
}

/// Detects memory using int 0x15 with eax = 0xE820, returns number of entries read
unsafe fn detect_memory() -> u16 {
    let int15_ax = 0xE820;
    let magic_number = 0x534d4150;
    let mut mem_address: u16 = MEMORY_MAP_START as u16;

    // Registers
    let mut eax = int15_ax;
    let mut di = mem_address;
    let mut ebx = 0;
    let mut ecx = 24;
    let mut edx = magic_number;

    let mut count = 0;
    loop {
        count += 1;
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

        if eax != magic_number {
            fail(b"bad eax mem");
        }

        if ecx != 20 {
            fail(b"mem offset bad");
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
}

#[inline(always)]
unsafe fn enable_a20() {
    // enable A20-Line via IO-Port 92, might not work on all motherboards
    // Check if A20 is enabled
    let al: u8;
    asm!(
        "in {al}, 0x92",
        //"test al, 2",
        al = out(reg_byte) al
    );

    if al != 2 {
        //println(b"A20 already enabled");
        return;
    }

    // Enable a20
    asm!("or al, 2", "and al, 0xFE", "out 0x92, al",);
}

/// Writes GDT and retuns number of bytes written
unsafe fn write_protected_gdt() -> usize {
    let gdt_code = GdtEntry::new(0, u32::MAX, kernel_code_flags(), extra_flags_protected());
    let gdt_data = GdtEntry::new(0, u32::MAX, kernel_data_flags(), extra_flags_protected());

    // TODO: Switch to writing entries directly instead of looping over bytes
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
