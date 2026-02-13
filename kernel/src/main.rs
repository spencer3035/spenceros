// #![no_std]
// #![no_main]

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use bootloader_api::BootInfo;
use core::fmt::Write;

use crate::{
    alloc::BUDDY_ALLOCATOR,
    framebuffer::{FrameBuffer, FrameBufferDisplay},
    mem::PhysicalMemoryRegion,
    mutex::Mutex,
};

pub mod alloc;
pub mod font;
pub mod framebuffer;
pub mod gdt;
pub mod keyboard_ps_2;
pub mod mem;
pub mod mutex;
pub mod prealloc_array;

#[macro_use]
pub mod loggers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::{nop, port::Port};

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        nop();
    }
}

pub fn serial() -> uart_16550::SerialPort {
    let mut port = unsafe { uart_16550::SerialPort::new(0x3F8) };
    port.init();
    port
}

#[allow(dead_code)]
pub struct MyBootInfo {
    framebuffer: Option<FrameBufferDisplay>,
    port: Option<uart_16550::SerialPort>,
    kernel_addr: u64,
    kernel_len: u64,
    mem_info: mem::MemInfo,
}

impl MyBootInfo {
    fn init_loggers(&mut self) {
        loggers::init_loggers(self);
    }
}

#[derive(Debug)]
pub enum BiosInfoError {
    #[allow(dead_code)]
    NotEnoughBytesPerPixel(u8),
    NoFramebufferFound,
    TooManyMemEntries,
}

fn parse_boot_info(info: &'static mut BootInfo) -> Result<MyBootInfo, BiosInfoError> {
    let mut port = serial();
    writeln!(port, "Bios info: {info:#X?}").unwrap();

    let fb = info.framebuffer.take().unwrap();
    let mut fb = framebuffer::FrameBufferDisplay::new(fb)?;
    fb.clear();
    // TODO: Check out workings of memory detection in BIOS. It doesn't seem to be detecting all
    // the avaliable memory
    let mem_info = mem::parse_mem_info(info)?;

    Ok(MyBootInfo {
        framebuffer: Some(fb),
        port: Some(port),
        kernel_addr: info.kernel_addr,
        kernel_len: info.kernel_len,
        mem_info,
    })
}

#[cfg(not(test))]
bootloader_api::entry_point!(kernel_main);
#[allow(unused)]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let info = parse_boot_info(boot_info).unwrap();
    main_inner(info);
    exit_qemu(QemuExitCode::Success);
}

fn main_inner(mut info: MyBootInfo) {
    info.init_loggers();
    let mut kb = keyboard_ps_2::KeyboardDriver::new();

    loggers::println!("test");

    let mut buddy_alloc = BUDDY_ALLOCATOR.lock();

    println!("Memory entries:");
    let mut prev = None;
    for entry in info.mem_info.iter() {
        println_port!("{entry:X?}");

        if let Some(prev) = prev
            && entry.start != prev
        {
            // Address range not accessable
            let start = prev as usize;
            let end = entry.start as usize;
            let size = end - start;
            // println_port!("Reserving unaccessable 0x{start:X}, 0x{end:X}");
            buddy_alloc.reserve_addr(start, size);
        }

        if !entry.usable {
            let start = entry.start as usize;
            let size = entry.len() as usize;
            // let end = entry.end;
            // println_port!("Reserving unusable 0x{start:X}, 0x{end:X}");
            buddy_alloc.reserve_addr(start, size);
        }
        prev = Some(entry.end);
    }

    println!("GDT: {:#X?}", gdt::GDT);

    println!("Press any key to continue:");
    let _ = kb.next_keypress();

    // loop {
    //     let key = kb.next_keypress();
    //     if key.is_backspace() {
    //         fb.backspace().unwrap();
    //     } else if key.is_enter() {
    //         writeln!(fb).unwrap();
    //     } else if key.is_char() {
    //         if let Some(ch) = kb.modify_key_to_char(key) {
    //             write!(fb, "{ch}").unwrap();
    //         }
    //     } else if key == KeyCode::KcEsc {
    //         break;
    //     } else {
    //         writeln!(fb, "not sure what to do with {key:?}").unwrap();
    //     }
    // }

    // writeln!(port, "\nDone\n").unwrap();
}

const IDT_SIZE: usize = 16;

#[repr(C, packed)]
pub struct IdtDescriptor {
    size: u16,
    offset: u64,
}

static IDT_DESCRIPTOR: IdtDescriptor = IdtDescriptor {
    size: IDT_SIZE as u16,
    offset: 0,
};

static IDT: [IdtEntry; IDT_SIZE] = [const { IdtEntry::null() }; IDT_SIZE];

#[repr(C)]
pub struct IdtEntry {
    // The lower 16 bits of the ISR's address
    isr_low: u16,
    // The GDT segment selector that the CPU will load into CS before calling the ISR
    kernel_cs: u16,
    // The IST in the TSS that the CPU will load into RSP; set to zero for now
    ist: u8,
    // Type and attributes; see the IDT page
    attributes: u8,
    // The higher 16 bits of the lower 32 bits of the ISR's address
    isr_mid: u16,
    // The higher 32 bits of the ISR's address
    isr_high: u32,
    // Set to zero
    reserved: u32,
}

impl IdtEntry {
    const fn null() -> Self {
        Self {
            isr_low: 0,
            kernel_cs: 0,
            ist: 0,
            attributes: 0,
            isr_mid: 0,
            isr_high: 0,
            reserved: 0,
        }
    }
}

// static IDT : [IdtEntry: 256] = [];

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(serial(), "PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
