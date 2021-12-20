#![no_std]
#![feature(ptr_metadata)]

//!
//! A `Vec<T: ?Sized>`
//!
//! It's implemented by laying out the elements in memory contiguously like `alloc::vec::Vec`

struct Vechonk<T: ?Sized>;
