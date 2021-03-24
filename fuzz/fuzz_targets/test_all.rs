#![no_main]
#![feature(maybe_uninit_slice)]

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};

use core::ptr;

use core::borrow::Borrow;
use core::convert::AsRef;
use core::cmp::{PartialOrd, Ord};
use core::mem::MaybeUninit;
use core::ops::Deref;

use slice_n::*;

fuzz_target!(|data: &[u8]| {
    match <(Box<[u8]>, Box<[u8]>)>::arbitrary(&mut Unstructured::new(data)) {
        Ok((a, b)) => {
            match Slice1::from_slice(&a[..]) {
                None => assert_eq!(a.len(), 0),
                Some(x) => match Slice1::from_slice(&b[..]) {
                    None => assert_eq!(b.len(), 0),
                    Some(y) => {
                        assert_eq!(x.len().get(), x.len_());

                        assert_eq!((x as *const Slice1<u8>) as *const () as usize, (&a[..] as *const [u8]) as *const () as usize);
                        assert_eq!(x.slice.len(), a.len() - 1);
                        assert_eq!(x.len_(), a.len());

                        let x_unchecked = unsafe { Slice1::from_slice_unchecked(&a[..]) };
                        assert!(ptr::eq(x_unchecked as *const Slice1<u8>, x as *const Slice1<u8>));
                        assert_eq!(x_unchecked.len(), x.len());

                        let x_maybe_uninit = x.as_maybe_uninit();
                        assert!(ptr::eq(unsafe { MaybeUninit::slice_assume_init_ref(x_maybe_uninit) } as *const [u8], &a[..] as *const [u8]));

                        assert_eq!(x.first(), (&a[..]).first().unwrap());
                        assert_eq!(x.last(), (&a[..]).last().unwrap());

                        let x_raw = unsafe { from_raw_parts_unchecked(&a[0] as *const u8, a.len()) };
                        assert!(ptr::eq(x_raw as *const Slice1<u8>, x as *const Slice1<u8>));
                        assert_eq!(x_raw.len(), x.len());

                        assert_eq!(x.as_ref(), &a[..]);
                        assert_eq!(<Slice1<u8> as Borrow<[u8]>>::borrow(x), &a[..]);
                        assert_eq!(x.deref(), &a[..]);

                        assert_eq!(x == y, &a[..] == &b[..]);
                        assert_eq!(x.partial_cmp(y), (&a[..]).partial_cmp(&b[..]));
                        assert_eq!(x < y, &a[..] < &b[..]);
                        assert_eq!(x <= y, &a[..] <= &b[..]);
                        assert_eq!(x > y, &a[..] > &b[..]);
                        assert_eq!(x >= y, &a[..] >= &b[..]);
                        assert_eq!(x.cmp(y), (&a[..]).cmp(&b[..]));
                    }
                }
            }
        }
        _ => {}
    }
});
