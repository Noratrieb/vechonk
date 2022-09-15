#![cfg(test)]

use core::{cmp::Ordering, hash::Hash};

use crate::{vechonk, Vechonk};
use alloc::boxed::Box;

const USIZE_BYTES: usize = (usize::BITS / 8) as usize;

#[repr(align(2048))]
struct BigAlign(u8);

trait TakeMut {
    fn take_mut(&mut self) {}
}

impl<T: ?Sized> TakeMut for T {}

trait Decrement {
    fn decrement(&mut self);
    fn value(&self) -> usize;
}

impl Decrement for usize {
    fn decrement(&mut self) {
        *self -= 1;
    }
    fn value(&self) -> usize {
        *self
    }
}

#[test]
fn new() {
    let chonk = Vechonk::<()>::new();
    assert_eq!(chonk.len(), 0);
}

#[test]
fn default() {
    let chonk = Vechonk::<()>::default();
    assert_eq!(chonk.len(), 0);
}

#[test]
fn zero_capacity() {
    let chonk = Vechonk::<()>::with_capacity(0);

    assert_eq!(chonk.len(), 0);
}

#[test]
fn some_capacity() {
    let chonk = Vechonk::<()>::with_capacity(96);

    assert_eq!(chonk.len(), 0);
}

#[test]
fn push_single_sized_elem() {
    let mut chonk = Vechonk::<u8>::with_capacity(96);

    chonk.push(Box::new(1));

    assert_eq!(chonk.len(), 1);
}

#[test]
fn push_single_unsized_elem() {
    let mut chonk = Vechonk::<str>::with_capacity(96);

    chonk.push("hello".into());

    assert_eq!(chonk.len(), 1);
}

#[test]
fn push_two_sized_elem() {
    let mut chonk = Vechonk::<u8>::with_capacity(96);

    chonk.push(Box::new(1));
    chonk.push(Box::new(2));

    assert_eq!(chonk.len(), 2);
    assert_eq!(chonk.raw.elem_size, 2);
    assert_eq!(chonk.raw.data_section_size(), USIZE_BYTES * 2); // two indecies
}

#[test]
fn push_two_unsized_elem() {
    let mut chonk = Vechonk::<str>::with_capacity(96);

    chonk.push("hello".into());
    chonk.push("uwu".into());

    assert_eq!(chonk.len(), 2);
    assert_eq!(chonk.raw.elem_size, 8);
    assert_eq!(chonk.raw.data_section_size(), USIZE_BYTES * 4); // two indecies + lengths
}

#[test]
#[should_panic]
fn index_out_of_bounds() {
    let chonk = Vechonk::<str>::with_capacity(96);

    let _ = chonk[0];
}

#[test]
fn index() {
    let mut chonk = Vechonk::<str>::with_capacity(96);

    chonk.push("hello".into());
    chonk.push("uwu".into());

    let hello = &chonk[0];
    let uwu = &chonk[1];

    assert_eq!(hello, "hello");
    assert_eq!(uwu, "uwu");
}

#[test]
fn grow_from_empty() {
    let mut chonk = Vechonk::<str>::new();

    assert_eq!(chonk.len(), 0);

    chonk.push("hello".into());
    chonk.push("uwu".into());

    let hello = &chonk[0];
    let uwu = &chonk[1];

    assert_eq!(hello, "hello");
    assert_eq!(uwu, "uwu");

    assert_eq!(chonk.len(), 2);
}

#[test]
fn grow_from_alloc() {
    let mut chonk = Vechonk::<str>::with_capacity(32);

    assert_eq!(chonk.len(), 0);

    chonk.push("hello".into());
    chonk.push("uwu".into());

    let hello = &chonk[0];
    let uwu = &chonk[1];

    assert_eq!(hello, "hello");
    assert_eq!(uwu, "uwu");

    assert_eq!(chonk.len(), 2);
}

#[test]
fn push_alignment() {
    use core::any::Any;

    let mut chonk = Vechonk::<dyn Any>::with_capacity(4096);

    chonk.push(Box::new(BigAlign(5)));
    chonk.push(Box::new(0_u8));
    chonk.push(Box::new(1_u64));

    let _ = chonk[0];
    let _ = chonk[1];
}

#[test]
fn grow_alignment() {
    use core::any::Any;

    let mut chonk = Vechonk::<dyn Any>::with_capacity(32);

    chonk.push(Box::new(0_u8));
    chonk.push(Box::new(1_u64));
    chonk.push(Box::new(0_u128));
    chonk.push(Box::new(BigAlign(5)));
    chonk.push(Box::new(8_u128));
    chonk.push(Box::new("dsajkfhdsajklfdsklaöfjdklsöjfkldsfjlkds"));
    chonk.push(Box::new(4_u128));
    chonk.push(Box::new(5_u128));
    chonk.push(Box::new(BigAlign(5)));
    chonk.push(Box::new(6_u128));
    chonk.push(Box::new(3_u128));

    let _ = chonk[0];
    let _ = chonk[1];
}

#[test]
fn popping() {
    let mut chonk = Vechonk::<str>::with_capacity(512);

    chonk.push("hello".into());
    chonk.push("uwu".into());
    chonk.push("I'm popping off!".into());

    let popping = chonk.pop().unwrap();
    let uwu = chonk.pop().unwrap();
    let hello = chonk.pop().unwrap();
    let end = chonk.pop();

    assert_eq!(popping.as_ref(), "I'm popping off!");
    assert_eq!(uwu.as_ref(), "uwu");
    assert_eq!(hello.as_ref(), "hello");
    assert_eq!(end, None);
}

#[test]
fn iter() {
    let chonk: Vechonk<str> = vechonk!["hello".into(), "uwu".into()];
    let mut iter = chonk.iter();

    assert_eq!(iter.next(), Some("hello"));
    assert_eq!(iter.next(), Some("uwu"));
    assert_eq!(iter.next(), None);
}

#[test]
fn into_iter() {
    let chonk: Vechonk<str> = vechonk!["hello".into(), "uwu".into()];

    let mut iter = chonk.into_iter();

    assert_eq!(iter.next().unwrap().as_ref(), "hello");
    assert_eq!(iter.next().unwrap().as_ref(), "uwu");
    assert_eq!(iter.next(), None);
}

#[test]
fn partial_eq_eq() {
    let chonk1 = vechonk![235.0.into(), 325.8.into()];
    let chonk2 = vechonk![235.0.into(), 325.8.into()];

    assert!(chonk1.eq(&chonk2));
}

#[test]
fn partial_eq_ne() {
    let chonk1: Vechonk<f32> = vechonk![235.0.into(), 325.9.into()];
    let chonk2: Vechonk<f32> = vechonk![235.0.into(), 325.8.into()];

    assert!(!chonk1.eq(&chonk2));
}

#[test]
fn eq_eq() {
    let chonk1: Vechonk<str> = vechonk!["hello".into(), "uwu".into()];
    let chonk2: Vechonk<str> = vechonk!["hello".into(), "uwu".into()];

    assert!(chonk1.eq(&chonk2));
}

#[test]
fn eq_ne() {
    let chonk1: Vechonk<str> = vechonk!["hello".into(), "uwu".into()];
    let chonk2: Vechonk<str> = vechonk!["hewwo".into(), "owo".into()];

    assert!(!chonk1.eq(&chonk2));
}

#[test]
fn get_mut() {
    let mut chonk1: Vechonk<str> = vechonk!["hello".into(), "uwu".into()];

    let hello = chonk1.get_mut(0).unwrap();

    assert_eq!(&*hello, "hello");
}

#[test]
fn get_mut_mutating() {
    let mut chonk1: Vechonk<dyn TakeMut> = Vechonk::new();
    chonk1.push(Box::new("hello"));
    chonk1.push(Box::new("uwu"));

    let hello = chonk1.get_mut(0).unwrap();

    hello.take_mut();

    assert_eq!(chonk1.len(), 2);
}

#[test]
fn insert() {
    let mut chonk: Vechonk<str> = vechonk!["hello".into(), "uwu".into()];

    chonk.try_replace(0, "owo".into()).unwrap();

    assert_eq!(&chonk[0], "owo");
}

#[test]
fn zst_with_capacity() {
    let _ = Vechonk::<()>::with_capacity(96);
}

#[test]
fn zst_push() {
    let mut chonk = Vechonk::<()>::with_capacity(96);
    chonk.push(().into());

    assert_eq!(chonk.len(), 1);
}

#[test]
fn zst_realloc() {
    let mut chonk = Vechonk::new();
    chonk.push(().into());
    assert_eq!(chonk.len(), 1);
}

#[test]
fn zst_replace() {
    let mut chonk = Vechonk::new();
    chonk.push(().into());
    assert_eq!(chonk.len(), 1);
    let mut old = chonk.try_replace(0, ().into()).unwrap();
    old.take_mut();
    assert_eq!(chonk.len(), 1);
}

#[test]
fn empty_slice_replace() {
    let mut chonk = Vechonk::<[u8]>::new();
    chonk.push([].into());
    assert_eq!(chonk.len(), 1);
    let mut old = chonk.try_replace(0, [].into()).unwrap();
    assert_eq!(chonk.len(), 1);

    drop(chonk);

    old.take_mut();
}

#[test]
fn iter_mut() {
    fn b(x: usize) -> Box<dyn Decrement> {
        Box::new(x)
    }

    let mut chonk: Vechonk<dyn Decrement> = vechonk![b(1), b(2), b(3)];

    chonk.iter_mut().for_each(|elem| elem.decrement());

    chonk.iter().enumerate().for_each(|(i, elem)| {
        assert_eq!(i, elem.value());
    });
}

#[test]
fn iter_sizes() {
    let mut chonk: Vechonk<str> = vechonk!["hello".into(), "uwu".into(), "owo".into()];

    let iter = chonk.iter();
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.len(), 3);

    let iter = chonk.iter_mut();
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.len(), 3);

    let iter = chonk.into_iter();
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.len(), 3);
}

#[test]
fn partial_ord() {
    fn b(x: u64) -> Box<u64> {
        Box::new(x)
    }

    let chonk1 = vechonk![b(4), b(2)];
    let chonk2 = vechonk![b(4), b(3)];

    assert_eq!(
        PartialOrd::partial_cmp(&chonk1, &chonk2),
        Some(Ordering::Less)
    );
}

#[test]
fn ord() {
    fn b(x: u64) -> Box<u64> {
        Box::new(x)
    }

    let chonk1 = vechonk![b(4), b(2)];
    let chonk2 = vechonk![b(4), b(3)];

    assert_eq!(Ord::cmp(&chonk1, &chonk2), Ordering::Less);
}

#[test]
fn hash() {
    fn hash<H: Hash>(h: H) -> u64 {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        h.hash(&mut hasher);
        hasher.finish()
    }

    let chonk1: Vechonk<str> = vechonk!["hello".into(), "uwu".into()];
    let chonk2: Vechonk<str> = vechonk!["hello".into(), "uwu".into()];

    assert_eq!(hash(chonk1), hash(chonk2));
}

#[test]
fn get() {
    let mut chonk: Vechonk<str> = vechonk!["hello".into(), "uwu".into()];

    assert_eq!(chonk.get(0), Some("hello"));
    assert_eq!(chonk.get(1), Some("uwu"));
    assert_eq!(chonk.get(2), None);
    assert_eq!(chonk.get_mut(0).map(|s| &*s), Some("hello"));
    assert_eq!(chonk.get_mut(1).map(|s| &*s), Some("uwu"));
    assert_eq!(chonk.get_mut(2).map(|s| &*s), None);
}

#[test]
fn index_mut() {
    fn b(x: usize) -> Box<dyn Decrement> {
        Box::new(x)
    }

    let mut chonk: Vechonk<dyn Decrement> = vechonk![b(1), b(2), b(3)];
    chonk[2].decrement();

    assert_eq!(chonk[2].value(), 2);
}

#[test]
#[should_panic]
fn index_mut_out_of_bounds() {
    fn b(x: usize) -> Box<dyn Decrement> {
        Box::new(x)
    }

    let mut chonk: Vechonk<dyn Decrement> = vechonk![b(1), b(2), b(3)];

    chonk[3].decrement();
}
