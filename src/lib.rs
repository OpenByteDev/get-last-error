#![cfg(windows)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    nightly,
    feature(maybe_uninit_uninit_array, maybe_uninit_array_assume_init)
)]
#![warn(
    unsafe_op_in_unsafe_fn,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    clippy::todo,
    clippy::manual_assert,
    clippy::must_use_candidate,
    clippy::inconsistent_struct_constructor,
    clippy::wrong_self_convention,
    clippy::missing_const_for_fn,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]

//! # get-last-error
//!
//! An error wrapper over Win32 API errors.
//!
//! ## Examples
//!
//! A [`Win32Error`] can be constructed from an arbitrary [`DWORD`]:
//!
//! ```
//! use get_last_error::Win32Error;
//!
//! let err = Win32Error::new(0);
//! println!("{}", err); // prints "The operation completed successfully."
//! ```
//!
//! The [`Win32Error::get_last_error`] retrieves the last error code for the current thread:
//!
//! ```
//! use get_last_error::Win32Error;
//! use winapi::um::{winnt::HANDLE, processthreadsapi::OpenProcess};
//!
//! fn open_process() -> Result<HANDLE, Win32Error> {
//!     let result = unsafe { OpenProcess(0, 0, 0) }; // some windows api call
//!     if result.is_null() { // null indicates failure.
//!         Err(Win32Error::get_last_error())
//!     } else {
//!         Ok(result)
//!     }
//! }
//! ```

use core::{
    fmt::{self, Display, Write},
    mem::MaybeUninit,
    ptr,
};

#[cfg(feature = "std")]
use std::{error::Error, io};

use winapi::{
    shared::minwindef::DWORD,
    um::{
        errhandlingapi::GetLastError,
        winbase::{
            FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
            FORMAT_MESSAGE_MAX_WIDTH_MASK,
        },
    },
};

/// A wrapper over Win32 API errors.
/// Implements [`Display`] using [`FormatMessageW`](https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessagew).
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Win32Error(DWORD);

impl Win32Error {
    /// Constructs a new error from an arbitrary [`DWORD`].
    #[must_use]
    pub const fn new(code: DWORD) -> Self {
        Self(code)
    }

    /// Returns the last error code for the current thread.
    #[must_use]
    pub fn get_last_error() -> Self {
        Self::new(unsafe { GetLastError() })
    }

    /// Returns the underlying error code.
    #[must_use]
    pub const fn code(&self) -> DWORD {
        self.0
    }
}

impl From<DWORD> for Win32Error {
    fn from(other: DWORD) -> Self {
        Self::new(other)
    }
}

impl From<Win32Error> for DWORD {
    fn from(other: Win32Error) -> Self {
        other.code()
    }
}

impl Display for Win32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = maybe_uninit_uninit_array::<u16, 1024>();

        let len = unsafe {
            FormatMessageW(
                FORMAT_MESSAGE_FROM_SYSTEM
                    | FORMAT_MESSAGE_IGNORE_INSERTS
                    | FORMAT_MESSAGE_MAX_WIDTH_MASK,
                ptr::null(),
                self.0,
                0,
                buf[0].as_mut_ptr(),
                buf.len() as _,
                ptr::null_mut(),
            )
        } as usize;

        if len == 0 {
            // `FormatMessageW` failed -> use raw error code instead
            write!(f, "{:#08X}", self.0)
        } else {
            // `FormatMessageW` succeeded -> convert to UTF8 and process
            let wide_chars = unsafe { maybe_uninit_slice_assume_init_ref(&buf[..len]) };
            let mut char_buf = maybe_uninit_uninit_array::<char, 1024>();

            let char_iter = char::decode_utf16(wide_chars.iter().copied())
                .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER));

            let mut i = 0;
            for c in char_iter {
                char_buf[i].write(c);
                i += 1;
            }

            let chars = unsafe { maybe_uninit_slice_assume_init_ref(&char_buf[..i]) };
            let start = chars.iter().position(|c| !c.is_whitespace()).unwrap_or(0);
            let end = chars
                .iter()
                .rposition(|c| !c.is_whitespace())
                .unwrap_or(chars.len());
            for c in &chars[start..end] {
                f.write_char(*c)?;
            }

            Ok(())
        }
    }
}

#[cfg(feature = "std")]
impl Error for Win32Error {}

#[cfg(feature = "std")]
/// Error when converting a [`io::Error`] to a [`Win32Error`] when no raw os error code is available.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryFromIoError;

#[cfg(feature = "std")]
impl Display for TryFromIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the given io error did not contain a windows api error code."
        )
    }
}

#[cfg(feature = "std")]
impl Error for TryFromIoError {}

#[cfg(feature = "std")]
impl TryFrom<io::Error> for Win32Error {
    type Error = TryFromIoError;

    fn try_from(err: io::Error) -> Result<Self, Self::Error> {
        err.raw_os_error()
            .map_or_else(|| Err(TryFromIoError), |code| Ok(Self::new(code as _)))
    }
}

#[cfg(feature = "std")]
impl From<Win32Error> for io::Error {
    fn from(err: Win32Error) -> Self {
        Self::from_raw_os_error(err.code() as _)
    }
}

const unsafe fn maybe_uninit_slice_assume_init_ref<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    #[cfg(nightly)]
    unsafe {
        MaybeUninit::slice_assume_init_ref(slice)
    }
    #[cfg(not(nightly))]
    unsafe {
        &*(slice as *const [MaybeUninit<T>] as *const [T])
    }
}

fn maybe_uninit_uninit_array<T, const LEN: usize>() -> [MaybeUninit<T>; LEN] {
    #[cfg(nightly)]
    unsafe {
        MaybeUninit::uninit_array::<MAX_PATH, T>()
    }
    #[cfg(not(nightly))]
    unsafe {
        MaybeUninit::<[MaybeUninit<T>; LEN]>::uninit().assume_init()
    }
}
