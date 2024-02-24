#![allow(unused_macros)]
// TODO more internal docs

macro_rules! impmod { // TODO remove
	($($osmod:ident)::+, $($orig:ident $(as $into:ident)?),* $(,)?) => {
		#[cfg(unix)]
		use $crate::os::unix::$($osmod)::+::{$($orig $(as $into)?,)*};
		#[cfg(windows)]
		use $crate::os::windows::$($osmod)::+::{$($orig $(as $into)?,)*};
	};
}

/// Branches on the given boolean expression, returning `Err` with `errno`/`GetLastError()` if it's
/// false.
macro_rules! ok_or_errno {
	($success:expr => $($scb:tt)+) => {
		if $success {
			Ok($($scb)+)
		} else {
			Err(::std::io::Error::last_os_error())
		}
	};
}

/// Generates a method that projects `self.0` of type `src` to a `Pin` for type `dst`.
macro_rules! pinproj_for_unpin {
	($src:ty, $dst:ty) => {
		impl $src {
			#[inline(always)]
			fn pinproj(&mut self) -> ::std::pin::Pin<&mut $dst> {
				::std::pin::Pin::new(&mut self.0)
			}
		}
	};
}

/// Calls multiple macros, passing the same identifer or type as well as optional per-macro
/// parameters.
///
/// The identifier or type goes first, then comma-separated macro names without exclamation points.
/// To pass per-macro parameters, encase them in parentheses.
macro_rules! multimacro {
	($pre:tt $ty:ident, $($macro:ident $(($($arg:tt)+))?),+ $(,)?) => {$(
		$macro!($pre $ty $(, $($arg)+)?);
	)+};
	($pre:tt $ty:ty, $($macro:ident $(($($arg:tt)+))?),+ $(,)?) => {$(
		$macro!($pre $ty $(, $($arg)+)?);
	)+};
	($ty:ident, $($macro:ident $(($($arg:tt)+))?),+ $(,)?) => {$(
		$macro!($ty $(, $($arg)+)?);
	)+};
	($ty:ty, $($macro:ident $(($($arg:tt)+))?),+ $(,)?) => {$(
		$macro!($ty $(, $($arg)+)?);
	)+};
}

/// Generates a method that immutably borrows `self.0` of type `int` for type `ty`.
///
/// If `kind` is `&`, `self.0` is borrowed directly. If `kind` is `*`, `self.0` is treated as a
/// smart pointer (`Deref` is applied).
///
/// The method generated by this macro is used by forwarding macros.
macro_rules! forward_rbv {
	(@$slf:ident, &) => { &$slf.0 };
	(@$slf:ident, *) => { &&*$slf.0 };
	($ty:ty, $int:ty, $kind:tt) => {
		impl $ty {
			#[inline(always)]
			fn refwd(&self) -> &$int {
				forward_rbv!(@self, $kind)
			}
		}
	};
}

/// Generates this module's macro submodules.
macro_rules! make_macro_modules {
	($($modname:ident),+ $(,)?) => {$(
		#[macro_use] mod $modname;
		#[allow(unused_imports)]
		pub(crate) use $modname::*;
	)+};
}

make_macro_modules! {
	derive_raw, derive_mut_iorw, derive_trivconv,
	forward_handle_and_fd, forward_try_clone, forward_to_self, forward_iorw, forward_fmt,
}
