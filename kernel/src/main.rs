// #![no_std]
// #![no_main]

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use bootloader_api::{BootInfo, entry_point, info::MemoryRegionKind};
use core::{
    fmt::Write,
    ops::{Deref, DerefMut},
};

use crate::framebuffer::{FrameBuffer, FrameBufferDisplay};

pub mod font;
pub mod framebuffer;
pub mod keyboard_ps_2;
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

const MEM_ENTRY_MAX: usize = 100;

struct MyBootInfo {
    framebuffer: Option<FrameBufferDisplay>,
    port: Option<uart_16550::SerialPort>,
    kernel_addr: u64,
    kernel_len: u64,
    mem_info: MemInfo,
}

pub trait PhysicalMemoryInfo {
    fn entries(&self) -> impl Iterator<Item = &MemEntry>;
}

impl PhysicalMemoryInfo for MemInfo {
    fn entries(&self) -> impl Iterator<Item = &MemEntry> {
        self.iter()
    }
}

#[derive(Debug, Default)]
struct MemEntry {
    start: u64,
    end: u64,
    usable: bool,
}

impl MemEntry {
    const fn null() -> Self {
        Self {
            start: 0,
            end: 0,
            usable: false,
        }
    }
}

struct MemInfo {
    mem_entries: [MemEntry; MEM_ENTRY_MAX],
    mem_len: usize,
}

impl Deref for MemInfo {
    type Target = [MemEntry];

    fn deref(&self) -> &Self::Target {
        &self.mem_entries[0..self.mem_len]
    }
}

impl DerefMut for MemInfo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mem_entries[0..self.mem_len]
    }
}

fn parse_mem_info(info: &'static BootInfo) -> Result<MemInfo, BiosInfoError> {
    let mut arr = [const { MemEntry::null() }; MEM_ENTRY_MAX];
    let mut ii = 0;
    for entry in info.memory_regions.iter() {
        let is_usable = matches!(entry.kind, MemoryRegionKind::Usable);
        if !is_usable {
            // We only care about usable memory
            continue;
        }

        if ii > 0 {
            // Not first entry
            if entry.start == arr[ii - 1].end && arr[ii - 1].usable == is_usable {
                // Combine with previous entry
                arr[ii - 1].end = entry.end;
            } else {
                // Make new entry
                arr[ii].end = entry.end;
                arr[ii].start = entry.start;
                arr[ii].usable = is_usable;
                ii += 1;
            }
        } else {
            // First entry
            arr[ii].end = entry.end;
            arr[ii].start = entry.start;
            arr[ii].usable = is_usable;
            ii += 1;
        }

        if ii >= MEM_ENTRY_MAX {
            return Err(BiosInfoError::TooManyMemEntries);
        }
    }

    Ok(MemInfo {
        mem_entries: arr,
        mem_len: ii,
    })
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
    let mem_info = parse_mem_info(info)?;

    Ok(MyBootInfo {
        framebuffer: Some(fb),
        port: Some(port),
        kernel_addr: info.kernel_addr,
        kernel_len: info.kernel_len,
        mem_info,
    })
}

#[cfg(not(test))]
entry_point!(kernel_main);
#[allow(unused)]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let info = parse_boot_info(boot_info).unwrap();
    main_inner(info);
    exit_qemu(QemuExitCode::Success);
}

/// Maps physical memory addesses to a continuous range
pub struct PhysicalMemoryMapper<M>
where
    M: PhysicalMemoryInfo,
{
    memory_map: M,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddr(u64);
#[derive(Debug)]
pub struct VirtualAddr(u64);

impl<M> PhysicalMemoryMapper<M>
where
    M: PhysicalMemoryInfo,
{
    pub fn new(mem: M) -> Self {
        Self { memory_map: mem }
    }
    pub fn map(&self, addr: VirtualAddr) -> Option<PhysicalAddr> {
        #[cfg(test)]
        println!("Mapping {addr:?}");

        let mut prev_section_break = 0;
        for entry in self.memory_map.entries() {
            #[cfg(test)]
            dbg!(&entry, prev_section_break);
            let section_length = entry.end - entry.start;
            if prev_section_break <= addr.0 && addr.0 < prev_section_break + section_length {
                let phys_addr = addr.0 - prev_section_break + entry.start;
                return Some(PhysicalAddr(phys_addr));
            }
            #[cfg(test)]
            dbg!(section_length);
            prev_section_break += section_length;
        }

        None
    }
}

#[test]
fn test_phys_mem_map() {
    struct Info {
        entries: Vec<MemEntry>,
    }
    impl PhysicalMemoryInfo for Info {
        fn entries(&self) -> impl Iterator<Item = &MemEntry> {
            self.entries.iter()
        }
    }
    let info = Info {
        entries: vec![
            MemEntry {
                start: 100,
                end: 200,
                usable: true,
            },
            MemEntry {
                start: 400,
                end: 500,
                usable: true,
            },
            MemEntry {
                start: 600,
                end: 700,
                usable: true,
            },
        ],
    };

    let pmm = PhysicalMemoryMapper::new(info);

    assert_eq!(pmm.map(VirtualAddr(0)), Some(PhysicalAddr(100)));
    assert_eq!(pmm.map(VirtualAddr(100)), Some(PhysicalAddr(400)));
    assert_eq!(pmm.map(VirtualAddr(250)), Some(PhysicalAddr(650)));
    assert_eq!(pmm.map(VirtualAddr(500)), None);
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

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(serial(), "PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
