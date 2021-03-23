#![no_std]
#![cfg_attr(any(feature = "alloc", feature = "std"), feature(allocator_api))]

use core::borrow::{Borrow, BorrowMut};
use core::convert::{AsRef, AsMut};
use core::cmp::{PartialEq, Eq, PartialOrd, Ord, Ordering};
use core::fmt::Debug;
use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};
use core::slice;

/// A slice guaranteed to contain at least `N` elements.
#[repr(C)]
pub struct SliceN<T, const N: usize> {
    pub arr: [T; N],
    pub slice: [T],
}

/// A slice guaranteed to contain at least one element.
pub type Slice1<T> = SliceN<T, 1>;

impl<T, const N: usize> SliceN<T, N> {
    /// Converts a regular slice into a `SliceN` if it is long enough.
    pub fn from_slice(s: &[T]) -> Option<&Self> {
        if s.len() >= N {
            Some(unsafe { Self::from_slice_unchecked(s) })
        } else {
            None
        }
    }

    /// Converts a regular mutable slice into a `SliceN` if it is long enough.
    pub fn from_slice_mut(s: &mut [T]) -> Option<&mut Self> {
        if s.len() >= N {
            Some(unsafe { Self::from_slice_unchecked_mut(s) })
        } else {
            None
        }
    }

    /// Converts a regular slice into a `SliceN` without checking that its length is sufficient.
    pub unsafe fn from_slice_unchecked(s: &[T]) -> &Self {
        &*core::mem::transmute::<_, *const Self>(s)
    }

    /// Converts a regular mutable slice into a `SliceN` without checking that its length is
    /// sufficient.
    pub unsafe fn from_slice_unchecked_mut(s: &mut[T]) -> &mut Self {
        &mut *core::mem::transmute::<_, *mut Self>(s)
    }

    pub fn as_maybe_uninit(&self) -> &SliceN<MaybeUninit<T>, N> {
        let ptr = self.as_ptr().cast::<MaybeUninit<T>>();
        unsafe { from_raw_parts_unchecked(ptr, self.len()) }
    }

    pub fn as_maybe_uninit_mut(&mut self) -> &mut SliceN<MaybeUninit<T>, N> {
        let ptr = self.as_mut_ptr().cast::<MaybeUninit<T>>();
        unsafe { from_raw_parts_unchecked_mut(ptr, self.len()) }
    }
}

// Rust does not yet support implementing these for all sizes greater than zero.
impl<T> SliceN<T, 1> {
    /// Returns the number of elements in the slice as a `NonZeroUsize`.
    pub fn len(&self) -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(self.as_ref().len()) }
    }

    /// Returns the number of elements in the slice as a `usize`.
    pub fn len_(&self) -> usize {
        self.as_ref().len()
    }

    /// Returns a reference to the first element in the slice.
    pub fn first(&self) -> &T {
        &self.arr[0]
    }

    /// Returns a mutable reference to the first element in the slice.
    pub fn first_mut(&mut self) -> &mut T {
        &mut self.arr[0]
    }

    /// Returns a reference to the last element in the slice.
    pub fn last(&self) -> &T {
        self.as_ref().last().unwrap()
    }

    /// Returns a mutable reference to the last element in the slice.
    pub fn last_mut(&mut self) -> &mut T {
        self.as_mut().last_mut().unwrap()
    }

    /// Returns the first and all the rest of the elements of the slice.
    pub fn split_first(&self) -> (&T, &[T]) {
        self.as_ref().split_first().unwrap()
    }

    /// Returns the first and all the rest of the elements of the slice.
    pub fn split_first_mut(&mut self) -> (&mut T, &mut [T]) {
        self.as_mut().split_first_mut().unwrap()
    }

    /// Returns the last and all the rest of the elements of the slice.
    pub fn split_last(&self) -> (&T, &[T]) {
        self.as_ref().split_last().unwrap()
    }

    /// Returns the last and all the rest of the elements of the slice.
    pub fn split_last_mut(&mut self) -> (&mut T, &mut [T]) {
        self.as_mut().split_last_mut().unwrap()
    }
}

/// Converts a reference to T into a `Slice1` of length 1 (without copying).
pub fn from_mut<T>(s: &mut T) -> &mut Slice1<T> {
    unsafe { Slice1::from_slice_unchecked_mut(slice::from_mut(s)) }
}

/// Forms a `SliceN` from a pointer and a length, without checking the length.
///
/// In addition to the length being too low, this is just as unsafe as
/// `core::slice::from_raw_parts`.
pub unsafe fn from_raw_parts_unchecked<'a, T, const N: usize>(data: *const T, len: usize) -> &'a SliceN<T, N> {
    SliceN::from_slice_unchecked(slice::from_raw_parts(data, len))
}

/// Forms a `SliceN` from a pointer and a length if it is long enough.
///
/// In addition to the length being too low, this is just as unsafe as
/// `core::slice::from_raw_parts`.
pub unsafe fn from_raw_parts<'a, T, const N: usize>(data: *const T, len: usize) -> Option<&'a SliceN<T, N>> {
    SliceN::from_slice(slice::from_raw_parts(data, len))
}

/// Forms a `SliceN` from a mutable pointer and a length, without checking the length.
///
/// In addition to the length being too low, this is just as unsafe as
/// `core::slice::from_raw_parts`.
pub unsafe fn from_raw_parts_unchecked_mut<'a, T, const N: usize>(data: *mut T, len: usize) -> &'a mut SliceN<T, N> {
    SliceN::from_slice_unchecked_mut(slice::from_raw_parts_mut(data, len))
}

/// Forms a `SliceN` from a mutable pointer and a length if it is long enough.
///
/// In addition to the length being too low, this is just as unsafe as
/// `core::slice::from_raw_parts`.
pub unsafe fn from_raw_parts_mut<'a, T, const N: usize>(data: *mut T, len: usize) -> Option<&'a mut SliceN<T, N>> {
    SliceN::from_slice_mut(slice::from_raw_parts_mut(data, len))
}

impl<T, const N: usize> AsRef<[T]> for SliceN<T, N> {
    fn as_ref(&self) -> &[T] {
        unsafe { slice::from_raw_parts((self as *const SliceN<T, N>).cast(), N + self.slice.len()) }
    }
}

impl<T, const N: usize> AsMut<[T]> for SliceN<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut((self as *mut SliceN<T, N>).cast(), N + self.slice.len()) }
    }
}

impl<T, const N: usize> Borrow<[T]> for SliceN<T, N> {
    fn borrow(&self) -> &[T] {
        self.as_ref()
    }
}

impl<T, const N: usize> BorrowMut<[T]> for SliceN<T, N> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

impl<T, const N: usize> Deref for SliceN<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_ref()
    }
}

impl<T, const N: usize> DerefMut for SliceN<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

impl<T: Debug, const N: usize> Debug for SliceN<T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        self.as_ref().fmt(f)
    }
}

impl<T, S, const N: usize, const M: usize> PartialEq<SliceN<S, M>> for SliceN<T, N> where T: PartialEq<S> {
    fn eq(&self, other: &SliceN<S, M>) -> bool {
        self.as_ref().eq(other.as_ref())
    }

    fn ne(&self, other: &SliceN<S, M>) -> bool {
        self.as_ref().ne(other.as_ref())
    }
}

impl<A, B, const N: usize, const M: usize> PartialEq<[A; M]> for SliceN<B, N> where
B: PartialEq<A> {
    fn eq(&self, other: &[A; M]) -> bool {
        self.as_ref().eq(other)
    }
}

impl<'a, A, B, const N: usize, const M: usize> PartialEq<[A; M]> for &'a SliceN<B, N> where
B: PartialEq<A> {
    fn eq(&self, other: &[A; M]) -> bool {
        self.as_ref().eq(other)
    }
}

impl<'a, A, B, const N: usize, const M: usize> PartialEq<[A; M]> for &'a mut SliceN<B, N> where
B: PartialEq<A> {
    fn eq(&self, other: &[A; M]) -> bool {
        self.as_ref().eq(other)
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
mod alloc_impls {
    use maybe_std::{
        alloc::Allocator,
        vec::Vec,
    };

    use super::*;

    impl<T, U, A, const N: usize> PartialEq<Vec<U, A>> for SliceN<T, N> where
    T: PartialEq<U>, A: Allocator {
        fn eq(&self, other: &Vec<U, A>) -> bool {
            self.as_ref().eq(other)
        }
    }

    impl<'a, T, U, A, const N: usize> PartialEq<Vec<U, A>> for &'a SliceN<T, N> where
    T: PartialEq<U>, A: Allocator {
        fn eq(&self, other: &Vec<U, A>) -> bool {
            self.as_ref().eq(other)
        }
    }

    impl<'a, T, U, A, const N: usize> PartialEq<Vec<U, A>> for &'a mut SliceN<T, N> where
    T: PartialEq<U>, A: Allocator {
        fn eq(&self, other: &Vec<U, A>) -> bool {
            self.as_ref().eq(other)
        }
    }
}

impl<T, const N: usize> Eq for SliceN<T, N> where T: Eq {}

impl<T, const N: usize> PartialOrd for SliceN<T, N> where T: PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }

    fn lt(&self, other: &Self) -> bool {
        self.as_ref().lt(other.as_ref())
    }

    fn le(&self, other: &Self) -> bool {
        self.as_ref().le(other.as_ref())
    }

    fn gt(&self, other: &Self) -> bool {
        self.as_ref().gt(other.as_ref())
    }

    fn ge(&self, other: &Self) -> bool {
        self.as_ref().ge(other.as_ref())
    }
}

impl<T, const N: usize> Ord for SliceN<T, N> where T: Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other)
    }
}

impl<T, const N: usize> Hash for SliceN<T, N> where T: Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}
