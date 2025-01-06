// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate provides raw bindings and a safe wrapper for [TableGen](https://llvm.org/docs/TableGen/),
//! a domain-specific language used by the [LLVM project](https://llvm.org/).
//!
//! The goal of this crate is to enable users to develop custom [TableGen backends](https://llvm.org/docs/TableGen/BackGuide.html)
//! in Rust. Hence the primary use case of this crate are procedural macros that
//! generate Rust code from TableGen description files.
//!
//! # Safety
//!
//! This crate aims to be completely safe.
//!
//! # Supported LLVM Versions
//!
//! An installation of LLVM is required to use this crate.
//! The versions of LLVM currently supported are 16.x.x (default) and 17.x.x.
//! Different LLVM version can be selected using features flags (llvm16-0 or
//! llvm17-0).
//!
//! The `TABLEGEN_<version>_PREFIX` environment variable can be used to specify
//! a custom directory of the LLVM installation.
//!
//! # Examples
//!
//! The following example parse simple TableGen code provided as a `&str` and
//! iterates over classes and defs defined in this file.
//!
//! ```rust
//! use tblgen_alt::{TableGenParser, RecordKeeper};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let keeper: RecordKeeper = TableGenParser::new()
//!     .add_source(
//!         r#"
//!         class A;
//!         def D: A;
//!         "#,
//!     )?
//!     .parse()?;
//! assert_eq!(keeper.classes().next().unwrap().0, Ok("A"));
//! assert_eq!(keeper.defs().next().unwrap().0, Ok("D"));
//! assert_eq!(keeper.all_derived_definitions("A").next().unwrap().name(), Ok("D"));
//! # Ok(())
//! # }
//! ```
//!
//! By adding include paths, external TableGen files can be included.
//!
//! ```rust
//! use tblgen_alt::{TableGenParser, RecordKeeper};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let keeper: RecordKeeper = TableGenParser::new()
//!     .add_source(r#"include "mlir/IR/OpBase.td""#)?
//!     .add_include_path(&format!("{}/include", std::env::var("TABLEGEN_190_PREFIX")?))
//!     .parse()?;
//! let i32_def = keeper.def("I32").expect("has I32 def");
//! assert!(i32_def.subclass_of("I"));
//! assert_eq!(i32_def.int_value("bitwidth"), Ok(32));
//! # Ok(())
//! # }
//! ```
//!
//! You can also pass an included file's path directly.
//!
//! ```rust
//! use tblgen_alt::{TableGenParser, RecordKeeper};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let keeper: RecordKeeper = TableGenParser::new()
//!     .add_source_file("mlir/IR/OpBase.td")
//!     .add_include_path(&format!("{}/include", std::env::var("TABLEGEN_190_PREFIX")?))
//!     .parse()?;
//! let i32_def = keeper.def("I32").expect("has I32 def");
//! assert!(i32_def.subclass_of("I"));
//! assert_eq!(i32_def.int_value("bitwidth"), Ok(32));
//! # Ok(())
//! # }
//! ```
//!
//! # API Stability
//!
//! LLVM does not provide a stable C API for TableGen, and the C API provided by
//! this crate is not stable. Furthermore, the safe wrapper does not provide a
//! stable interface either, since this crate is still in early development.

pub mod error;
pub mod init;
/// TableGen records and record values.
pub mod record;
/// TableGen record keeper.
pub mod record_keeper;
mod string_ref;
mod util;

/// This module contains raw bindings for TableGen. Note that these bindings are
/// unstable and can change at any time.
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod raw {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::ffi::CStr;
use std::ffi::CString;
use std::marker::PhantomData;
use std::sync::Mutex;

pub use error::Error;
use error::TableGenError;
pub use init::TypedInit;
pub use record::Record;
pub use record::RecordValue;
pub use record_keeper::RecordKeeper;

use raw::{
    tableGenAddIncludePath, tableGenAddSource, tableGenAddSourceFile, tableGenFree, tableGenGet,
    tableGenParse, TableGenParserRef,
};
use string_ref::StringRef;

// TableGen only exposes `TableGenParseFile` in its API.
// However, this function uses global state and therefore it is not thread safe.
// Until they remove this hack, we have to deal with it ourselves.
static TABLEGEN_PARSE_LOCK: Mutex<()> = Mutex::new(());

/// Builder struct that parses TableGen source files and builds a
/// [`RecordKeeper`].
#[derive(Debug, PartialEq, Eq)]
pub struct TableGenParser<'s> {
    raw: TableGenParserRef,
    source_strings: Vec<CString>,
    _source_ref: PhantomData<&'s str>,
}

impl<'s> Default for TableGenParser<'s> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> TableGenParser<'s> {
    /// Initalizes a new TableGen parser.
    pub fn new() -> Self {
        Self {
            raw: unsafe { tableGenGet() },
            source_strings: Vec::new(),
            _source_ref: PhantomData,
        }
    }

    /// Adds the given path to the list of included directories.
    pub fn add_include_path(self, include: &str) -> Self {
        unsafe { tableGenAddIncludePath(self.raw, StringRef::from(include).to_raw()) }
        self
    }

    /// Reads TableGen source code from the file at the given path.
    pub fn add_source_file(self, source: &str) -> Self {
        unsafe { tableGenAddSourceFile(self.raw, StringRef::from(source).to_raw()) }
        self
    }

    /// Adds the given TableGen source string.
    ///
    /// The string must be null-terminated and is not copied, hence it is
    /// required to live until the source code is parsed.
    pub fn add_source_raw(self, source: &'s CStr) -> Result<Self, Error> {
        if unsafe { tableGenAddSource(self.raw, source.as_ptr()) > 0 } {
            Ok(self)
        } else {
            Err(TableGenError::InvalidSource.into())
        }
    }

    /// Adds the given TableGen source string.
    ///
    /// The string is copied into a null-terminated [`CString`].
    pub fn add_source(mut self, source: &str) -> Result<Self, Error> {
        let string = CString::new(source).map_err(TableGenError::from)?;
        self.source_strings.push(string);
        if unsafe {
            tableGenAddSource(
                self.raw,
                self.source_strings.last().expect("not empty").as_ptr(),
            ) > 0
        } {
            Ok(self)
        } else {
            Err(TableGenError::InvalidSource.into())
        }
    }

    pub fn source_info(&self) -> SourceInfo {
        SourceInfo(self)
    }

    /// Parses the TableGen source files and returns a [`RecordKeeper`].
    ///
    /// Due to limitations of TableGen, parsing TableGen is not thread-safe.
    /// In order to provide thread-safety, this method ensures that any
    /// concurrent parse operations are executed sequentially.
    pub fn parse(self) -> Result<RecordKeeper<'s>, Error> {
        unsafe {
            let guard = TABLEGEN_PARSE_LOCK.lock().unwrap();
            let keeper = tableGenParse(self.raw);
            let res = if !keeper.is_null() {
                Ok(RecordKeeper::from_raw(keeper, self))
            } else {
                Err(TableGenError::Parse.into())
            };
            drop(guard);
            res
        }
    }
}

impl<'s> Drop for TableGenParser<'s> {
    fn drop(&mut self) {
        unsafe {
            tableGenFree(self.raw);
        }
    }
}

/// Reference to TableGen source file.
///
/// See [`TableGenParser::source_info`](TableGenParser::source_info) and
/// [`RecordKeeper::source_info`](RecordKeeper::source_info).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceInfo<'a>(pub(crate) &'a TableGenParser<'a>);
