use core::arch::asm;

use common::config::MEMORY_MAP_START;
use common::println_bios;
use common::static_string::StaticString;
use common::{gdt::*, println_vbe};
use common::{print_bios, print_vbe};

use common::io::bios::{print_char, print_chars, print_hex, print_hex32};

/// Detects memory using int 0x15 with eax = 0xE820, returns number of entries read
pub unsafe fn detect_memory() -> u16 {
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
