#![no_std]
#![feature(ptr_metadata)]
#![feature(unsize)]
#![deny(unsafe_op_in_unsafe_fn)]

//!
//! A `Vec<T: ?Sized>`
//!
//! It's implemented by laying out the elements in memory contiguously like [`alloc::vec::Vec`]
//!
//! # Layout
//!
//! A [`Vechonk`] is 4 `usize` long. It owns a single allocation, containing the elements and the metadata.
//! The elements are laid out contiguously from the front, while the metadata is laid out contiguously from the back.
//! Both grow towards the center until they meet and get realloced to separate them again.
//!
//! ```txt
//!
//!             Vechonk<str>
//!             ╭──────────────────────────────────╮
//!             │ ptr   | len   | cap  | elem_size │
//!             ╰──────────────────────────────────╯
//!                │               │        │
//!                │               ╰────────│──────────────────────────────────────╮
//!                │                        │                                      │
//!                │               ╭────────╯                                      │
//!         Heap   ▼               ▼                      PtrData       PtrData    ▼
//!         ╭────────────┬─────────┬─────────────────┬──────────────┬──────────────╮
//! value   │ "hello"    │ "uwu"   │  <uninit>       │ 0 - 5        │ 5 - 3        │
//!         ├────────────┼─────────┼─────────────────┼──────────────┼──────────────┤
//!  size   │ dynamic    │ dynamic │  rest of alloc  │ usize + meta │ usize + meta │
//!         ╰────────────┴─────────┴─────────────────┴──────────────┴──────────────╯
//!             ▲            ▲                          │              │
//!             ╰────────────│──────────────────────────╯              │
//!                          ╰─────────────────────────────────────────╯
//! ```

mod iter;
mod raw;
mod test;

extern crate alloc;

use crate::raw::RawVechonk;
use alloc::boxed::Box;
use core::cmp;
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::{Deref, Index};

use crate::iter::IterMut;
pub use iter::{IntoIter, Iter};

/// chonky af
///
/// note: it does not run destructors for now, thankfully that is 100% safe :))))
pub struct Vechonk<T: ?Sized> {
    raw: RawVechonk<T>,
}

impl<T: ?Sized> Vechonk<T> {
    /// The amount of elements in the `Vechonk`, O(1)
    pub const fn len(&self) -> usize {
        self.raw.len
    }

    /// Whether the `Vechonk` is empty, O(1)
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create a new empty Vechonk that doesn't allocate anything
    pub const fn new() -> Self {
        Self {
            raw: RawVechonk::new(),
        }
    }

    /// Create a new Vechonk that allocates `capacity` bytes. `capacity` gets shrunken down
    /// to the next multiple of the alignment of usize + metadata of `T`
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            raw: RawVechonk::with_capacity(capacity),
        }
    }

    /// Pushes a new element into the [`Vechonk`]. Does panic (for now) if there is no more capacity
    /// todo: don't take a box but some U that can be unsized into T
    pub fn push(&mut self, element: Box<T>) {
        self.raw.push(element)
    }

    /// Get the last element, returns `None` if the `Vechonk` is empty
    pub fn pop(&mut self) -> Option<Box<T>> {
        self.raw.pop()
    }

    /// An iterator over the elements yielding shared references
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    /// An iterator over the elements yielding [`MutGuard`]s
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut::new(self)
    }

    /// Get a reference to an element at the index. Returns `None` if the index is out of bounds
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            // SAFETY: The index has been checked above
            unsafe { Some(self.get_unchecked(index)) }
        } else {
            None
        }
    }

    /// Get a mutable guard to an element at the index. Returns `None` if the index is out of bounds
    pub fn get_mut(&mut self, index: usize) -> Option<MutGuard<T>> {
        if index < self.len() {
            // SAFETY: The index has been checked above
            unsafe { Some(self.get_unchecked_mut(index)) }
        } else {
            None
        }
    }

    /// # Safety
    /// The index must be in bounds
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> MutGuard<T> {
        // SAFETY: We can assume that `index` is not out of bounds
        unsafe { MutGuard::new(self.raw.copy(), index) }
    }

    /// # Safety
    /// The index must be in bounds
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        // SAFETY: The metadata is only assigned directly from the pointer metadata of the original object and therefore valid
        //         The pointer is calculated from the offset, which is also valid
        //         The pointer is aligned, because it has been aligned manually in `Self::push`
        unsafe { &*self.raw.get_unchecked_ptr(index) }
    }

    /// used for debugging memory layout
    /// safety: cap must be 96
    #[allow(dead_code)]
    #[doc(hidden)]
    #[cfg(debug_assertions)]
    pub unsafe fn debug_chonk(&self) {
        let array = unsafe { *(self.raw.ptr.as_ptr() as *mut [u8; 96]) };

        panic!("{:?}", array)
    }
}

/// A guard that acts similarly to a `&mut T`, but does not allow any arbitrary value to be written,
/// instead checking whether the element has the correct size/alignment to fit the space of the old element.
pub struct MutGuard<T: ?Sized> {
    raw: RawVechonk<T>,
    /// Must always be in bounds
    index: usize,
}

impl<T: ?Sized> MutGuard<T> {
    /// # Safety
    /// The index must not be out of bounds, and `raw` must be mutable
    pub(crate) unsafe fn new(raw: RawVechonk<T>, index: usize) -> Self {
        Self { raw, index }
    }

    /// Write a new element to the location.
    /// * If the element fits in the space, the old element is returned
    /// * If the element does not fit in the space, the new element is returned again
    pub fn write(&mut self, element: Box<T>) -> Result<Box<T>, Box<T>> {
        // SAFETY: We can assume that `index` is in bounds
        unsafe { self.raw.insert_elem_unchecked(element, self.index) }
    }
}

impl<T: ?Sized> Deref for MutGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The metadata is only assigned directly from the pointer metadata of the original object and therefore valid
        //         The pointer is calculated from the offset, which is also valid
        //         The pointer is aligned, because it has been aligned manually in `Self::push`
        //         We can assume that the index is in bounds
        unsafe { &*self.raw.get_unchecked_ptr(self.index) }
    }
}

impl<T: ?Sized> Index<usize> for Vechonk<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len() {
            panic!("Out of bounds, index {} for len {}", index, self.len());
        }

        // SAFETY: The index is not out of bounds
        unsafe { self.get_unchecked(index) }
    }
}

/// don't bother with destructors for now
impl<T: ?Sized> Drop for Vechonk<T> {
    fn drop(&mut self) {
        // SAFETY: We as `Vechonk` do own the data, and it has the length `self.raw.cap`
        unsafe {
            RawVechonk::<T>::dealloc(self.raw.cap, self.raw.ptr.as_ptr());
        }
    }
}

impl<T: ?Sized> IntoIterator for Vechonk<T> {
    type Item = Box<T>;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

// default trait impls

impl<T: ?Sized> Default for Vechonk<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PartialEq for Vechonk<T>
where
    T: ?Sized + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<T> Eq for Vechonk<T> where T: ?Sized + PartialEq + Eq {}

impl<T> PartialOrd for Vechonk<T>
where
    T: ?Sized + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // see core::slice::cmp::SlicePartialOrd::partial_compare

        let len = cmp::min(self.len(), other.len());
        for i in 0..len {
            // SAFETY: We did the bounds check above
            let ordering = unsafe { self.get_unchecked(i).partial_cmp(other.get_unchecked(i)) };

            match ordering {
                Some(Ordering::Equal) => {}
                non_eq => return non_eq,
            }
        }

        self.len().partial_cmp(&other.len())
    }
}

impl<T> Ord for Vechonk<T>
where
    T: ?Sized + PartialOrd + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // see core::slice::cmp::SliceOrd::compare

        let len = cmp::min(self.len(), other.len());

        for i in 0..len {
            // SAFETY: We did the bounds check above
            let ordering = unsafe { self.get_unchecked(i).cmp(other.get_unchecked(i)) };

            match ordering {
                Ordering::Equal => {}
                non_eq => return non_eq,
            }
        }

        self.len().cmp(&other.len())
    }
}

impl<T> Hash for Vechonk<T>
where
    T: ?Sized + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|elem| elem.hash(state))
    }
}

const fn force_align(size: usize, align: usize) -> usize {
    size - (size % align)
}

#[macro_export]
macro_rules! vechonk {
    ($($x:expr),* $(,)?) => {{
        let mut chonk = $crate::Vechonk::new();
        $( chonk.push($x); )*
        chonk
    }};
}
