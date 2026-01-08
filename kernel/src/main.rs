#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use bootloader_api::{BootInfo, entry_point};
use core::fmt::Write;

use crate::framebuffer::FrameBuffer as _;

pub mod font;
pub mod framebuffer;

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

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let mut port = serial();
    writeln!(port, "Entered kernel with boot info: {boot_info:#?}").unwrap();

    let mut fb = framebuffer::FrameBufferDisplay::new(boot_info).unwrap();

    for y in 0..fb.height() {
        for x in 0..fb.width() {
            fb.set_pixel(x, y, &framebuffer::Color::BLACK);
        }
    }

    for ii in 0..(fb.width().min(fb.height())) {
        match ii % 3 {
            0 => fb.set_pixel(ii, ii, &framebuffer::Color::RED),
            1 => fb.set_pixel(ii, ii, &framebuffer::Color::GREEN),
            2 => fb.set_pixel(ii, ii, &framebuffer::Color::BLUE),
            _ => unreachable!(),
        };
        // fb.set_pixel(ii, ii, &Color::WHITE);
    }

    // for ii in 0..fb.width() / 2 {
    //     for jj in 0..fb.height() / 2 {
    //         fb.set_pixel(ii, jj, &Color::WHITE);
    //     }
    // }

    // fb.write_char_impl('A').unwrap();
    // writeln!(fb, "from framebuffer").unwrap();

    // if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
    //     let FrameBufferInfo {
    //         byte_len,
    //         width,
    //         height,
    //         pixel_format,
    //         bytes_per_pixel,
    //         stride,
    //     } = framebuffer.info();
    //     let h = height / 2;
    //     let buf = framebuffer.buffer_mut();
    //     let start_off = h * stride;
    //     for w in 0..width {
    //         buf[start_off + w * bytes_per_pixel] = 0xFF;
    //     }
    // }

    writeln!(port, "\nDone\n").unwrap();
    use x86_64::instructions::nop;

    // unsafe {
    //     let mut port = Port::new(0xf4);
    //     port.write(exit_code as u32);
    // }

    loop {
        nop();
    }
}

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(serial(), "PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
