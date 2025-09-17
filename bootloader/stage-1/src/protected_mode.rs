use core::{arch::asm, ptr::addr_of};

use common::{
    config::STAGE_2_START,
    gdt::{Gdt, GdtPointer},
    static_items::static_variable::StaticVariable as _,
};

pub(crate) static mut GDT_POINTER: GdtPointer = GdtPointer::null();

pub(crate) unsafe fn disable_inturrupts() {
    unsafe {
        asm!("cli", options(readonly, nostack, preserves_flags));
    }
}

pub(crate) unsafe fn load_protected_gdt() {
    unsafe {
        let gdt_addr = {
            Gdt::init();
            let gdt = Gdt::get_mut();
            *gdt = Gdt::protected_mode();
            gdt as *const Gdt
        };
        GDT_POINTER = GdtPointer::new(
            gdt_addr,
            (Gdt::NUM_ENTRIES * size_of::<u64> as u16 - 1) as u16,
        );
        asm!(
            "lgdt [{}]",
             in(reg) addr_of!(GDT_POINTER),
             options(readonly, nostack, preserves_flags)
        );
    }
}

pub(crate) unsafe fn set_protected_flag() {
    unsafe {
        let mut cr0: u32;
        asm!(
            "mov {:e}, cr0", // Set protection enable bit
            out(reg) cr0,
            options(nomem,nostack,preserves_flags),
        );

        let cr0_protected = cr0 | 1;

        asm!(
            "mov cr0, {:e}",
            in(reg) cr0_protected,
            options(nostack,preserves_flags)
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
            "mov sp, bp",
            // Enter stage 2
            "call ax",
            // enter endless loop in case stage-2 returns
            "2:",
            "jmp 2b",
            options(noreturn)
        );
    }
}
