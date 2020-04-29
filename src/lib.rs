#![forbid(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    feature = "nightly",
    feature(
        seek_convenience,
        backtrace,
        unboxed_closures,
        fn_traits,
        const_fn,
        const_panic,
        const_if_match,
        const_transmute,
        specialization,
        const_generics,
        marker_trait_attr
    )
)]

//! # minibox
//!
//! a small box implementation that packs the doesn't allocate if the value is layout compatible with
//! a pointer. (i.e. if it is not larger or more aligned than a pointer). This is almost a drop-in replacement
//! for `Box<T>` when `T: Sized`. `T: !Sized` is not supported.
//!
//! ```rust
//! # use minibox::MiniBox;
//! struct Foo {
//!     a: u8,
//!     b: u32,
//!     _align: [usize; 0],
//! }
//!
//! let bx = MiniBox::new(Foo { a: 31, b: 0x90abcdef, _align: [] });
//! assert_eq!(bx.a, 31);
//! assert_eq!(bx.b, 0x90abcdef);
//!
//! let addr_0 = &bx.a as *const u8 as usize;
//!
//! let bx = bx;
//!
//! let addr_1 = &bx.a as *const u8 as usize;
//!
//! // this is because `Foo` is smaller than a pointer, and is pointer aligned
//! // so it is stored in the `bx` inline (with no allocation)
//! assert_ne!(addr_0, addr_1);
//! ```
//!
//! If the type is not layout compatible with a pointer, then it is heap allocated
//!
//! ```rust
//! # use minibox::MiniBox;
//! let bx = MiniBox::<[u8; 1024]>::zeroed();
//! assert!(bx.iter().all(|&x| x == 0));
//!
//! let addr_0 = &*bx as *const [u8; 1024] as usize;
//!
//! let bx = bx;
//!
//! let addr_1 = &*bx as *const [u8; 1024] as usize;
//!
//! // this is because `Foo` is larger than a pointer, so it is heap allocated
//! assert_eq!(addr_0, addr_1);
//! ```
//!
//! ```rust
//! # use minibox::MiniBox;
//! #[repr(align(64))]
//! struct Overaligned(u8);
//!
//! let bx = MiniBox::with(|| Overaligned(31));
//! assert_eq!(bx.0, 31);
//!
//! let addr_0 = &*bx as *const Overaligned as usize;
//!
//! let bx = bx;
//!
//! let addr_1 = &*bx as *const Overaligned as usize;
//!
//! // this is because `Foo` is more aligned than a pointer, it is heap allocated
//! assert_eq!(addr_0, addr_1);
//! ```
//!
//! If the type is zero-sized, then regardless of it's alignment, it is never allocated
//! but they are guaranteed to be aligned
//!
//! ```rust
//! # use minibox::MiniBox;
//! #[repr(align(64))]
//! struct Overaligned;
//!
//! // no allocation
//! let bx = MiniBox::new(Overaligned);
//! ```

#[cfg(not(feature = "std"))]
extern crate alloc as std;

use core::marker::PhantomData;
use core::mem;
use core::mem::MaybeUninit;
use std::boxed::Box;

mod default;
#[cfg(feature = "serde")]
mod serde;
mod trait_impls;

pub use default::{zeroed, Zeroable};

const fn dangling<T>() -> *mut T {
    core::mem::align_of::<T>() as *mut T
}

/// A box equivalent that stores the value inline if it is layout compatible with a pointer
///
/// see crate docs for more information
#[repr(transparent)]
pub struct MiniBox<T> {
    ptr: MaybeUninit<*const T>,
    drop: PhantomData<T>,
}

/// A raw pointer equivalent that stores the value inline if it is layout compatible with a pointer
///
/// In order for this `MiniPtr` to be safe to use, it must abide by the following
/// rules based on `T`'s `SizeClass`
///
/// * `SizeClass::Zero` - the pointer has no requirements (may even be uninitialized)
/// * `SizeClass::Inline` - the pointer must be store an initialized `T`
/// * `SizeClass::Boxed` - the pointer must be store a pointer to a heap
///     allocated `T` that is allocated with the global allocator
#[repr(transparent)]
pub struct MiniPtr<T>(pub MaybeUninit<*const T>);

impl<T> Copy for MiniPtr<T> {}
impl<T> Clone for MiniPtr<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// The storage strategy of a `MiniBox`/`MiniPtr`
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SizeClass {
    /// The pointer is guaranteed to be aligned and initialized, and
    /// `MiniBox` will not allocate
    Zero = 0,

    /// The value will be stored inline, and `MiniBox` will not allocate
    /// The pointer may contain uninitialized bytes if `T` contains any
    /// `MaybeUninit` or padding bytes
    Inline = 1,

    /// The value is allocated on the heap, and the pointer is guaranteed
    /// to be aligned and initialized
    Boxed = 2,
}

impl SizeClass {
    /// Get the storage strategy for the given type
    #[inline]
    pub const fn new<T>() -> Self {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        let size_ptr = mem::size_of::<*mut ()>();
        let align_ptr = mem::align_of::<*mut ()>();

        #[cfg(feature = "nightly")]
        {
            if size == 0 {
                SizeClass::Zero
            } else if size <= size_ptr && align <= align_ptr {
                SizeClass::Inline
            } else {
                SizeClass::Boxed
            }
        }

        #[cfg(not(feature = "nightly"))]
        {
            [
                [SizeClass::Zero, SizeClass::Zero],
                [SizeClass::Inline, SizeClass::Boxed],
                [SizeClass::Boxed, SizeClass::Boxed],
            ][(size != 0) as usize * ((align > align_ptr) as usize + 1)]
                [(size > size_ptr) as usize]
        }
    }
}

impl<T> MiniPtr<T> {
    /// The size class for `T`
    pub const SIZE_CLASS: SizeClass = SizeClass::new::<T>();

    /// Create a new `MiniPtr` from the given raw pointer
    #[cfg(not(feature = "nightly"))]
    #[inline]
    pub unsafe fn from_raw(ptr: usize) -> Self {
        mem::transmute(ptr)
    }

    /// Create a new `MiniPtr` from the given raw pointer
    #[cfg(feature = "nightly")]
    #[inline]
    pub const unsafe fn from_raw(ptr: usize) -> Self {
        mem::transmute(ptr)
    }

    /// Get a word from the underlying pointer, note this may not be the pointer you provided in `from_raw`
    /// if `T`'s `SizeClass` is `Zero`
    ///
    /// # Safety
    ///
    /// One of
    /// * `T`'s `SizeClass` is `Zero`
    /// * the underlying pointer must not contain any uninitialized bytes
    #[inline]
    pub unsafe fn to_raw(self) -> usize {
        match Self::SIZE_CLASS {
            SizeClass::Zero => core::mem::align_of::<T>(),
            SizeClass::Inline | SizeClass::Boxed => self.0.assume_init() as usize,
        }
    }

    /// Get a reference to the underlying value
    ///
    /// # Safety
    ///
    /// The safety rules described in the type-level documentation must be followed
    #[inline]
    pub unsafe fn as_ref(&self) -> &T {
        match Self::SIZE_CLASS {
            SizeClass::Zero => &*dangling::<T>(),
            SizeClass::Inline => &*(self as *const Self as *const T),
            SizeClass::Boxed => &*self.0.assume_init(),
        }
    }

    /// Get a mutable reference to the underlying value
    ///
    /// # Safety
    ///
    /// The safety rules described in the type-level documentation must be followed
    #[inline]
    pub unsafe fn as_mut(&mut self) -> &mut T {
        match Self::SIZE_CLASS {
            SizeClass::Zero => &mut *dangling::<T>(),
            SizeClass::Inline => &mut *(self as *mut Self as *mut T),
            SizeClass::Boxed => &mut *(self.0.assume_init() as *mut T),
        }
    }
}

impl<T> MiniBox<T> {
    /// The size class for `T`
    pub const SIZE_CLASS: SizeClass = SizeClass::new::<T>();

    /// Create a new `MiniBox<T>`
    #[inline]
    pub fn new(value: T) -> Self {
        Self::new_uninit().write(value)
    }

    /// Create a new `MiniBox<T>`
    #[inline]
    pub fn with<F: FnOnce() -> T>(value: F) -> Self {
        Self::new_uninit().write(value())
    }

    /// Create a new `MiniBox<T>`
    ///
    /// # Panic
    ///
    /// if `T` is not zero-sized, this function will panic
    #[inline]
    pub const fn new_zst(value: T) -> Self {
        #[cfg(not(feature = "nightly"))]
        [()][Self::SIZE_CLASS as usize];

        #[cfg(feature = "nightly")]
        {
            match Self::SIZE_CLASS {
                SizeClass::Zero => (),
                _ => panic!("The size class of `T` must be `Zero`"),
            }
        }

        // core::mem::forget is not a const-fn
        core::mem::ManuallyDrop::new(value);

        Self {
            ptr: MaybeUninit::uninit(),
            drop: PhantomData,
        }
    }

    /// Create a new uninitialized `MiniBox<T>`
    ///
    /// # Panic
    ///
    /// if the `SizeClass` of `T` is `SizeClass::Boxed`, this function will panic
    #[inline]
    pub const fn new_zeroed_inline() -> MiniBox<MaybeUninit<T>> {
        let ptr =
            [MaybeUninit::uninit(), MaybeUninit::new(core::ptr::null())][Self::SIZE_CLASS as usize];

        MiniBox {
            ptr,
            drop: PhantomData,
        }
    }

    /// Create a new uninitialized `MiniBox<T>`
    pub fn new_uninit() -> MiniBox<MaybeUninit<T>> {
        Self::with_alloc(std::alloc::alloc)
    }

    /// Create a new uninitialized `MiniBox<T>`
    pub fn new_zeroed() -> MiniBox<MaybeUninit<T>> {
        Self::with_alloc(std::alloc::alloc_zeroed)
    }

    #[inline]
    fn with_alloc(alloc: unsafe fn(std::alloc::Layout) -> *mut u8) -> MiniBox<MaybeUninit<T>> {
        match Self::SIZE_CLASS {
            SizeClass::Zero | SizeClass::Inline => Self::new_zeroed_inline(),
            SizeClass::Boxed => {
                use std::alloc::{handle_alloc_error, Layout};

                let layout = Layout::new::<T>();
                let ptr = unsafe { alloc(layout).cast::<MaybeUninit<T>>() };
                if ptr.is_null() {
                    handle_alloc_error(layout);
                }

                MiniBox {
                    ptr: MaybeUninit::new(ptr),
                    drop: PhantomData,
                }
            }
        }
    }

    /// Create a new uninitialized `MiniBox<T>` from the given pointer
    ///
    /// # Safety
    ///
    /// The safety rules described on `MiniPtr`'s type-level documentation must be followed
    /// This provided `MiniPtr` must not be used after this function
    #[inline]
    pub const unsafe fn from_ptr(MiniPtr(ptr): MiniPtr<T>) -> Self {
        Self {
            ptr,
            drop: PhantomData,
        }
    }

    /// Convert the box into a `MiniPtr` without deallocating or dropping the underlying value
    ///
    /// The provided `MiniPtr<T>` is guaranteed is be safe to pass to `MiniBox::from_ptr`
    #[inline]
    pub const fn into_ptr(bx: Self) -> MiniPtr<T> {
        let ptr = bx.ptr;
        core::mem::ManuallyDrop::new(bx);
        MiniPtr(ptr)
    }

    /// Consume the `MiniBox` returning the underlying data.
    pub fn into_inner(bx: Self) -> T {
        unsafe {
            let ptr = Self::into_ptr(bx);
            match Self::SIZE_CLASS {
                SizeClass::Zero => dangling::<T>().read(),
                SizeClass::Inline => core::ptr::read(ptr.as_ref()),
                SizeClass::Boxed => *Box::from_raw(ptr.0.assume_init() as *mut T),
            }
        }
    }

    #[inline]
    /// project through a `Pin` to get the underlying value
    pub fn deref_pin_mut(bx: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut T> {
        use core::pin::Pin;
        unsafe { Pin::new_unchecked(Pin::into_inner_unchecked(bx) as &mut T) }
    }
}

impl<T> MiniBox<MaybeUninit<T>> {
    /// Consume and initialize the `MiniBox<MaybeUninit<T>>`. This overwrites any previous value without dropping it.
    /// Returns the initialized `MiniBox<T>`
    #[inline]
    pub fn write(mut self, value: T) -> MiniBox<T> {
        unsafe {
            self.as_mut_ptr().write(value);
            self.assume_init()
        }
    }

    /// Extracts the value from the `MiniBox<MaybeUninit<T>>` container. This is a great way to ensure
    /// that the data will get dropped, because the resulting T is subject to the usual drop handling.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that the `MiniBox<MaybeUninit<T>>` really is in an initialized state.
    /// Calling this when the content is not yet fully initialized causes immediate undefined behavior.
    ///
    /// see `MaybeUninit<T>` for more information the initialization invariant
    #[inline]
    pub unsafe fn assume_init(self) -> MiniBox<T> {
        mem::transmute(self)
    }
}

impl<T> Drop for MiniBox<T> {
    fn drop(&mut self) {
        unsafe {
            match Self::SIZE_CLASS {
                SizeClass::Zero => dangling::<T>().drop_in_place(),
                SizeClass::Inline => self.ptr.as_mut_ptr().cast::<T>().drop_in_place(),
                SizeClass::Boxed => {
                    dbg!();
                    drop(Box::from_raw(self.ptr.assume_init() as *mut T))
                }
            }
        }
    }
}

impl<T> core::ops::Deref for MiniBox<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe {
            match Self::SIZE_CLASS {
                SizeClass::Zero => &*dangling::<T>(),
                SizeClass::Inline => &*(self as *const Self as *const T),
                SizeClass::Boxed => &*self.ptr.assume_init(),
            }
        }
    }
}

impl<T> core::ops::DerefMut for MiniBox<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            match Self::SIZE_CLASS {
                SizeClass::Zero => &mut *dangling::<T>(),
                SizeClass::Inline => &mut *(self as *mut Self as *mut T),
                SizeClass::Boxed => &mut *(self.ptr.assume_init() as *mut T),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[repr(align(64))]
    pub struct OverAlignedZeroSized;
    #[repr(align(64))]
    pub struct OverAlignedByte(u8);

    #[test]
    fn size_class() {
        assert!(matches!(SizeClass::new::<()>(), SizeClass::Zero));
        assert!(matches!(
            SizeClass::new::<OverAlignedZeroSized>(),
            SizeClass::Zero
        ));

        assert!(matches!(SizeClass::new::<u8>(), SizeClass::Inline));
        assert!(matches!(
            SizeClass::new::<OverAlignedByte>(),
            SizeClass::Boxed
        ));

        assert!(matches!(SizeClass::new::<usize>(), SizeClass::Inline));
        assert!(matches!(SizeClass::new::<[usize; 2]>(), SizeClass::Boxed));
    }

    #[test]
    #[should_panic]
    #[cfg(not(miri))]
    pub fn nonzerosized_const() {
        MiniBox::new_zst(0);
    }

    #[test]
    pub fn zst() {
        let bx = MiniBox::new_zst(());
        assert_eq!(*bx, ());

        let bx = MiniBox::new_zst(OverAlignedZeroSized);
        assert!(matches!(*bx, OverAlignedZeroSized));
    }

    #[test]
    pub fn overaligned() {
        let bx = MiniBox::new(OverAlignedByte(3));
        assert_eq!(bx.0, 3);
    }

    #[test]
    pub fn boxed() {
        let bx = MiniBox::new([3_u8; 32]);
        assert_eq!(*bx, [3; 32]);

        let bx = MiniBox::<[u8; 32]>::zeroed();
        assert_eq!(*bx, [0; 32]);
    }

    #[test]
    pub fn inline() {
        let bx = MiniBox::new(3_u8);
        assert_eq!(*bx, 3);

        let bx = MiniBox::<u8>::zeroed();
        assert_eq!(*bx, 0);
    }

    #[test]
    pub fn word() {
        let bx = MiniBox::new(3_usize);
        assert_eq!(*bx, 3);

        let bx = MiniBox::<usize>::zeroed();
        assert_eq!(*bx, 0);
    }

    #[test]
    fn test_uninitialized_minibox_new() {
        type Uninint = core::mem::MaybeUninit<usize>;

        let _sto = MiniBox::new(Uninint::uninit());
    }

    #[test]
    fn test_ref_from_miniptr_small() {
        let value: u16 = 173;
        let storage = MiniBox::into_ptr(MiniBox::new(value));

        {
            let value_ref_1: &u16 = unsafe { storage.as_ref() };
            let value_ref_2: &u16 = unsafe { storage.as_ref() };

            assert_eq!(*value_ref_1, 173);
            assert_eq!(*value_ref_2, 173);
        }

        // drop stowed
        unsafe { MiniBox::from_ptr(storage) };
    }

    #[test]
    fn test_ref_from_miniptr_large() {
        use std::vec::Vec;

        let value: Vec<i64> = vec![3245, 5675, 4653, 1234, 7345];

        let storage = MiniBox::into_ptr(MiniBox::new(value));

        {
            let value_ref_1: &Vec<i64> = unsafe { storage.as_ref() };
            let value_ref_2: &Vec<i64> = unsafe { storage.as_ref() };

            assert_eq!(**value_ref_1, [3245, 5675, 4653, 1234, 7345]);
            assert_eq!(**value_ref_2, [3245, 5675, 4653, 1234, 7345]);
        }

        // drop stowed
        unsafe { MiniBox::from_ptr(storage) };
    }
}

#[cfg(test)]
mod test_drop {
    use crate::MiniBox;
    use core::cell::Cell;
    use core::mem;
    use core::sync::atomic::{AtomicU32, Ordering};

    struct DropCounter<'a> {
        counter: &'a Cell<u32>,
    }

    impl<'a> Drop for DropCounter<'a> {
        fn drop(&mut self) {
            self.counter.set(self.counter.get() + 1);
        }
    }

    #[test]
    fn zero_size_value() {
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        #[derive(Debug)]
        struct StaticDropCounter;

        impl Drop for StaticDropCounter {
            fn drop(&mut self) {
                COUNTER.fetch_add(1, Ordering::SeqCst);
            }
        }

        {
            let value = StaticDropCounter;
            assert_eq!(COUNTER.load(Ordering::SeqCst), 0);

            let stowed_value = MiniBox::new(value);
            assert_eq!(COUNTER.load(Ordering::SeqCst), 0);

            let storage = MiniBox::into_ptr(stowed_value);
            assert_eq!(COUNTER.load(Ordering::SeqCst), 0);

            let stowed_value = unsafe { MiniBox::<StaticDropCounter>::from_ptr(storage) };
            assert_eq!(COUNTER.load(Ordering::SeqCst), 0);

            mem::drop(stowed_value);
            assert_eq!(COUNTER.load(Ordering::SeqCst), 1);
        }
    }

    #[test]
    fn small_stowed_value() {
        let counter: Cell<u32> = Cell::new(0);

        // Create a value, cycle it through the MiniBox lifecycle, and
        // ensure it was dropped exactly once.
        let value = DropCounter { counter: &counter };
        assert_eq!(counter.get(), 0);

        let stowed_value = MiniBox::new(value);
        assert_eq!(counter.get(), 0);

        let storage = MiniBox::into_ptr(stowed_value);
        assert_eq!(counter.get(), 0);

        let stowed_value = unsafe { MiniBox::<DropCounter>::from_ptr(storage) };
        assert_eq!(counter.get(), 0);

        mem::drop(stowed_value);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn small_raw_value() {
        let counter: Cell<u32> = Cell::new(0);

        // Create a value, cycle it through the MiniBox lifecycle, and
        // ensure it was dropped exactly once.
        let value = DropCounter { counter: &counter };
        assert_eq!(counter.get(), 0);

        let stowed_value = MiniBox::new(value);
        assert_eq!(counter.get(), 0);

        let storage = MiniBox::into_ptr(stowed_value);
        assert_eq!(counter.get(), 0);

        let stowed_value = unsafe { MiniBox::<DropCounter>::from_ptr(storage) };
        assert_eq!(counter.get(), 0);

        let raw_value: DropCounter = MiniBox::into_inner(stowed_value);
        assert_eq!(counter.get(), 0);

        mem::drop(raw_value);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn large_stowed_value() {
        let counter: Cell<u32> = Cell::new(0);

        // Create a large array of DropCounters, cycle it through the
        // MiniBox lifecycle, and ensure it was dropped exactly once.
        let value: [DropCounter; 16] = [
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
        ];
        assert_eq!(counter.get(), 0);

        let stowed_value = MiniBox::new(value);
        assert_eq!(counter.get(), 0);

        let storage = MiniBox::into_ptr(stowed_value);
        assert_eq!(counter.get(), 0);

        let stowed_value = unsafe { MiniBox::<[DropCounter; 16]>::from_ptr(storage) };
        assert_eq!(counter.get(), 0);

        mem::drop(stowed_value);
        assert_eq!(counter.get(), 16);
    }

    #[test]
    fn large_raw_stowed_value() {
        let counter: Cell<u32> = Cell::new(0);

        // Create a large array of DropCounters, cycle it through the
        // MiniBox lifecycle, and ensure it was dropped exactly once.
        let value: [DropCounter; 16] = [
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
            DropCounter { counter: &counter },
        ];
        assert_eq!(counter.get(), 0);

        let stowed_value = MiniBox::new(value);
        assert_eq!(counter.get(), 0);

        let storage = MiniBox::into_ptr(stowed_value);
        assert_eq!(counter.get(), 0);

        let stowed_value = unsafe { MiniBox::<[DropCounter; 16]>::from_ptr(storage) };
        assert_eq!(counter.get(), 0);

        let raw_value: [DropCounter; 16] = MiniBox::into_inner(stowed_value);
        assert_eq!(counter.get(), 0);

        drop(raw_value);
        assert_eq!(counter.get(), 16);
    }
}
