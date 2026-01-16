use core::ops::DerefMut;

use core::ops::Deref;

use derive_more::Add;
use derive_more::AddAssign;

pub const MEM_ENTRY_MAX: usize = 100;

pub fn parse_mem_info(
    info: &'static bootloader_api::BootInfo,
) -> Result<MemInfo, crate::BiosInfoError> {
    let mut arr = [const { MemEntry::null() }; MEM_ENTRY_MAX];
    let mut ii = 0;
    for entry in info.memory_regions.iter() {
        let is_usable = matches!(entry.kind, bootloader_api::info::MemoryRegionKind::Usable);

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
            return Err(crate::BiosInfoError::TooManyMemEntries);
        }
    }

    Ok(MemInfo {
        mem_entries: arr,
        mem_len: ii,
    })
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign)]
pub struct PhysicalAddr(pub u64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign)]
pub struct VirtualAddr(pub u64);

pub trait PhysicalMemoryMap {
    type Region: PhysicalMemoryRegion;
    fn entries(&self) -> impl Iterator<Item = &Self::Region>;
}

pub trait PhysicalMemoryRegion {
    fn start(&self) -> PhysicalAddr;
    fn end(&self) -> PhysicalAddr;
    fn is_usable(&self) -> bool;
    fn len(&self) -> u64 {
        self.end().0.saturating_sub(self.start().0)
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Maps physical memory addesses to a continuous range
pub struct PhysicalMemoryMapper<M>
where
    M: PhysicalMemoryMap,
{
    memory_map: M,
}

impl<M> PhysicalMemoryMapper<M>
where
    M: PhysicalMemoryMap,
{
    pub fn new(mem: M) -> Self {
        Self { memory_map: mem }
    }
    pub fn map(&self, addr: VirtualAddr) -> Option<PhysicalAddr> {
        let mut prev_section_break = 0;
        for entry in self.memory_map.entries().filter(|e| e.is_usable()) {
            let section_length = entry.len();
            if prev_section_break <= addr.0 && addr.0 < prev_section_break + section_length {
                let phys_addr = addr.0 - prev_section_break + entry.start().0;
                return Some(PhysicalAddr(phys_addr));
            }
            prev_section_break += section_length;
        }

        None
    }
}

#[derive(Debug, Default)]
pub struct MemEntry {
    pub start: u64,
    pub end: u64,
    pub usable: bool,
}

impl MemEntry {
    pub const fn null() -> Self {
        Self {
            start: 0,
            end: 0,
            usable: false,
        }
    }
}

impl PhysicalMemoryRegion for MemEntry {
    fn start(&self) -> PhysicalAddr {
        PhysicalAddr(self.start)
    }

    fn end(&self) -> PhysicalAddr {
        PhysicalAddr(self.end)
    }

    fn is_usable(&self) -> bool {
        self.usable
    }
}

pub struct MemInfo {
    pub mem_entries: [MemEntry; MEM_ENTRY_MAX],
    pub mem_len: usize,
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

impl PhysicalMemoryMap for MemInfo {
    type Region = MemEntry;

    fn entries(&self) -> impl Iterator<Item = &Self::Region> {
        self.iter()
    }
}

impl PhysicalMemoryRegion for bootloader_api::info::MemoryRegion {
    fn start(&self) -> PhysicalAddr {
        PhysicalAddr(self.start)
    }

    fn end(&self) -> PhysicalAddr {
        PhysicalAddr(self.end)
    }
    fn is_usable(&self) -> bool {
        matches!(self.kind, bootloader_api::info::MemoryRegionKind::Usable)
    }
}

impl PhysicalMemoryMap for bootloader_api::info::MemoryRegions {
    type Region = bootloader_api::info::MemoryRegion;

    fn entries(&self) -> impl Iterator<Item = &Self::Region> {
        self.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_phys_mem_map() {
        struct Info {
            entries: Vec<MemEntry>,
        }
        impl PhysicalMemoryMap for Info {
            type Region = MemEntry;

            fn entries(&self) -> impl Iterator<Item = &Self::Region> {
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
}
