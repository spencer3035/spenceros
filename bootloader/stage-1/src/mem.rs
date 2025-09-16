use core::{arch::asm, ptr::addr_of};

use common::println_vbe;

#[repr(C, packed)]
#[derive(Debug)]
struct MemEntry {
    base_address: u64,
    length: u64,
    mem_type: u32,
    bitfield: u32,
}

impl MemEntry {
    const fn null() -> Self {
        Self {
            base_address: 0,
            length: 0,
            mem_type: 0,
            bitfield: 0,
        }
    }
}

impl core::fmt::Display for MemEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let base_addr = self.base_address;
        let len = self.length;
        write!(f, "[address,size]: [0x{:X}, 0x{:X}]", base_addr, len)
    }
}

static mut MEM_ENTRY: MemEntry = MemEntry::null();

/// Detects memory using int 0x15 with eax = 0xE820, returns number of entries read
pub fn detect_memory() -> u16 {
    let int15_ax: u32 = 0xE820;
    // "SMAP"
    let magic_number: u32 = 0x534d4150;
    let mem_address = addr_of!(MEM_ENTRY) as usize;

    if mem_address > u16::MAX as usize {
        panic!("address for target is out of range [0, 0xFFFF]: 0x{mem_address:X}");
    } else {
        println_vbe!("address for target is 0x{mem_address:X}");
    }

    // Registers
    let mut return_code;
    let mut addr: u16;
    let mut map_index = 0;
    // Size of buffer, should be 24
    let mut buffer_bytes;

    let mut count = 0;
    loop {
        return_code = int15_ax;
        // Address to save to before, address saved to after
        addr = mem_address as u16;
        // Size of our buffer in, number of bytes stored out
        buffer_bytes = size_of::<MemEntry>();

        #[allow(unused_assignments)]
        unsafe {
            asm!(
                // TODO: Figure out how to do this
                // We assume that the values we are storing are within the 0 to 0xFFFF range and
                // that di contains the address of the target without any offset
                // "mov es, 0",
                "int 0x15",
                "jnc 2f",
                "cmp eax, 0x534d4150",
                "jne 2f",
                // IF CF is set, it is an error. If, for some reason, eax hasn't changed from the
                // magic number, set it to 0 because whe check that below as our only check
                "mov eax, 0",
                "2:",
                // https://wiki.osdev.org/Detecting_Memory_(x86)#BIOS_Function:_INT_0x15,_EAX_=_0xE820
                // If success:
                // EBX is nonzero, should be preserved to next call
                // If end:
                // ebx == 0 or carry flag is set
                inout("di") addr,
                inout("eax") return_code,
                inout("ebx") map_index,
                inout("ecx") buffer_bytes,
                in("edx") magic_number,
            );
        }

        if return_code != magic_number {
            panic!("Failed getting memory return code: 0x{return_code:X}");
        }

        if buffer_bytes == 20 {
            // Can NOT use ACPI 3.0 Extended bitfield
        } else if buffer_bytes == 24 {
            // Can use ACPI 3.0 Extended bitfield
        } else {
            println_vbe!("Bad number of bytes read");
        }

        let mem_type = unsafe { MEM_ENTRY.mem_type };
        if mem_type == 1 {
            // Valid memory we can use
            println_vbe!("{count} FREE: {}", unsafe {
                addr_of!(MEM_ENTRY).as_ref().unwrap()
            });
        } else {
            // Invalid memory
            println_vbe!("{count} RESV: {}", unsafe {
                addr_of!(MEM_ENTRY).as_ref().unwrap()
            });
        }

        count += 1;

        // TODO: Also check carry is clear
        if map_index == 0 {
            break;
        }
    }

    // Reference: https://wiki.osdev.org/Detecting_Memory_(x86)
    // TODO: Increment di, reset eax and ecx, until ebx==0 or carry is set
    count
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mem_entry_size() {
        assert_eq!(size_of::<MemEntry>(), 24);
    }
}
