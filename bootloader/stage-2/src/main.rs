#![no_std]
#![no_main]

use common::*;
use core::arch::asm;

#[allow(dead_code)]
#[repr(packed)]
#[derive(Debug)]
struct MemoryMapEntry {
    base_address: u64,
    length: u64,
    region_type: u32,
    attributes: u32,
}

#[link_section = ".start"]
#[no_mangle]
pub extern "C" fn _start(_count: u16) -> ! {
    clear_screen();
    println!("Started protected mode");

    //let mut mmap_reader: *const MemoryMapEntry = MEMORY_MAP_START as *const MemoryMapEntry;
    //for ii in 0..count {
    //    unsafe {
    //        let val = mmap_reader.add(ii as usize).read();
    //        println!("{:#x?}", val);
    //    }
    //}

    // Note that CPUID functionality is checked in stage-1
    if has_long_mode() {
        println!("We have long mode");
    } else {
        println!("No long mode!");
        hlt();
    }

    unsafe {
        asm!("cli");
        println!("Setting up paging");
        setup_paging();
    }
    // Enter enable paging and enter 32 bit compatability submode of long mode
    {
        unsafe {
            asm!(
                // Set the C-register to the EFER Model Specific Register (MSR)
                "mov ecx, 0xc0000080",
                // Read from MSR
                "rdmsr",
                // Set the LM-bit
                "or eax, 1 << 8",
                // Write to MSR
                "wrmsr"
            );
        }
    }

    unsafe {
        write_long_mode_gdt();
    }

    // TODO: Load gdt and enter perform long jump to enter long mode
    unsafe {
        let entry_point =
            STAGE_0_START + (STAGE_0_SECTIONS + STAGE_1_SECTIONS + STAGE_2_SECTIONS) * 512;

        clear_screen();
        println!("In protected mode, about to enter long mode");

        asm!(
            "lgdt [{gdt_pointer}]",
            // Push value
            "push 0",
            "push '3'",
            // Push entry point
            "push 0",
            "push {entry_point:e}",
            gdt_pointer = in(reg) GDT_POINTER as u32,
            entry_point = in(reg) entry_point as u32,
        );

        // Perform a "long jump" to one line down.
        asm!(
            // TODO: How do we know this is sector 0x8?
            // Note that 2f means jump (f)orward to the next local label "2:"
            "ljmp $0x08, $2f",
            // Relative label that we jump to
            "2:",
            options(att_syntax)
        );

        asm!(
            ".code64",
            // Debug
            "mov ah, 0xf0",
            "mov al, '4'",
            "mov [0xb8000], ax",
            "2:",
            "jmp 2b",
            // End debug
        );

        asm!(
            ".code64",

            "mov ah, 0x0f",
            "mov al, '3'",
            "mov [0xb8000], ax",
            "2:",
            "jmp 2b",

            // reload segment registers
            "mov {0}, 0x10",
            "mov ds, {0}",
            "mov es, {0}",
            "mov ss, {0}",


            "2:",
            "jmp 2b",


            // jump to stage-3
            "pop rax",
            "pop rdi",
            "call rax",

            // enter endless loop in case stage-2 returns
            "2:",
            "jmp 2b",
            out(reg) _,
        );
    }
    hlt();
}

use common::gdt::*;

/// Writes GDT and retuns number of bytes written
unsafe fn write_long_mode_gdt() -> usize {
    let gdt_code = GdtEntry::new(0, u32::MAX, kernel_code_flags(), extra_flags_long());
    let gdt_data = GdtEntry::new(0, u32::MAX, kernel_data_flags(), extra_flags_protected());

    let mut gdt_bytes = 0;
    // TODO: Switch to writing entries directly instead of looping over bytes
    // TODO: Write TSS
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

#[allow(dead_code)]
fn print_memory_addresses(start: *const u8) {
    let size = 16;
    //let num_lines = 20;
    //for ii in 0..num_lines {
    //    let as_arr = core::slice::from_raw_parts(start, size);
    //    print!("0x{:04x}: ", start as u64);
    //    println!("{as_arr:02x?}");
    //    start = start.add(16);
    //}
    let as_arr = unsafe { core::slice::from_raw_parts(start, size) };
    print!("0x{:04x}: ", start as u64);
    println!("{as_arr:02x?}");
}

/// If paget is present in physical memory, else not present
const PRESENT: u64 = 1 << 0;
/// If the page is read/write, else read-only
const READ_WRITE: u64 = 1 << 1;
#[allow(dead_code)]
/// If the page can be accessed by all, else only superuser
const USER_SUPERUSER: u64 = 1 << 2;
#[allow(dead_code)]
/// If write through caching is enabled, else not
const WRITE_THROUGH: u64 = 1 << 3;
#[allow(dead_code)]
/// If caching is disabled, else it is enabled
const CACHE_DISABLE: u64 = 1 << 4;
#[allow(dead_code)]
/// If the page is being accessed or not
const ACCESSED: u64 = 1 << 5;

/// Sets up paging at the configured addresses
unsafe fn setup_paging() {
    // TODO: Read and understand paging better:
    // https://wiki.osdev.org/Setting_Up_Paging

    // Zero out addresses used for paging
    let starts = [PML4T_START, PDPT_START, PDT_START, PT_START];
    //for val in unsafe { PML4T_START.as_uninit_ref()}
    for start in starts {
        // There are 512 entries in 64 bit page
        // SAFETY: start is expected to be uninit, we init it here.
        let table: &mut [u64; 0x200] = unsafe { start.as_mut().unwrap() };
        for val in table.iter_mut() {
            // Read write, but not present (yet)
            *val = READ_WRITE;
        }
    }

    // Point first entry in PML4T, PDPT, and PDT to point to the only existing table
    unsafe {
        // This or operation works because the page tables must be 0x1000 aligned, so the last 12
        // bits of the address must be 0.
        PML4T_START.as_mut().unwrap()[0] = PDPT_START as u64 | READ_WRITE | PRESENT;
        PDPT_START.as_mut().unwrap()[0] = PDT_START as u64 | READ_WRITE | PRESENT;
        PDT_START.as_mut().unwrap()[0] = PT_START as u64 | READ_WRITE | PRESENT;
    }

    // Identity map the PT to the first 2 MB
    // SAFETY: PT_START is expected to be uninitalized, we initialize all the values here
    let pt: &mut [u64; 0x200] = unsafe { PT_START.as_mut().unwrap() };
    for ii in 0..0x200 {
        let addr: u64 = 0x1000 * (ii as u64);
        let flags: u64 = PRESENT | READ_WRITE;
        let entry = addr | flags;
        pt[ii] = entry;
    }

    // Enable paging
    unsafe {
        asm!(
            "mov eax, cr4",
            "or eax, 1 << 5", // PAE-bit is the 6th bit
            "mov cr4, eax",
        );
    }
}

// Uses CPUID to check for long mode
fn has_long_mode() -> bool {
    let eax: u32;
    unsafe {
        asm!(
            "mov eax, 0x80000000",
            "cpuid",
            out("eax") eax
        );
    }

    if eax < 0x80000001 {
        println!("No long mode feature");
        return false;
    }

    let edx: u32;
    unsafe {
        asm!(
            "mov eax, 0x80000001",
            "cpuid",
            out("edx") edx
        );
    }

    edx & 1 << 29 != 0
}
