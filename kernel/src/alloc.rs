#![allow(dead_code)]

use core::alloc::GlobalAlloc;

use crate::mutex::Mutex;

/// PAGE_SIZE is 2^PAGE_EXP
pub const PAGE_EXP: usize = 12;
/// Bytes per page
pub const PAGE_SIZE: usize = 1 << PAGE_EXP;
/// Length of buddy 0
const BUDDY_0_SIZE: usize = 1 << 14;
const BUDDY_0_PAGES_PER_BIT: usize = 1;
/// Length of buddy 1
const BUDDY_1_SIZE: usize = 1 << 8;
const BUDDY_1_PAGES_PER_BIT: usize = 64;
/// Length of buddy 2
const BUDDY_2_SIZE: usize = 1 << 2;
const BUDDY_2_PAGES_PER_BIT: usize = 64 * 64;

pub static BUDDY_ALLOCATOR: Mutex<BuddyAllocator> = Mutex::new(BuddyAllocator::new());

#[derive(Debug)]
pub enum AllocError {
    NotEnoughSpace,
    Internal(BuddyError),
}

impl From<BuddyError> for AllocError {
    fn from(value: BuddyError) -> Self {
        match value {
            BuddyError::PageOutOfRange => Self::NotEnoughSpace,
            BuddyError::DoubleSet => Self::Internal(value),
        }
    }
}

pub struct PhysicalMemoryAllocator;

unsafe impl GlobalAlloc for PhysicalMemoryAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        BUDDY_ALLOCATOR.lock().alloc(layout).unwrap()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        BUDDY_ALLOCATOR.lock().dealloc(ptr, layout).unwrap()
    }
}

pub struct Page<const SIZE: usize = PAGE_SIZE> {
    addr: usize,
}

impl Page {
    fn new(addr: usize) -> Self {
        Self { addr }
    }
}

pub struct BuddyAllocator {
    b0: Buddy<BUDDY_0_SIZE, BUDDY_0_PAGES_PER_BIT>,
    b1: Buddy<BUDDY_1_SIZE, BUDDY_1_PAGES_PER_BIT>,
    b2: Buddy<BUDDY_2_SIZE, BUDDY_2_PAGES_PER_BIT>,
}

impl BuddyAllocator {
    const fn new() -> Self {
        Self {
            b0: Buddy::new(),
            b1: Buddy::new(),
            b2: Buddy::new(),
        }
    }

    fn buddy_level(layout: core::alloc::Layout) -> Option<usize> {
        let size_req = layout.pad_to_align().size();
        let pages_needed = size_req.div_ceil(PAGE_SIZE);
        let pages_for_align = layout.align().div_ceil(PAGE_SIZE);
        if pages_for_align <= BUDDY_0_PAGES_PER_BIT && pages_needed <= BUDDY_0_PAGES_PER_BIT {
            // Use buddy 0
            Some(0)
        } else if pages_for_align <= BUDDY_1_PAGES_PER_BIT && pages_needed <= BUDDY_1_PAGES_PER_BIT
        {
            // Use buddy 1
            Some(1)
        } else if pages_for_align <= BUDDY_2_PAGES_PER_BIT && pages_needed <= BUDDY_2_PAGES_PER_BIT
        {
            // Use buddy 2
            Some(2)
        } else {
            // Out of memory
            None
        }
    }

    pub fn reserve_addr(&mut self, addr: usize, size: usize) {
        self.b0.set_addr_unchecked(addr, size);
        self.b1.set_addr_unchecked(addr, size);
        self.b2.set_addr_unchecked(addr, size);
    }

    pub fn alloc(&mut self, layout: core::alloc::Layout) -> Result<*mut u8, AllocError> {
        let size_req = layout.pad_to_align().size();
        let num_pages = size_req.div_ceil(PAGE_SIZE);
        let page_align = layout.align().div_ceil(PAGE_SIZE) as u32;
        let page = match Self::buddy_level(layout) {
            Some(0) => self
                .b0
                .get_free_page(num_pages, page_align)
                .ok_or(AllocError::NotEnoughSpace)?,
            Some(1) => self
                .b1
                .get_free_page(num_pages, page_align)
                .ok_or(AllocError::NotEnoughSpace)?,
            Some(2) => self
                .b2
                .get_free_page(num_pages, page_align)
                .ok_or(AllocError::NotEnoughSpace)?,
            _ => return Err(AllocError::NotEnoughSpace),
        };
        self.b0.set_addr(page.addr, size_req)?;
        self.b1.set_addr(page.addr, size_req)?;
        self.b2.set_addr(page.addr, size_req)?;
        Ok(page.addr as *mut u8)
    }

    pub fn dealloc(&mut self, ptr: *mut u8, layout: core::alloc::Layout) -> Result<(), AllocError> {
        let size = layout.pad_to_align().size();
        self.b0.unset_addr(ptr.addr(), size)?;
        self.b1.unset_addr(ptr.addr(), size)?;
        self.b2.unset_addr(ptr.addr(), size)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum BuddyError {
    PageOutOfRange,
    DoubleSet,
}

#[derive(Debug)]
pub struct Buddy<const N: usize, const P: usize> {
    masks: [u64; N],
}

impl<const N: usize, const P: usize> Default for Buddy<N, P> {
    fn default() -> Self {
        Self::new()
    }
}

// N: size of internal array
// P: Pages per bit
//
// Total pages = N * P * 64
#[cfg(test)]
impl<const N: usize, const P: usize> core::fmt::Display for Buddy<N, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut page_ii = 0;
        for mask in self.masks.iter() {
            writeln!(f, "0x{page_ii:X} ")?;
            writeln!(f, "0b{mask:064b} ")?;
            page_ii += P * 64;
        }

        Ok(())
    }
}

/// Finds the offset that there are `n` consecatvie zeros
fn find_n_consec_zero_with_align(mask: u64, n: u8, align: u8) -> Option<u8> {
    let mut offset = 0;
    while (!mask << offset).leading_ones() < n as u32 {
        offset += align;
        if offset >= 64 {
            break;
        }
    }
    if (!mask << offset).leading_ones() >= n as u32 {
        Some(offset)
    } else {
        None
    }
}

impl<const N: usize, const P: usize> Buddy<N, P> {
    pub const fn new() -> Self {
        Self { masks: [0; N] }
    }

    /// Gets the linear address that a given mask and bit index starts
    fn addr_of(mask_ii: usize, bit_ii: usize) -> Option<usize> {
        if mask_ii < N && bit_ii < 64 {
            let num_bits = mask_ii * 64 + bit_ii;
            Some(num_bits * P)
        } else {
            None
        }
    }

    /// Gets sets the correct pages to used
    pub fn get_free_page(&self, num_pages: usize, page_align: u32) -> Option<Page> {
        // Full empty mask does not contain enough space
        if num_pages > P * 64 {
            return None;
        }

        // Allignment relative to each bit within a mask
        let bit_align = page_align.div_ceil(P as u32);

        for (mask_ii, mask) in self.masks.iter().enumerate() {
            // Inverted mask, 1 indicates there is free space
            let mut mask_inv = !mask;
            if mask_inv.count_ones() < bit_align {
                continue;
            }
            let mut bit_ii = 0;
            while bit_ii < 64 && mask_inv.leading_ones() < bit_align {
                mask_inv <<= bit_align;
                bit_ii += bit_align;
            }

            if mask_inv.leading_ones() >= bit_align {
                let bit_ii = bit_ii as usize;
                let addr = Self::addr_of(mask_ii, bit_ii)?;
                return Some(Page::new(addr));
            }
        }

        None
    }

    fn convert_addr_len(addr: usize, len: usize) -> (usize, usize) {
        let page_index = addr / PAGE_SIZE;
        let num_pages = len.div_ceil(PAGE_SIZE);
        (page_index, num_pages)
    }

    pub fn set_addr_unchecked(&mut self, addr: usize, len: usize) {
        let (page_index, num_pages) = Self::convert_addr_len(addr, len);
        self.set_unchecked(page_index, num_pages)
    }

    pub fn set_addr(&mut self, addr: usize, len: usize) -> Result<(), BuddyError> {
        let (page_index, num_pages) = Self::convert_addr_len(addr, len);
        self.set(page_index, num_pages)
    }

    pub fn unset_addr(&mut self, addr: usize, len: usize) -> Result<(), BuddyError> {
        let (page_index, num_pages) = Self::convert_addr_len(addr, len);
        self.unset(page_index, num_pages)
    }

    /// Sets the relevant range to occupied
    pub fn set_unchecked(&mut self, page_index: usize, num_pages: usize) {
        if num_pages == 0 {
            return;
        }
        let Ok((mask, mut bits_to_set, mut start_ii)) =
            self.get_first_mask_nbits_and_start_ii(page_index, num_pages)
        else {
            // Page is out of range, ignore it
            return;
        };

        self.masks[start_ii] |= mask;
        start_ii += 1;

        while bits_to_set >= 64 {
            bits_to_set -= 64;
            self.masks[start_ii] = u64::MAX;
            start_ii += 1;
        }

        if bits_to_set > 0 {
            let mask = !(u64::MAX >> bits_to_set);
            self.masks[start_ii] |= mask;
        }
    }

    /// Sets the relevant range to occupied
    pub fn set(&mut self, page_index: usize, num_pages: usize) -> Result<(), BuddyError> {
        if num_pages == 0 {
            return Ok(());
        }
        let (mask, mut bits_to_set, mut start_ii) =
            self.get_first_mask_nbits_and_start_ii(page_index, num_pages)?;

        if self.masks[start_ii] & mask != 0 {
            return Err(BuddyError::DoubleSet);
        }
        self.masks[start_ii] |= mask;
        start_ii += 1;

        while bits_to_set >= 64 {
            bits_to_set -= 64;
            if self.masks[start_ii] != 0 {
                return Err(BuddyError::DoubleSet);
            }
            self.masks[start_ii] = u64::MAX;
            start_ii += 1;
        }

        if bits_to_set > 0 {
            let mask = !(u64::MAX >> bits_to_set);
            if self.masks[start_ii] & mask != 0 {
                return Err(BuddyError::DoubleSet);
            }
            self.masks[start_ii] |= mask;
        }

        Ok(())
    }

    /// Sets the relevant range to free
    pub fn unset(&mut self, page_index: usize, num_pages: usize) -> Result<(), BuddyError> {
        if num_pages == 0 {
            return Ok(());
        }
        let (mask, mut bits_to_set, mut start_ii) =
            self.get_first_mask_nbits_and_start_ii(page_index, num_pages)?;

        self.masks[start_ii] &= !mask;

        start_ii += 1;
        while bits_to_set >= 64 {
            bits_to_set -= 64;
            self.masks[start_ii] = 0;
        }

        if bits_to_set > 0 {
            let mask = !(u64::MAX >> bits_to_set);
            self.masks[start_ii] &= !mask;
        }

        Ok(())
    }

    /// Helper to simplify logic of other calls
    fn get_first_mask_nbits_and_start_ii(
        &self,
        page_index: usize,
        num_pages: usize,
    ) -> Result<(u64, usize, usize), BuddyError> {
        let start_bit = page_index / P;
        let end_bit = (page_index + num_pages).div_ceil(P);
        if end_bit / 64 >= self.masks.len() {
            return Err(BuddyError::PageOutOfRange);
        }
        let mut bits_to_set = end_bit - start_bit;
        let start_ii = start_bit / 64;
        let offset = start_bit % 64;

        let mask = if offset + bits_to_set < 64 {
            let mask = (u64::MAX >> offset) & (u64::MAX << (64 - offset - bits_to_set));
            bits_to_set = 0;
            mask
        } else {
            let mask = u64::MAX >> offset;
            bits_to_set -= 64 - offset;
            mask
        };

        Ok((mask, bits_to_set, start_ii))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_n_consec_zero() {
        let mask: u64 = 0;
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 1), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 2), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 4), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 3, 1), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 3, 4), Some(0));

        let mask: u64 = u64::MAX;
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 1), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 3, 1), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 4), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 3, 4), None);

        // Last bit zero
        let mask: u64 = !1;
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 1), Some(63));
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 2), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 4), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 1), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 2), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 3, 1), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 4, 1), None);

        // Last 2 bits zero
        let mask: u64 = !0b11;
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 1), Some(62));
        assert_eq!(find_n_consec_zero_with_align(mask, 3, 1), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 4, 1), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 2), Some(62));
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 2), Some(62));
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 1), Some(62));

        // First bit zero
        let mask: u64 = !(1 << 63);
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 1), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 2), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 4), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 1), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 2), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 3, 1), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 4, 1), None);

        // First 2 bits zero
        let mask: u64 = !(0b11 << 62);
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 1), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 2), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 4), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 1), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 2), Some(0));
        assert_eq!(find_n_consec_zero_with_align(mask, 3, 1), None);
        assert_eq!(find_n_consec_zero_with_align(mask, 4, 1), None);

        let mask: u64 = 0b1000000000000000000000000000000000000000000000000000000000000000;
        assert_eq!(find_n_consec_zero_with_align(mask, 1, 1), Some(1));
        let mask: u64 = 0b1010100000000000000000000000000000000000000000000000000000000000;
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 1), Some(5));
        let mask: u64 = 0b1010100000000000000000000000000000000000000000000000000000000000;
        assert_eq!(find_n_consec_zero_with_align(mask, 2, 2), Some(6));
    }

    #[test]
    fn test_buddy() {
        const P: usize = 10;
        const N: usize = 10;

        let mut buddy: Buddy<N, P> = Buddy::new();
        let page_index = 0;
        let num_pages = 1;
        buddy.set(page_index, num_pages).unwrap();
        for (ii, mask) in buddy.masks.iter().enumerate() {
            if ii == 0 {
                assert_eq!(*mask, 1 << 63);
            } else {
                assert_eq!(*mask, 0);
            }
        }

        let mut buddy: Buddy<N, P> = Buddy::new();
        let page_index = 0;
        let num_pages = 63;
        buddy.set(page_index, num_pages).unwrap();
        for (ii, mask) in buddy.masks.iter().enumerate() {
            if ii == 0 {
                assert_eq!(*mask, u64::MAX << (64 - 7));
            } else {
                assert_eq!(*mask, 0);
            }
        }

        let mut buddy: Buddy<N, P> = Buddy::new();
        let page_index = 63;
        let num_pages = 64 * P;
        buddy.set(page_index, num_pages).unwrap();
        for (ii, mask) in buddy.masks.iter().enumerate() {
            if ii == 0 {
                assert_eq!(*mask, u64::MAX >> 6);
            } else if ii == 1 {
                assert_eq!(*mask, u64::MAX << (64 - 7));
            } else {
                assert_eq!(*mask, 0);
            }
        }

        let mut buddy: Buddy<N, P> = Buddy::new();
        let page_index = 63;
        let num_pages = 3 * 64 * P;
        buddy.set(page_index, num_pages).unwrap();
        for (ii, mask) in buddy.masks.iter().enumerate() {
            if ii == 0 {
                assert_eq!(*mask, u64::MAX >> 6);
            } else if ii == 1 || ii == 2 {
                assert_eq!(*mask, u64::MAX);
            } else if ii == 3 {
                assert_eq!(*mask, u64::MAX << (64 - 7));
            } else {
                assert_eq!(*mask, 0);
            }
        }
    }
}
