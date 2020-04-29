use super::{MiniBox, SizeClass};

use core::fmt;
use core::future::Future;
use core::hash::{Hash, Hasher};
use core::pin::Pin;
use core::task::{Context, Poll};

use std::boxed::Box;

#[cfg(feature = "std")]
use std::error::Error;
#[cfg(feature = "std")]
use std::io;

unsafe impl<T: Send> Send for MiniBox<T> {}
unsafe impl<T: Sync> Sync for MiniBox<T> {}
impl<T: core::marker::Unpin> core::marker::Unpin for MiniBox<T> {}
impl<T: core::marker::Unpin> core::marker::Unpin for super::MiniPtr<T> {}

#[cfg(not(feature = "nightly"))]
impl<T: Default> Default for MiniBox<T> {
    #[inline]
    fn default() -> Self {
        Self::with(T::default)
    }
}

impl<T> AsRef<T> for MiniBox<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for MiniBox<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T> AsRef<MiniBox<T>> for MiniBox<T> {
    #[inline]
    fn as_ref(&self) -> &MiniBox<T> {
        self
    }
}

impl<T> AsMut<MiniBox<T>> for MiniBox<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut MiniBox<T> {
        self
    }
}

impl<T> std::borrow::Borrow<T> for MiniBox<T> {
    #[inline]
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> std::borrow::BorrowMut<T> for MiniBox<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

#[cfg(feature = "std")]
impl<T: Error> Error for MiniBox<T> {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        T::source(self)
    }

    #[inline]
    #[cfg(feature = "nightly")]
    fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        T::backtrace(self)
    }
}

impl<T: Eq> Eq for MiniBox<T> {}
impl<T: PartialEq<U>, U> PartialEq<MiniBox<U>> for MiniBox<T> {
    #[inline]
    fn eq(&self, other: &MiniBox<U>) -> bool {
        T::eq(self, other)
    }
}

impl<T: PartialOrd<U>, U> PartialOrd<MiniBox<U>> for MiniBox<T> {
    #[inline]
    fn partial_cmp(&self, other: &MiniBox<U>) -> Option<core::cmp::Ordering> {
        T::partial_cmp(self, other)
    }
}

impl<T: Ord> Ord for MiniBox<T> {
    #[inline]
    fn cmp(&self, other: &MiniBox<T>) -> core::cmp::Ordering {
        T::cmp(self, other)
    }
}

impl<T: Hash> Hash for MiniBox<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        T::hash(self, state)
    }
}

impl<T: Hasher> Hasher for MiniBox<T> {
    #[inline]
    fn finish(&self) -> u64 {
        T::finish(self)
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        T::write(self, bytes)
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        T::write_u8(self, i)
    }
    #[inline]
    fn write_u16(&mut self, i: u16) {
        T::write_u16(self, i)
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        T::write_u32(self, i)
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        T::write_u64(self, i)
    }
    #[inline]
    fn write_u128(&mut self, i: u128) {
        T::write_u128(self, i)
    }
    #[inline]
    fn write_usize(&mut self, i: usize) {
        T::write_usize(self, i)
    }

    #[inline]
    fn write_i8(&mut self, i: i8) {
        T::write_i8(self, i)
    }
    #[inline]
    fn write_i16(&mut self, i: i16) {
        T::write_i16(self, i)
    }
    #[inline]
    fn write_i32(&mut self, i: i32) {
        T::write_i32(self, i)
    }
    #[inline]
    fn write_i64(&mut self, i: i64) {
        T::write_i64(self, i)
    }
    #[inline]
    fn write_i128(&mut self, i: i128) {
        T::write_i128(self, i)
    }
    #[inline]
    fn write_isize(&mut self, i: isize) {
        T::write_isize(self, i)
    }
}

impl<T: Clone> Clone for MiniBox<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new_uninit().write(T::clone(self))
    }

    #[inline]
    fn clone_from(&mut self, other: &Self) {
        T::clone_from(self, other)
    }
}

impl<T> From<T> for MiniBox<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Clone> From<&T> for MiniBox<T> {
    #[inline]
    fn from(value: &T) -> Self {
        Self::with(move || value.clone())
    }
}

impl<T> From<Box<T>> for MiniBox<T> {
    fn from(value: Box<T>) -> Self {
        match Self::SIZE_CLASS {
            SizeClass::Zero => Self::new_zst(*value),
            SizeClass::Inline => Self::new(*value),
            SizeClass::Boxed => Self {
                ptr: core::mem::MaybeUninit::new(Box::into_raw(value)),
                drop: core::marker::PhantomData,
            },
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for MiniBox<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as fmt::Debug>::fmt(self, f)
    }
}

impl<T: fmt::Display> fmt::Display for MiniBox<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as fmt::Display>::fmt(self, f)
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for MiniBox<I> {}
impl<I: core::iter::FusedIterator> core::iter::FusedIterator for MiniBox<I> {}
impl<I: Iterator> Iterator for MiniBox<I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        I::next(self)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        I::size_hint(self)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<I::Item> {
        I::nth(self, n)
    }

    #[inline]
    fn last(self) -> Option<I::Item> {
        I::last(Self::into_inner(self))
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for MiniBox<I> {
    #[inline]
    fn next_back(&mut self) -> Option<I::Item> {
        I::next_back(self)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<I::Item> {
        I::nth_back(self, n)
    }
}

impl<T: Future> Future for MiniBox<T> {
    type Output = T::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        T::poll(Self::deref_pin_mut(self), cx)
    }
}

#[cfg(feature = "std")]
impl<T: io::Read> io::Read for MiniBox<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        T::read(self, buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        T::read_vectored(self, bufs)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        T::read_to_end(self, buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        T::read_to_string(self, buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        T::read_exact(self, buf)
    }
}

#[cfg(feature = "std")]
impl<T: io::Write> io::Write for MiniBox<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        T::write(self, buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        T::write_vectored(self, bufs)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        T::flush(self)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        T::write_all(self, buf)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        T::write_fmt(self, fmt)
    }
}

#[cfg(feature = "std")]
impl<T: io::Seek> io::Seek for MiniBox<T> {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        T::seek(self, pos)
    }

    #[cfg(feature = "nightly")]
    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        T::stream_len(self)
    }

    #[cfg(feature = "nightly")]
    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        T::stream_position(self)
    }
}

#[cfg(feature = "std")]
impl<T: io::BufRead> io::BufRead for MiniBox<T> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        T::fill_buf(self)
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        T::consume(self, amt)
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        T::read_until(self, byte, buf)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        T::read_line(self, buf)
    }
}

#[cfg(feature = "nightly")]
impl<T: FnOnce<A>, A> FnOnce<A> for MiniBox<T> {
    type Output = T::Output;

    #[inline]
    extern "rust-call" fn call_once(self, args: A) -> Self::Output {
        Self::into_inner(self).call_once(args)
    }
}

#[cfg(feature = "nightly")]
impl<T: FnMut<A>, A> FnMut<A> for MiniBox<T> {
    #[inline]
    extern "rust-call" fn call_mut(&mut self, args: A) -> Self::Output {
        T::call_mut(self, args)
    }
}

#[cfg(feature = "nightly")]
impl<T: Fn<A>, A> Fn<A> for MiniBox<T> {
    #[inline]
    extern "rust-call" fn call(&self, args: A) -> Self::Output {
        T::call(self, args)
    }
}
