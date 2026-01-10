#[derive(Debug)]
pub struct Buddy<const N: usize> {
    pages_per_bit: usize,
    masks: [u64; N],
}

#[cfg(test)]
impl<const N: usize> core::fmt::Display for Buddy<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut page_ii = 0;
        for mask in self.masks.iter() {
            writeln!(f, "0x{page_ii:X} ")?;
            writeln!(f, "0b{mask:064b} ")?;
            page_ii += self.pages_per_bit * 64;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum BuddyError {
    OutOfRange,
    DoubleSet,
}

impl<const N: usize> Buddy<N> {
    pub fn new(pages_per_bit: usize) -> Self {
        Self {
            pages_per_bit,
            masks: [0; N],
        }
    }
    /// Gets the index of the first free block
    pub fn first_free(&self) -> Option<usize> {
        for mask in self.masks.iter() {
            if *mask == u64::MAX {
                continue;
            }
            let offset = mask.leading_ones() + 1;
            return Some(offset as usize);
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
        let start_bit = page_index / self.pages_per_bit;
        let end_bit = (page_index + num_pages).div_ceil(self.pages_per_bit);
        if end_bit / 64 >= self.masks.len() {
            return Err(BuddyError::OutOfRange);
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

pub struct BuddyAllocator {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_buddy() {
        let pages_per_bit = 10;
        const N: usize = 10;

        let mut buddy: Buddy<N> = Buddy::new(pages_per_bit);
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

        let mut buddy: Buddy<N> = Buddy::new(pages_per_bit);
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

        let mut buddy: Buddy<N> = Buddy::new(pages_per_bit);
        let page_index = 63;
        let num_pages = 64 * pages_per_bit;
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

        let mut buddy: Buddy<N> = Buddy::new(pages_per_bit);
        let page_index = 63;
        let num_pages = 3 * 64 * pages_per_bit;
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
