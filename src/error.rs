// Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains error types used by this crate and provides additional
//! error handling utilities for dependent crates.
//!
//! Disclaimer: this module may change significantly in the future.
//!
//! All different error types are defined in the [`TableGenError`] enum.
//! However, most functions return a [`SourceError<TableGenError>`] (has alias
//! [`Error`]). This error type includes a [`SourceLocation`], a reference to a
//! line in a TableGen source file.
//!
//! To provide information about the source code at this location (e.g. code at
//! location, file name, line and column), [`SourceInfo`] must be provided to
//! the error. In this case, the error message will be formatted by LLVM's
//! `SourceMgr` class.
//!
//! ```rust
//! use tblgen::{TableGenParser, RecordKeeper};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let keeper: RecordKeeper = TableGenParser::new()
//!     .add_source(
//!         r#"
//!         def A {
//!             int i = 5;
//!         }
//!         "#,
//!     )?
//!     .parse()?;
//! if let Err(e) = keeper.def("A").unwrap().string_value("i") {
//!     println!("{}", e);
//!     // invalid conversion from Int to alloc::string::String
//!
//!     println!("{}", e.add_source_info(keeper.source_info()));
//!     // error: invalid conversion from Int to alloc::string::String
//!     //   int a = test;
//!     //       ^
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Note that `add_source_info` should be called with the correct source info.
//! This is not statically enforced, but runtime checks are implemented to check
//! that the given [`SourceInfo`] matches the [`SourceLocation`] in the error.
//! If it does not match, the error will be printed without information about
//! the TableGen source file.
//!
//! Custom error types that implement [`std::error::Error`] also implement
//! [`WithLocation`]. That way, a [`SourceLocation`] can be attached to any
//! error by calling [`with_location`](`WithLocation::with_location`).

use std::{
    convert::Infallible,
    ffi::{c_void, NulError},
    fmt::{self, Display, Formatter},
    str::Utf8Error,
    string::FromUtf8Error,
};

use crate::{
    raw::{
        tableGenPrintError, tableGenSourceLocationClone, tableGenSourceLocationFree,
        tableGenSourceLocationNull, TableGenDiagKind::TABLEGEN_DK_ERROR, TableGenSourceLocationRef,
    },
    string_ref::StringRef,
    util::print_string_callback,
    SourceInfo, TableGenParser,
};

/// Enum of TableGen errors.
#[non_exhaustive]
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum TableGenError {
    #[error("invalid TableGen source")]
    InvalidSource,
    #[error("invalid TableGen source")]
    InvalidSourceString(#[from] NulError),
    #[error("invalid UTF-8 string")]
    InvalidUtf8Str(#[from] Utf8Error),
    #[error("invalid UTF-8 string")]
    InvalidUtf8String(#[from] FromUtf8Error),
    #[error("failed to parse TableGen source")]
    Parse,
    #[error("expected field {0} in record")]
    MissingValue(String),
    #[error("expected def {0}")]
    MissingDef(String),
    #[error("expected class {0}")]
    MissingClass(String),
    #[error("invalid conversion from {from} to {to}")]
    InitConversion {
        from: &'static str,
        to: &'static str,
    },
    #[error("invalid source location")]
    InvalidSourceLocation,
    #[error("infallible")]
    Infallible(#[from] Infallible),
}

/// A location in a TableGen source file.
#[derive(Debug, PartialEq, Eq)]
pub struct SourceLocation {
    raw: TableGenSourceLocationRef,
}

// SourceLocation is a read-only llvm::ArrayRef, which should be thread-safe.
unsafe impl Sync for SourceLocation {}
unsafe impl Send for SourceLocation {}

impl SourceLocation {
    /// # Safety
    /// The passed pointer should be a valid table gen source location.
    pub unsafe fn from_raw(raw: TableGenSourceLocationRef) -> Self {
        Self { raw }
    }

    /// Returns a [`SourceLocation`] for an undetermined location in the
    /// TableGen source file.
    pub fn none() -> Self {
        unsafe {
            Self {
                raw: tableGenSourceLocationNull(),
            }
        }
    }
}

impl Clone for SourceLocation {
    fn clone(&self) -> Self {
        unsafe { Self::from_raw(tableGenSourceLocationClone(self.raw)) }
    }
}

impl Drop for SourceLocation {
    fn drop(&mut self) {
        unsafe { tableGenSourceLocationFree(self.raw) }
    }
}

/// A wrapper around error types which includes a [`SourceLocation`].
///
/// This error is used to describe erros in the TableGen source file at a
/// certain location.
///
/// By calling `add_source_info`, information about the TableGen source file at
/// the [`SourceLocation`] will be included in this error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceError<E> {
    location: SourceLocation,
    message: Option<String>,
    error: E,
}

impl<E: std::error::Error> SourceError<E> {
    /// Creates a new [`SourceError`].
    pub fn new(location: SourceLocation, error: E) -> Self {
        Self {
            location,
            error,
            message: None,
        }
    }

    pub fn location(&self) -> &SourceLocation {
        &self.location
    }

    pub fn error(&self) -> &E {
        &self.error
    }

    /// Replaces the inner error with the given error.
    ///
    /// Any source information that was previously attached with
    /// [`SourceError::add_source_info`] will be removed.
    pub fn set_error<F: std::error::Error>(self, error: F) -> SourceError<F> {
        SourceError {
            error,
            message: None,
            location: self.location,
        }
    }

    /// Replaces the location.
    ///
    /// Any source information that was previously attached with
    /// [`SourceError::add_source_info`] will be removed.
    pub fn set_location(mut self, location: impl SourceLoc) -> Self {
        self.location = location.source_location();
        self
    }

    /// Adds information about the TableGen source file at the
    /// given [`SourceLocation`] to this error.
    ///
    /// A new error message will be created by `SourceMgr` class of LLVM.
    pub fn add_source_info(mut self, info: SourceInfo) -> Self {
        self.message = Some(Self::create_message(
            info.0,
            &self.location,
            &format!("{}", self.error),
        ));
        self
    }

    fn create_message(parser: &TableGenParser, location: &SourceLocation, message: &str) -> String {
        let mut data: (_, Result<_, TableGenError>) = (String::new(), Ok(()));
        let res = unsafe {
            tableGenPrintError(
                parser.raw,
                location.raw,
                TABLEGEN_DK_ERROR,
                StringRef::from(message).to_raw(),
                Some(print_string_callback),
                &mut data as *mut _ as *mut c_void,
            )
        };
        if res == 0 {
            data.1 = Err(TableGenError::InvalidSourceLocation);
        }
        if let Err(e) = data.1 {
            data.0 = format!("{}\nfailed to print source information: {}", message, e);
        }
        data.0
    }
}

impl<E: std::error::Error> Display for SourceError<E> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(message) = self.message.as_ref() {
            write!(f, "{}", message)
        } else {
            write!(f, "{}", self.error)
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for SourceError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl From<TableGenError> for SourceError<TableGenError> {
    fn from(value: TableGenError) -> Self {
        value.with_location(SourceLocation::none())
    }
}

pub trait WithLocation: std::error::Error + Sized {
    /// Creates a [`SourceError`] wrapper.
    fn with_location<L: SourceLoc>(self, location: L) -> SourceError<Self> {
        SourceError::new(location.source_location(), self)
    }
}

impl<E> WithLocation for E where E: std::error::Error {}

pub trait SourceLoc {
    /// Returns the source location.
    fn source_location(self) -> SourceLocation;
}

impl SourceLoc for SourceLocation {
    fn source_location(self) -> SourceLocation {
        self
    }
}

/// Main error type.
pub type Error = SourceError<TableGenError>;
