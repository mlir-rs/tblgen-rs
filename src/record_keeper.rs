// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::marker::PhantomData;

#[cfg(any(feature = "llvm18-0", feature = "llvm19-0", feature = "llvm20-0"))]
use crate::error::TableGenError;
#[cfg(any(feature = "llvm16-0", feature = "llvm17-0"))]
use crate::error::{SourceLocation, TableGenError, WithLocation};
use crate::{
    Error, SourceInfo, TableGenParser,
    raw::{
        TableGenRecordKeeperIteratorRef, TableGenRecordKeeperRef, TableGenRecordVectorRef,
        tableGenRecordKeeperFree, tableGenRecordKeeperGetAllDerivedDefinitions,
        tableGenRecordKeeperGetClass, tableGenRecordKeeperGetDef,
        tableGenRecordKeeperGetFirstClass, tableGenRecordKeeperGetFirstDef,
        tableGenRecordKeeperGetNextClass, tableGenRecordKeeperGetNextDef,
        tableGenRecordKeeperItemGetName, tableGenRecordKeeperItemGetRecord,
        tableGenRecordKeeperIteratorClone, tableGenRecordKeeperIteratorFree,
        tableGenRecordVectorFree, tableGenRecordVectorGet,
    },
    record::Record,
    string_ref::StringRef,
};

/// Struct that holds all records from a TableGen file.
#[derive(Debug, PartialEq, Eq)]
pub struct RecordKeeper<'s> {
    raw: TableGenRecordKeeperRef,
    pub(crate) parser: TableGenParser<'s>,
}

impl<'s> RecordKeeper<'s> {
    pub(crate) unsafe fn from_raw(
        raw: TableGenRecordKeeperRef,
        parser: TableGenParser<'s>,
    ) -> RecordKeeper<'s> {
        RecordKeeper { raw, parser }
    }

    /// Returns an iterator over all classes.
    ///
    /// The iterator yields tuples of type `(String, Record)`.
    pub fn classes(&self) -> NamedRecordIter<'_, IsClass> {
        unsafe { NamedRecordIter::from_raw(tableGenRecordKeeperGetFirstClass(self.raw)) }
    }

    /// Returns an iterator over all definitions.
    ///
    /// The iterator yields tuples of type `(String, Record)`.
    pub fn defs(&self) -> NamedRecordIter<'_, IsDef> {
        unsafe { NamedRecordIter::from_raw(tableGenRecordKeeperGetFirstDef(self.raw)) }
    }

    /// Returns the class with the given name.
    pub fn class(&self, name: &str) -> Result<Record, Error> {
        unsafe {
            let class = tableGenRecordKeeperGetClass(self.raw, StringRef::from(name).to_raw());
            if class.is_null() {
                Err(TableGenError::MissingClass(name.into()).into())
            } else {
                Ok(Record::from_raw(class))
            }
        }
    }

    /// Returns the definition with the given name.
    pub fn def(&self, name: &str) -> Result<Record, Error> {
        unsafe {
            let def = tableGenRecordKeeperGetDef(self.raw, StringRef::from(name).to_raw());
            if def.is_null() {
                Err(TableGenError::MissingDef(name.into()).into())
            } else {
                Ok(Record::from_raw(def))
            }
        }
    }

    /// Returns an iterator over all definitions that derive from the class with
    /// the given name.
    pub fn all_derived_definitions(&self, name: &str) -> RecordIter {
        unsafe {
            RecordIter::from_raw_vector(tableGenRecordKeeperGetAllDerivedDefinitions(
                self.raw,
                StringRef::from(name).to_raw(),
            ))
        }
    }

    pub fn source_info(&self) -> SourceInfo {
        SourceInfo(&self.parser)
    }
}

impl Drop for RecordKeeper<'_> {
    fn drop(&mut self) {
        unsafe {
            tableGenRecordKeeperFree(self.raw);
        }
    }
}

#[doc(hidden)]
pub struct IsClass;
#[doc(hidden)]
pub struct IsDef;

trait NextRecord {
    unsafe fn next(raw: &mut TableGenRecordKeeperIteratorRef);
}

impl NextRecord for IsClass {
    unsafe fn next(raw: &mut TableGenRecordKeeperIteratorRef) {
        unsafe {
            tableGenRecordKeeperGetNextClass(raw);
        }
    }
}

impl NextRecord for IsDef {
    unsafe fn next(raw: &mut TableGenRecordKeeperIteratorRef) {
        unsafe {
            tableGenRecordKeeperGetNextDef(raw);
        }
    }
}

#[derive(Debug)]
pub struct NamedRecordIter<'a, T> {
    raw: TableGenRecordKeeperIteratorRef,
    _kind: PhantomData<&'a T>,
}

impl<T> NamedRecordIter<'_, T> {
    unsafe fn from_raw(raw: TableGenRecordKeeperIteratorRef) -> Self {
        NamedRecordIter {
            raw,
            _kind: PhantomData,
        }
    }
}

impl<'a, T: NextRecord> Iterator for NamedRecordIter<'a, T> {
    type Item = (Result<&'a str, std::str::Utf8Error>, Record<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let current = if self.raw.is_null() {
            return None;
        } else {
            unsafe {
                Some((
                    StringRef::from_raw(tableGenRecordKeeperItemGetName(self.raw)).try_into(),
                    Record::from_raw(tableGenRecordKeeperItemGetRecord(self.raw)),
                ))
            }
        };
        unsafe { T::next(&mut self.raw) };
        current
    }
}

impl<T> Clone for NamedRecordIter<'_, T> {
    fn clone(&self) -> Self {
        unsafe { Self::from_raw(tableGenRecordKeeperIteratorClone(self.raw)) }
    }
}

impl<T> Drop for NamedRecordIter<'_, T> {
    fn drop(&mut self) {
        unsafe { tableGenRecordKeeperIteratorFree(self.raw) }
    }
}

pub struct RecordIter<'a> {
    raw: TableGenRecordVectorRef,
    index: usize,
    _reference: PhantomData<&'a ()>,
}

impl<'a> RecordIter<'a> {
    unsafe fn from_raw_vector(ptr: TableGenRecordVectorRef) -> RecordIter<'a> {
        RecordIter {
            raw: ptr,
            index: 0,
            _reference: PhantomData,
        }
    }
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = Record<'a>;

    fn next(&mut self) -> Option<Record<'a>> {
        let next = unsafe { tableGenRecordVectorGet(self.raw, self.index) };
        self.index += 1;
        if next.is_null() {
            None
        } else {
            unsafe { Some(Record::from_raw(next)) }
        }
    }
}

impl Drop for RecordIter<'_> {
    fn drop(&mut self) {
        unsafe { tableGenRecordVectorFree(self.raw) }
    }
}

#[cfg(test)]
mod test {
    use crate::TableGenParser;

    #[test]
    fn classes_and_defs() {
        let rk = TableGenParser::new()
            .add_source(
                r#"
                class A;
                class B;
                class C;
                def D1: A;
                def D2: B;
                def D3: C;
                "#,
            )
            .unwrap()
            .parse()
            .expect("valid tablegen");
        rk.classes()
            .for_each(|i| assert!(i.1.name().unwrap() == i.0.unwrap()));
        rk.defs()
            .for_each(|i| assert!(i.1.name().unwrap() == i.0.unwrap()));
        assert!(rk.classes().map(|i| i.0.unwrap()).eq(["A", "B", "C"]));
        assert!(rk.defs().map(|i| i.0.unwrap()).eq(["D1", "D2", "D3"]));
    }

    #[test]
    fn derived_defs() {
        let rk = TableGenParser::new()
            .add_source(
                r#"
                class A;
                class B;
                class C;

                def D1: A;
                def D2: A, B;
                def D3: B, C;
                "#,
            )
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let a = rk.all_derived_definitions("A");
        assert!(a.map(|i| i.name().unwrap().to_string()).eq(["D1", "D2"]));
        let b = rk.all_derived_definitions("B");
        assert!(b.map(|i| i.name().unwrap().to_string()).eq(["D2", "D3"]));
    }

    #[test]
    fn single() {
        let rk = TableGenParser::new()
            .add_source(
                r#"
                class A;
                def D1;
                "#,
            )
            .unwrap()
            .parse()
            .expect("valid tablegen");
        assert_eq!(rk.class("A").expect("class exists").name().unwrap(), "A");
        assert_eq!(rk.def("D1").expect("def exists").name().unwrap(), "D1");
    }
}
