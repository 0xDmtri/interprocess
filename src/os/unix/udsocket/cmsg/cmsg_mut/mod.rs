mod add_raw;
mod error;
mod ext;
pub(crate) mod read;

pub use {error::*, ext::*};

use super::{context::Collector, *};
use std::mem::MaybeUninit;

/// A type which can be used as a buffer for ancillary data.
///
/// # Safety
/// The following invariants must be upheld by implementations:
/// - The slices returned by `as_bytes()` and `as_bytes_mut()`:
///     - Must be pointer-wise equivalent (point to the same base address and have the same length)
///     - Must not change base address and legnth without a call to `reserve()`.
///         - No method from the `CmsgMut` trait in a conformant implementation of it may call `reserve()` indirectly (
///           other than `reserve()` itself).
/// - The reference returned by `context_mut()` must be pointer-wise equivalent to that of `context()`.
/// - If `valid_len()` returns some value 𝑛:
///     - It must not return any different 𝑛 until `set_len()` is called with some value 𝑚 as its argument, after which
///       `valid_len()` must return 𝑚.
///     - The first 𝑛 bytes of the slice returned by `as_bytes()`/`as_bytes_mut()` must be valid in the ancillary buffer
///       validity sense as described in the [module-level docs](super).
/// - As long as the safety contract of `set_len()` is upheld, `as_uninit().len()` must never go below `valid_len()`.
/// - `set_len()` must not lead to undefined behavior if its safety contract is upheld.
/// - If `Context` is not a zero-sized type, the slice returned by `as_bytes()`/`as_bytes_mut()` must not overlap with
///   the return value of `context()`/`context_mut()`.
/// - The following methods may not unwind (if divergence is needed, it must abort the process or kill the thread):
///     - `as_bytes()`
///     - `as_bytes_mut()`
///     - `valid_len()`
pub unsafe trait CmsgMut {
    /// The context collector type contained by the buffer.
    type Context: Collector + ?Sized;

    /// Returns the entire buffer, including both its initialized and uninitialized parts, as a single immutable slice.
    fn as_bytes(&self) -> &[MaybeUninit<u8>];
    /// Returns the entire buffer, including both its initialized and uninitialized parts, as a single mutable slice.
    ///
    /// # Safety
    /// The valid part of the slice (as designated by the return value of `valid_len()`) may not be modified in ways
    /// that compromise its validity. Ideally, `split_at_init()` should be used instead.
    unsafe fn as_bytes_mut(&mut self) -> &mut [MaybeUninit<u8>];
    /// Returns the amount of bytes at the beginning of the buffer considered to be valid in the ancillary buffer
    /// validity sense as described in the [module-level docs](super).
    fn valid_len(&self) -> usize;
    /// Sets the amount of bytes at the beginning of the buffer considered to be valid to the specified value.
    ///
    /// # Safety
    /// No checks are to be expected of an implementation. The following invariants must be upheld:
    /// - `new_len` must not exceed the capacity (given by `as_uninit().len()`).
    /// - The given amount of bytes at the beginning of the buffer must indeed be valid in the ancillary buffer validity
    /// sense as described in the [module-level docs](super).
    unsafe fn set_len(&mut self, new_len: usize);
    /// Immutably borrows the context collector contained in the buffer.
    ///
    /// Immutable access to the context collector is useful for deserializing control messages via the [`ancillary`]
    /// module's facilities. See [`CmsgMutExt::as_ref`].
    fn context(&self) -> &Self::Context;
    /// Mutably borrows the context collector contained in the buffer.
    ///
    /// This is used by implementations of [`ReadAncillary`](super::super::ReadAncillary) to collect context to aid
    /// later deserialization.
    fn context_mut(&mut self) -> &mut Self::Context;

    /// Attempts to increase the underlying buffer's capacity by the given amount of bytes. Returns `Ok` if capacity was
    /// increased by `additional` or more, or `Err` if it was left unchanged.
    ///
    /// This is the only method which is allowed to change the base pointer returned by the next call to
    /// `as_bytes()`/`as_bytes_mut()`.
    fn reserve(&mut self, additional: usize) -> ReserveResult {
        let _ = additional;
        Err(ReserveError::Unsupported)
    }
    /// Like `reserve()`, but hints the underlying buffer data structure not to purposely overallocate. The memory
    /// allocator may still choose to overallocate.
    fn reserve_exact(&mut self, additional: usize) -> ReserveResult {
        let _ = additional;
        Err(ReserveError::Unsupported)
    }
}

/// The trait object type for [`CmsgMut`].
pub type DynCmsgMut<'m, 'c> = dyn CmsgMut<Context = dyn Collector + 'c> + 'm;
/// A trait object type for [`CmsgMut`] with no non-`'static` borrows.
///
/// Note that [`CmsgMutBuf`] will not work with this type, since it holds a `&mut [MaybeUninit<u8>]`.
pub type DynCmsgMutStatic = DynCmsgMut<'static, 'static>;

#[cfg(debug_assertions)]
fn _assert_object_safe<
    'm,
    'c,
    'x,
    'y,
    T: CmsgMut<Context = dyn Collector + 'c> + 'm,
    U: CmsgMut<Context = dyn Collector> + 'static,
>(
    x: &'x mut T,
    y: &'y mut U,
) -> (&'x mut DynCmsgMut<'m, 'c>, &'y mut DynCmsgMutStatic) {
    (x, y)
}
