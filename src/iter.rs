use crate::{RawVechonk, Vechonk};
use core::marker::PhantomData;

/// An iterator over the elements of a [`Vechonk`]
pub struct Iter<'a, T: ?Sized> {
    raw: RawVechonk<T>,
    current_index: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Iter<'a, T> {
    pub(super) fn new(chonk: &'a Vechonk<T>) -> Iter<'a, T> {
        Self {
            raw: chonk.raw.copy(),
            current_index: 0,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == self.raw.len {
            return None;
        }

        // SAFETY: We just did a bounds check above
        let ptr = unsafe { self.raw.get_unchecked_ptr(self.current_index) };

        self.current_index += 1;

        // SAFETY: We rely on `get_unchecked_ptr` returning a valid pointer, which is does, see its SAFETY comments
        unsafe { Some(&*ptr) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.raw.len - self.current_index;

        (count, Some(count))
    }
}

impl<'a, T: ?Sized> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.raw.len - self.current_index
    }
}
