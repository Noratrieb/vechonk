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
//! ---------------------------------
//! | ptr   | len   | cap  | filled |
//! ---|-----------------------------
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

use alloc::boxed::Box;
use core::alloc::Layout;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ops::Index;
use core::ptr::{NonNull, Pointee};
use core::{mem, ptr};

/// chonky af
///
/// note: it does not run destructors for now, thankfully that is 100% safe :))))
pub struct Vechonk<T: ?Sized> {
    /// A pointer to the first element
    ptr: NonNull<u8>,
    /// How many elements the Vechonk has
    len: usize,
    /// How much memory the Vechonk owns
    cap: usize,
    /// How much memory has been used by the elements
    elem_size: usize,
    _marker: PhantomData<T>,
}

impl<T: ?Sized> Vechonk<T> {
    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create a new empty Vechonk that doesn't allocate anything
    pub const fn new() -> Self {
        Self {
            // SAFETY: 1 is not 0
            ptr: unsafe { NonNull::new_unchecked(1 as *mut u8) },
            len: 0,
            cap: 0,
            elem_size: 0,
            _marker: PhantomData,
        }
    }

    /// Create a new Vechonk that allocates `capacity` bytes. `capacity` gets shrunken down
    /// to the next multiple of the alignment of usize + metadata of `T`
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity - (capacity % Self::data_align());

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

    /// Pushes a new element into the [`Vechonk`]. Does panic (for now) if there is no more capacity
    pub fn push(&mut self, element: Box<T>) {
        let element_size = mem::size_of_val(element.as_ref());

        let ptr = element.as_ref();
        let meta = ptr::metadata(ptr);

        let data_size = Self::data_size();

        // just panic here instead of a proper realloc
        assert!(!self.needs_grow(element_size + data_size));

        let data: PtrData<T> = (self.len, meta);

        // SAFETY: none for now
        unsafe {
            let target_ptr = self.ptr.as_ptr().add(self.elem_size);

            ptr::copy_nonoverlapping(ptr as *const T as _, target_ptr, element_size);
        }

        // SAFETY: none for now
        let data_ptr = unsafe { self.ptr.as_ptr().add(self.cap - data_size) };
        let data_ptr = data_ptr as *mut _;

        // SAFETY: none for now
        unsafe {
            *data_ptr = data;
        }

        self.elem_size += element_size;
        self.len += 1;
    }

    /// Grows the `Vechonk` to a new capacity. This will not copy any elements. This will put the `Vechonk`
    /// into an invalid state, since the `len` is still the length of the old allocation.
    ///
    /// # Safety
    /// The caller must either set the `len` to zero, or copy the elements to the new allocation by saving
    /// `self.ptr` before calling this function.
    unsafe fn grow_to(&mut self, size: NonZeroUsize) {
        let layout = Layout::from_size_align(size.get(), Self::data_align()).unwrap();

        // SAFETY: layout is guaranteed to have a non-zero size
        let alloced_ptr = unsafe { alloc::alloc::alloc(layout) };

        self.ptr =
            NonNull::new(alloced_ptr).unwrap_or_else(|| alloc::alloc::handle_alloc_error(layout));

        self.cap = size.get();
    }

    fn needs_grow(&self, additional_size: usize) -> bool {
        additional_size > self.cap - (self.elem_size + self.data_section_size())
    }

    fn data_section_size(&self) -> usize {
        self.len * mem::size_of::<PtrData<T>>()
    }

    fn data_align() -> usize {
        mem::align_of::<PtrData<T>>()
    }

    fn data_size() -> usize {
        mem::size_of::<PtrData<T>>()
    }
}

impl<T: ?Sized> Index<usize> for Vechonk<T> {
    type Output = T;

    fn index(&self, _index: usize) -> &Self::Output {
        todo!()
    }
}

/// don't bother with destructors for now
impl<T: ?Sized> Drop for Vechonk<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        // SAFETY: 1 is not 0 and a power of two. `size > usize::MAX` must always be true
        let layout = Layout::from_size_align(self.cap, mem::align_of::<PtrData<T>>()).unwrap();

        unsafe { alloc::alloc::dealloc(self.ptr.as_ptr(), layout) };
    }
}

impl<T: ?Sized> Default for Vechonk<T> {
    fn default() -> Self {
        Self::new()
    }
}

type PtrData<T> = (usize, <T as Pointee>::Metadata);
