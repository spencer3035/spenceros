use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

pub struct PreallocVec<T, const LEN: usize> {
    len: usize,
    inner: [T; LEN],
}

impl<T, const LEN: usize> PreallocVec<T, LEN> {
    /// Tries to push an element, returning it if there is not enough space
    #[must_use]
    pub fn push(&mut self, val: T) -> Option<T> {
        if self.len < LEN {
            self.inner[self.len - 1] = val;
            self.len += 1;
            None
        } else {
            Some(val)
        }
    }

    pub fn pop(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let val = &mut self.inner[self.len];
            Some(val)
        }
    }
}

impl<T, const LEN: usize> Deref for PreallocVec<T, LEN> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner[0..self.len]
    }
}

impl<T, const LEN: usize> DerefMut for PreallocVec<T, LEN> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner[0..self.len]
    }
}

impl<T, const LEN: usize> PreallocVec<T, LEN>
where
    T: Default,
{
    pub fn new() -> Self {
        let mut inner: [MaybeUninit<T>; LEN] = [const { MaybeUninit::uninit() }; LEN];

        for elem in &mut inner[..] {
            elem.write(T::default());
        }
        todo!()
    }
}

impl<T, const LEN: usize> Default for PreallocVec<T, LEN>
where
    T: Default + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const LEN: usize> PreallocVec<T, LEN>
where
    T: Copy,
{
    pub fn from_elem(elem: T) -> Self {
        Self {
            len: 0,
            inner: [elem; LEN],
        }
    }
}
