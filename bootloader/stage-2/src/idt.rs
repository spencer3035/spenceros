use core::{
    arch::asm,
    ptr::{addr_of, addr_of_mut},
};

use common::println_vbe;

struct IdtEntry(u64);

#[repr(C, packed)]
struct IdtDescriptor {
    offset: u32,
    size: u16,
}

impl IdtDescriptor {
    const fn null() -> Self {
        Self::new(0, 0)
    }
    const fn new(offset: u32, size: u16) -> Self {
        Self { offset, size }
    }
}

#[allow(dead_code)]
#[repr(u8)]
enum GateType {
    TaskGate = 0x5,
    IntGate16Bit = 0x6,
    TrapGate16Bit = 0x7,
    IntGate32Bit = 0xe,
    TrapGate32Bit = 0xf,
}

#[allow(dead_code)]
#[repr(u8)]
enum Privlidge {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
}

impl IdtEntry {
    const fn null() -> Self {
        Self(0)
    }
    fn trap() -> Self {
        let addr = dummy_handler as *const () as u32;
        let selector = 0x8;
        Self::new(addr, selector, Privlidge::Ring0, GateType::TrapGate32Bit)
    }
    fn fallback(gate_type: GateType) -> Self {
        let addr = dummy_handler as *const () as u32;
        let selector = 0x8;
        Self::new(addr, selector, Privlidge::Ring0, gate_type)
    }
    const fn new(offset: u32, selector: u16, privledge: Privlidge, gate_type: GateType) -> Self {
        let offset = if matches!(gate_type, GateType::TaskGate) {
            // Offset should be 0 of the type is taskgate
            0
        } else {
            offset
        };

        let mut mask: u64 = 0;
        // Bits 63 .. 48 upper offset (mask counts as shifting by 16 bits)
        mask |= ((offset & 0xFFFF0000) as u64) << 32;
        // Bits 47 .. 47 present bit
        mask |= 1 << 47;
        // Bits 46 .. 45 privlidge level
        mask |= ((privledge as u8 & 0b11) as u64) << 45;
        // Bits 44 .. 44 zero/unused
        // Bits 43 .. 40 gate type
        mask |= (((gate_type as u8) & 0b1111) as u64) << 40;
        // Bits 39 .. 32 reserved
        // Bits 31 .. 16 segment selector
        mask |= (selector as u64) << 16;
        // Bits 15 .. 0 lower offset
        mask |= (offset & 0xFFFF) as u64;

        Self(mask)
    }
}

static mut IDT_TABLE: [IdtEntry; 256] = [const { IdtEntry::null() }; 256];
static mut IDT_DESCRIPTOR: IdtDescriptor = IdtDescriptor::null();

pub fn setup_idt() {
    println_vbe!("Setting up IDT");

    let idt = unsafe {
        let size = 13 - 1;
        let offset = addr_of_mut!(IDT_TABLE);
        IDT_DESCRIPTOR = IdtDescriptor::new(offset as u32, size);
        offset.as_mut().unwrap()
    };

    idt[0] = IdtEntry::trap();
    idt[1] = IdtEntry::trap();
    idt[2] = IdtEntry::trap();
    idt[3] = IdtEntry::trap();
    idt[4] = IdtEntry::trap();
    idt[5] = IdtEntry::trap();
    idt[6] = IdtEntry::trap();
    idt[7] = IdtEntry::trap();
    idt[8] = IdtEntry::trap();
    idt[9] = IdtEntry::trap();
    idt[10] = IdtEntry::trap();
    idt[11] = IdtEntry::trap();
    idt[12] = IdtEntry::trap();
    idt[13] = IdtEntry::trap();

    // Rust gets rid of this call? I don't understand
    unsafe {
        asm!(
        "lidt [{}]",
        in(reg) addr_of!(IDT_DESCRIPTOR)
        );
    }
    loop {}

    println_vbe!("Throwing interrupt");
    test_div();
}

pub fn test_div() {
    unsafe {
        asm!("mov cx, 0", "div cx");
    }
}

pub extern "C" fn dummy_handler() {
    // TODO: This won't handle the stack properly, need to look over how this compiles closely
    unsafe {
        asm!("pushad");
    }
    println_vbe!("Handling interrupt");
    unsafe {
        asm!("popad", "iret");
    }
}
