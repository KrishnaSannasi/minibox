use super::MiniBox;

use core::ptr::NonNull;
use std::boxed::Box;
use std::rc::Rc;
use std::sync::Arc;

#[cfg(feature = "nightly")]
impl<T: Default> Default for MiniBox<T> {
    #[inline]
    default fn default() -> Self {
        Self::with(T::default)
    }
}

#[cfg(feature = "nightly")]
impl<T: Zeroable + Default> Default for MiniBox<T> {
    fn default() -> Self {
        Self::new_zeroed()
    }
}

impl<T: Zeroable> MiniBox<T> {
    /// intializes a new `MiniBox` with
    pub fn new_zeroed() -> Self {
        unsafe { Self::new_zeroed_unchecked().assume_init() }
    }
}

/// a type that is safe to be zeroed
///
/// # Safety
///
/// all zero bytes must be a valid bit pattern for the given type
#[cfg_attr(feature = "nightly", marker)]
pub unsafe trait Zeroable {}

// arrays

macro_rules! array {
    ($($size:expr),*) => {
        $(unsafe impl<T: Zeroable> Zeroable for [T; $size] {})*
    };
}

macro_rules! tuple {
    ($($ident:ident)*) => {
        tuple!{@next $($ident)*}
        unsafe impl<$($ident: Zeroable),*> Zeroable for ($($ident,)*) {}
    };

    (@next ) => {};
    (@next $first:ident $($ident:ident)*) => {
        tuple!{$($ident)*}
    };
}

// integers
unsafe impl Zeroable for u8 {}
unsafe impl Zeroable for u16 {}
unsafe impl Zeroable for u32 {}
unsafe impl Zeroable for u64 {}
unsafe impl Zeroable for u128 {}
unsafe impl Zeroable for usize {}
unsafe impl Zeroable for i8 {}
unsafe impl Zeroable for i16 {}
unsafe impl Zeroable for i32 {}
unsafe impl Zeroable for i64 {}
unsafe impl Zeroable for i128 {}
unsafe impl Zeroable for isize {}

// non-null
unsafe impl<T: ?Sized> Zeroable for Option<&T> {}
unsafe impl<T: ?Sized> Zeroable for Option<&mut T> {}
unsafe impl<T: ?Sized> Zeroable for Option<NonNull<T>> {}
unsafe impl<T: ?Sized> Zeroable for Option<Box<T>> {}
unsafe impl<T: ?Sized> Zeroable for Option<Rc<T>> {}
unsafe impl<T: ?Sized> Zeroable for Option<Arc<T>> {}
unsafe impl<T> Zeroable for Option<Vec<T>> {}

tuple! { A B C D E F G H I J K L M N O P }

unsafe impl<T> Zeroable for [T; 0] {}
#[cfg(feature = "nightly")]
unsafe impl<T: Zeroable, const N: usize> Zeroable for [T; N] {}

#[cfg(not(feature = "nightly"))]
array! {
    1, 2, 3, 4, 5, 6, 7, 8,
    9, 10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24,
    25, 26, 27, 28, 29, 30, 31, 32,
    64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536
}
