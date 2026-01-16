#![allow(dead_code)]

use crate::mem::VirtualAddr;

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

pub struct Page<const SIZE: usize = PAGE_SIZE> {
    addr: VirtualAddr,
}

pub struct BuddyAllocator {
    b0: Buddy<BUDDY_0_SIZE, BUDDY_0_PAGES_PER_BIT>,
    b1: Buddy<BUDDY_1_SIZE, BUDDY_1_PAGES_PER_BIT>,
    b2: Buddy<BUDDY_2_SIZE, BUDDY_2_PAGES_PER_BIT>,
}

impl BuddyAllocator {
    fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
        let size_req = layout.pad_to_align().size();
        let _pages_needed = size_req.div_ceil(PAGE_SIZE);
        let pages_for_align = layout.align().div_ceil(PAGE_SIZE);
        if pages_for_align <= BUDDY_0_PAGES_PER_BIT {
            // Use buddy 0
            todo!()
        } else if pages_for_align <= BUDDY_1_PAGES_PER_BIT {
            // Use buddy 1
            todo!()
        } else if pages_for_align <= BUDDY_2_PAGES_PER_BIT {
            // Use buddy 2
            todo!()
        } else {
            // Out of memory
            core::ptr::null_mut()
        }
    }
    fn dealloc(&mut self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        todo!()
    }
}

unsafe impl core::alloc::GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        todo!()
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
    pub fn new() -> Self {
        Self { masks: [0; N] }
    }

    /// Gets sets the correct pages to used
    pub fn alloc_pages(&self, _num_pages: usize, page_align: u32) -> Option<Page> {
        for mask in self.masks.iter() {
            if mask.count_zeros() < page_align {
                continue;
            }
            let _offset = page_align;
            // while !mask & !(u64::MAX << (64 - offset)) {
            //     todo!()
            // }
            todo!()
            // return Some(offset as usize);
        }

        None
    }

    /// Sets the relevant range to occupied
    pub fn set(&mut self, page_index: usize, num_pages: usize) -> Result<(), BuddyError> {
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
