#![no_std]
#![feature(ptr_metadata)]
#![deny(unsafe_op_in_unsafe_fn)]

//!
//! A `Vec<T: ?Sized>`
//!
//! It's implemented by laying out the elements in memory contiguously like [`alloc::vec::Vec`]
//!
//! # Layout
//!//!
//! A [`Vechonk`] is 3 `usize` long. It owns a single allocation, containing the elements and the metadata.
//! The elements are laid out contiguously from the front, while the metadata is laid out contiguously from the back.
//! Both grow towards the center until they meet and get realloced to separate them again.
//!
//! ```txt
//!
//! Vechonk<str>
//! ------------------------
//! | ptr   | len   | cap  |
//! ---|---------------------
//!    |
//!    |___
//!        |
//! Heap   v
//! ------------------------------------------------------------------------
//! | "hello"   | "uwu"    |  <uninit>       | 0 - 5        | 5 - 3        |
//! |-----------|----------|-----------------|--------------|--------------|
//! | dynamic   | dynamic  |  rest of alloc  | usize + meta | usize + meta |
//! --------------------------------------------|--------------|------------
//!     ^            ^                          |              |
//!     |___________ | _________________________|              |
//!                  |_________________________________________|
//! ```

mod test;

extern crate alloc;

use core::alloc::Layout;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ptr::NonNull;

/// chonky af
///
/// only works for copy types, but this is WIP and will be removed
pub struct Vechonk<T: ?Sized + Copy> {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T: ?Sized + Copy> Vechonk<T> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn new() -> Self {
        Self {
            // SAFETY: 1 is not 0
            ptr: unsafe { NonNull::new_unchecked(1 as *mut u8) },
            len: 0,
            cap: 0,
            _marker: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut vechonk = Self::new();

        if capacity == 0 {
            return vechonk;
        }

        // SAFETY: capacity has been checked to not be 0 and the len is 0
        unsafe {
            vechonk.grow_to(NonZeroUsize::new_unchecked(capacity));
        }
        vechonk
    }

    /// Grows the `Vechonk` to a new capacity. This will not copy any elements. This will put the `Vechonk`
    /// into an invalid state, since the `len` is still the length of the old allocation.
    ///
    /// # Safety
    /// The caller must either set the `len` to zero, or copy the elements to the new allocation by saving
    /// `self.ptr` before calling this function.
    unsafe fn grow_to(&mut self, size: NonZeroUsize) {
        // SAFETY: 1 is not 0 and a power of two. `size > usize::MAX` must always be true
        let layout = unsafe { Layout::from_size_align_unchecked(size.get(), 1) };

        // SAFETY: layout is guaranteed to have a non-zero size
        let alloced_ptr = unsafe { alloc::alloc::alloc(layout) };

        self.ptr =
            NonNull::new(alloced_ptr).unwrap_or_else(|| alloc::alloc::handle_alloc_error(layout));

        self.cap = size.get();
    }
}


// we can only drop copy for now because destructors 🤮
impl<T: ?Sized + Copy> Drop for Vechonk<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        // SAFETY: 1 is not 0 and a power of two. `size > usize::MAX` must always be true
        let layout = unsafe { Layout::from_size_align_unchecked(self.cap, 1) };

        unsafe { alloc::alloc::dealloc(self.ptr.as_ptr(), layout) };
    }
}
