use core::{arch::asm, ptr::addr_of};

use common::{
    gdt::{Gdt, GdtPointer},
    static_items::static_variable::StaticVariable as _,
};

static mut GDT_POINTER: GdtPointer = GdtPointer::null();

/// Disables interrupts and loads GDT
#[inline(always)]
pub unsafe fn load_gdt() {
    // Setup protected mode
    let gdt_addr = unsafe {
        Gdt::init();
        let gdt = Gdt::get_mut();
        *gdt = Gdt::protected_mode();
        gdt as *const Gdt
    };

    unsafe {
        GDT_POINTER = GdtPointer::new(gdt_addr, Gdt::NUM_ENTRIES - 1);
    };
    unsafe {
        asm!("lgdt [{}]", in(reg) addr_of!(GDT_POINTER), options(readonly, nostack, preserves_flags));
    }
    unsafe {
        asm!(
            "cli",          // Disable inturrupts
            "mov eax, cr0", // Set protection enable bit
            "or eax, 1",
            "mov cr0, eax",
        );
    }
}
