use core::{arch::asm, ptr::addr_of};

use common::{println_vbe, static_items::mem::MemInfo};

use crate::utils::prompt_continue;

#[repr(C, packed)]
#[derive(Debug)]
struct Int15MemEntry {
    base_address: u64,
    length: u64,
    mem_type: u32,
    bitfield: u32,
}

impl Int15MemEntry {
    const fn null() -> Self {
        Self {
            base_address: 0,
            length: 0,
            mem_type: 0,
            bitfield: 0,
        }
    }
}

impl core::fmt::Display for Int15MemEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let base_addr = self.base_address;
        let len = self.length;
        write!(f, "[address,size]: [0x{:X}, 0x{:X}]", base_addr, len)
    }
}

static mut MEM_ENTRY: Int15MemEntry = Int15MemEntry::null();

/// Detects memory using int 0x15 with eax = 0xE820, and populates information to send to bios
pub fn detect_memory(memory_info: &mut MemInfo) {
    // Reference: https://wiki.osdev.org/Detecting_Memory_(x86)
    let int15_ax: u32 = 0xE820;
    // "SMAP"
    let magic_number: u32 = 0x534d4150;
    // Address we will put the information from the interrupt.
    let mem_address = addr_of!(MEM_ENTRY) as usize;

    // Registers
    let mut return_code;
    let mut addr: u16;
    let mut map_index = 0;
    // Size of buffer, should be 24
    let mut buffer_bytes;

    let mut free_entries = 0;
    loop {
        return_code = int15_ax;
        // Address to save to before, address saved to after
        addr = mem_address as u16;
        // Size of our buffer in, number of bytes stored out
        buffer_bytes = size_of::<Int15MemEntry>();

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

        if free_entries >= MemInfo::NUM_ENTRIES {
            println_vbe!(
                "Got too many memory entries! We can only support {}",
                MemInfo::NUM_ENTRIES
            );
            prompt_continue();
            break;
        }

        let (mem_type, length, address) =
            unsafe { (MEM_ENTRY.mem_type, MEM_ENTRY.length, MEM_ENTRY.base_address) };
        if mem_type == 1 {
            // Valid memory we can use
            memory_info.table[free_entries].physical_address = address;
            memory_info.table[free_entries].length = length;
            free_entries += 1;
        } else {
            // Invalid or reserved memory
        }

        // TODO: Also check carry is clear
        if map_index == 0 {
            break;
        }
    }
    memory_info.num_entries = free_entries as u8;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mem_entry_size() {
        assert_eq!(size_of::<Int15MemEntry>(), 24);
    }
}
