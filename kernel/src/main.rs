// #![no_std]
// #![no_main]

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use bootloader_api::BootInfo;
use core::{cell::UnsafeCell, fmt::Write, marker::PhantomData, sync::atomic::AtomicBool};

use crate::framebuffer::{FrameBuffer, FrameBufferDisplay};

pub mod alloc;
pub mod font;
pub mod framebuffer;
pub mod keyboard_ps_2;
pub mod mem;
pub mod mutex;
pub mod prealloc_array;

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
struct MyBootInfo {
    framebuffer: Option<FrameBufferDisplay>,
    port: Option<uart_16550::SerialPort>,
    kernel_addr: u64,
    kernel_len: u64,
    mem_info: mem::MemInfo,
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
    let mut fb = info.framebuffer.take().unwrap();
    let mut port = info.port.take().unwrap();
    let mut kb = keyboard_ps_2::KeyboardDriver::new();

    writeln!(port, "Memory entries:").unwrap();
    for entry in info.mem_info.iter() {
        writeln!(port, "{entry:X?}").unwrap();
    }
    writeln!(port, "kernel addr: 0x{:X}", info.kernel_addr).unwrap();

    writeln!(fb, "Press any key to continue:").unwrap();
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

    writeln!(port, "\nDone\n").unwrap();
}

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

// static IDT : [IdtEntry: 256] = [];

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(serial(), "PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
