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
    pub fn zeroed() -> Self {
        unsafe { Self::new_zeroed().assume_init() }
    }
}

/// a safe wrapper of core::mem::zeroed
pub fn zeroed<T: Zeroable>() -> T {
    unsafe { core::mem::zeroed() }
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

// non-zero

unsafe impl Zeroable for Option<std::num::NonZeroU8> {}
unsafe impl Zeroable for Option<std::num::NonZeroU16> {}
unsafe impl Zeroable for Option<std::num::NonZeroU32> {}
unsafe impl Zeroable for Option<std::num::NonZeroU64> {}
unsafe impl Zeroable for Option<std::num::NonZeroU128> {}
unsafe impl Zeroable for Option<std::num::NonZeroUsize> {}
unsafe impl Zeroable for Option<std::num::NonZeroI8> {}
unsafe impl Zeroable for Option<std::num::NonZeroI16> {}
unsafe impl Zeroable for Option<std::num::NonZeroI32> {}
unsafe impl Zeroable for Option<std::num::NonZeroI64> {}
unsafe impl Zeroable for Option<std::num::NonZeroI128> {}
unsafe impl Zeroable for Option<std::num::NonZeroIsize> {}

unsafe impl Zeroable for core::sync::atomic::AtomicU8 {}
unsafe impl Zeroable for core::sync::atomic::AtomicU16 {}
unsafe impl Zeroable for core::sync::atomic::AtomicU32 {}
unsafe impl Zeroable for core::sync::atomic::AtomicU64 {}
unsafe impl Zeroable for core::sync::atomic::AtomicUsize {}
unsafe impl Zeroable for core::sync::atomic::AtomicI8 {}
unsafe impl Zeroable for core::sync::atomic::AtomicI16 {}
unsafe impl Zeroable for core::sync::atomic::AtomicI32 {}
unsafe impl Zeroable for core::sync::atomic::AtomicI64 {}
unsafe impl Zeroable for core::sync::atomic::AtomicIsize {}

// raw pointers

unsafe impl<T> Zeroable for core::sync::atomic::AtomicPtr<T> {}
unsafe impl<T: ?Sized> Zeroable for *const T {}
unsafe impl<T: ?Sized> Zeroable for *mut T {}

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
