use core::arch::asm;

use common::{
    config::STAGE_2_START,
    gdt::{Gdt, GdtPointer},
    println_bios,
};

static GDT_POINTER: GdtPointer = GdtPointer::new(&GDT_PROTECTED, Gdt::NUM_ENTRIES as u16);

static GDT_LONG: Gdt = Gdt::long_mode();
static GDT_PROTECTED: Gdt = Gdt::protected_mode();

#[inline(never)]
pub fn test_unreal() {
    let ptr = 0x10_0000 as *mut u8;
    unsafe {
        *&mut *ptr = 123;
    }

    unsafe {
        let val = ptr.read();
        println_bios!("ptr = {}", val);
    }
}

pub fn enter_unreal() {
    let ds: u16;
    let ss: u16;

    unsafe {
        asm!(
            "mov {0:x}, ds",
            "mov {1:x}, ss",
            out(reg) ds,
            out(reg) ss,
            options(readonly, nostack, preserves_flags)
        );

        GDT_PROTECTED.disable_cli_and_load();
        let cr0 = set_protected_flag();

        asm!(
            "mov {0:x}, 0x10",
            "mov ds, {0}",
            "mov ss, {0}",
            out(reg) _
        );

        write_cr0(cr0);

        asm!(
            "mov ds, {0:x}",
            "mov ss, {1:x}",
            in(reg) ds,
            in(reg) ss,
            options(nostack, preserves_flags)
        );
    }

    todo!()
}

/// Disable interrupts
///
/// SAFETY: This has concequences that can effect things in unexpected ways, use with care and
/// understanding
pub(crate) unsafe fn disable_interrupts() {
    unsafe {
        asm!(
            // This clears inturrupts, this should be right next to lgdt. If you move it it can
            // cause undefined behavior
            "cli",
            options(readonly, nostack, preserves_flags)
        );
    }
}

/// Loads the GDT for protected mode
///
/// SAFETY: Interrupts should be disabled before calling
pub unsafe fn load_protected_gdt() {
    unsafe {
        GDT_PROTECTED.disable_cli_and_load();
    }
}

/// Sets the protected flag and returns previous cr0 value
pub(crate) unsafe fn set_protected_flag() -> u32 {
    let mut cr0: u32;
    unsafe {
        asm!(
            "mov {:e}, cr0", // Set protection enable bit
            out(reg) cr0,
            options(nomem,nostack,preserves_flags),
        );
    }
    let cr0_protected = cr0 | 1;
    write_cr0(cr0_protected);
    cr0
}

fn write_cr0(cr0: u32) {
    unsafe {
        asm!(
            "mov cr0, {:e}",
            in(reg) cr0,
            options(nomem,nostack,preserves_flags)
        );
    }
}

pub(crate) unsafe fn jump_next_stage() {
    // Perform long jump
    unsafe {
        asm!(
            // align the stack
            // "mov esp, ebp",
            "and esp, 0xffffff00",
            // push entry point address
            "push {entry_point:e}",
            entry_point = in(reg) STAGE_2_START as u32,
        );
        // println_vbe!("DONE");
        // TODO: Something seems to be broken with this
        // Perform a "long jump" to one line down.
        asm!(
            // The code segment index is 1, so to get the selector we get
            // 0x08 = 1 << 3
            "ljmp $0x08, $2f",
            // Relative label that we jump to
            "2:",
            options(att_syntax, nostack)
        );
        asm!(
            ".code32",
            // reload segment registers
            // The data segment index is 2, so to get the selector we get
            // 0x10 = 2 << 3
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            // Address for entry point
            "pop ax",
            // Clear stack
            // "mov sp, bp",
            // Enter stage 2
            "call ax",
            // enter endless loop in case stage-2 returns
            "2:",
            "jmp 2b",
            options(noreturn)
        );
    }
}
