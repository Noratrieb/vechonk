use crate::Vechonk;
use core::marker::PhantomData;
use core::ptr::NonNull;

/// An iterator over the elements of a [`Vechonk`]
pub struct Iter<'a, T: ?Sized> {
    /// A pointer to the first element
    ptr: NonNull<u8>,
    /// How many elements the Vechonk has
    len: usize,
    /// How much memory the Vechonk owns
    cap: usize,
    /// How much memory has been used by the elements, where the next element starts
    elem_size: usize,
    /// The next element the iterator will return
    current_index: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Iter<'a, T> {
    pub(super) fn new(chonk: &'a Vechonk<T>) -> Iter<'a, T> {
        Self {
            ptr: chonk.ptr,
            len: chonk.len,
            cap: chonk.cap,
            elem_size: chonk.elem_size,
            current_index: 0,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.len - self.current_index;

        (count, Some(count))
    }
}

impl<'a, T: ?Sized> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len - self.current_index
    }
}
