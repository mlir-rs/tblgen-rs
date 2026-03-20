// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use paste::paste;
use std::{ffi::c_void, marker::PhantomData};

use crate::raw::{
    TableGenRecordRef, TableGenRecordValRef, tableGenRecordDump, tableGenRecordGetFieldType,
    tableGenRecordGetFirstValue, tableGenRecordGetLoc, tableGenRecordGetName,
    tableGenRecordGetNumSuperClasses, tableGenRecordGetNumTemplateArgs,
    tableGenRecordGetSuperClass, tableGenRecordGetTemplateArgName, tableGenRecordGetValue,
    tableGenRecordIsAnonymous, tableGenRecordIsSubclassOf, tableGenRecordPrint,
    tableGenRecordValDump, tableGenRecordValGetLoc, tableGenRecordValGetNameInit,
    tableGenRecordValGetValue, tableGenRecordValNext, tableGenRecordValPrint,
    tableGenRecordGetValueAsInt, tableGenRecordGetValueAsString, tableGenRecordGetValueAsBit,
    tableGenRecordGetValueAsDef, tableGenRecordGetValueAsDag, tableGenRecordGetValueAsBitsInit,
    tableGenRecordGetValueAsListInit, tableGenRecordGetValueAsListOfDefs,
    tableGenRecordGetValueAsListOfInts, tableGenIntArrayFree,
    tableGenRecordGetValueAsListOfStrings, tableGenStringRefArrayFree,
    tableGenRecordGetValueAsOptionalString, tableGenRecordGetValueAsOptionalDef,
    tableGenRecordIsValueUnset,
    tableGenRecordIsClass, tableGenRecordGetDefInit, tableGenRecordGetID,
    tableGenRecordGetNameInit, tableGenRecordHasDirectSuperClass,
    tableGenRecordRecTyGetNumClasses, tableGenRecordRecTyGetClass,
    tableGenRecordRecTyIsSubClassOf,
    tableGenRecordValIsTemplateArg, tableGenRecordValIsNonconcreteOK,
    tableGenRecordValGetBitsWidth, tableGenRecordValGetListElementType,
    TableGenRecTyKind::TableGenInvalidRecTyKind as RawInvalidRecTyKind,
};

use crate::{
    error::{Error, SourceLoc, SourceLocation, TableGenError, WithLocation},
    init::{BitInit, BitsInit, DagInit, DefInit, ListInit, StringInit, TypedInit},
    string_ref::StringRef,
    util::print_callback,
};
use std::fmt::{self, Debug, Display, Formatter};

/// An immutable reference to a TableGen record.
///
/// This reference cannot outlive the
/// [`RecordKeeper`](crate::record_keeper::RecordKeeper) from which it is
/// borrowed.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Record<'a> {
    raw: TableGenRecordRef,
    _reference: PhantomData<&'a TableGenRecordRef>,
}

impl Display for Record<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut data = (formatter, Ok(()));

        unsafe {
            tableGenRecordPrint(
                self.raw,
                Some(print_callback),
                &mut data as *mut _ as *mut c_void,
            );
        }

        data.1
    }
}

impl Debug for Record<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter, "Record(")?;
        Display::fmt(self, formatter)?;
        write!(formatter, ")")
    }
}

impl std::hash::Hash for Record<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

macro_rules! record_value {
    ($(#[$attr:meta])* $name:ident, $type:ty) => {
        paste! {
            $(#[$attr])*
            pub fn [<$name _value>](self, name: &str) -> Result<$type, Error> {
                self.value(name)?.try_into()
            }
        }
    };
}

impl<'a> Record<'a> {
    /// Creates a record from a raw object.
    ///
    /// # Safety
    ///
    /// The raw object must be valid.
    pub unsafe fn from_raw(ptr: TableGenRecordRef) -> Record<'a> {
        Record {
            raw: ptr,
            _reference: PhantomData,
        }
    }

    /// Returns the name of the record.
    ///
    /// # Errors
    ///
    /// Returns an error if the name is not a valid UTF-8 string.
    pub fn name(self) -> Result<&'a str, Error> {
        unsafe { StringRef::from_raw(tableGenRecordGetName(self.raw)) }
            .try_into()
            .map_err(TableGenError::from)
            .map_err(|e| e.with_location(self))
    }

    record_value!(
        /// Returns the field with the given name converted to a [`Vec<bool>`]
        /// if this field is of type [`BitsInit`](crate::init::BitsInit).
        bits,
        Vec<bool>
    );
    record_value!(
        /// Returns the field with the given name converted to a [`String`]
        /// if this field is of type [`StringInit`](crate::init::StringInit).
        ///
        /// Note that this copies the string into a new string.
        code,
        String
    );
    record_value!(
        /// Returns the field with the given name converted to a [`&str`]
        /// if this field is of type [`StringInit`](crate::init::StringInit).
        code_str,
        &'a str
    );
    record_value!(
        /// Returns the field with the given name converted to a [`String`]
        /// if this field is of type [`StringInit`](crate::init::StringInit).
        ///
        /// Note that this copies the string into a new string.
        string,
        String
    );

    /// Returns a [`RecordValue`] for the field with the given name.
    pub fn value<'n>(self, name: &'n str) -> Result<RecordValue<'a>, Error> {
        let value = unsafe { tableGenRecordGetValue(self.raw, StringRef::from(name).to_raw()) };
        if !value.is_null() {
            Ok(unsafe { RecordValue::from_raw(value) })
        } else {
            Err(TableGenError::MissingValue(String::from(name)).with_location(self))
        }
    }

    /// Returns true if the record is anonymous.
    pub fn anonymous(self) -> bool {
        unsafe { tableGenRecordIsAnonymous(self.raw) > 0 }
    }

    /// Returns true if the record is a subclass of the class with the given
    /// name.
    pub fn subclass_of(self, class: &str) -> bool {
        unsafe { tableGenRecordIsSubclassOf(self.raw, StringRef::from(class).to_raw()) > 0 }
    }

    /// Returns an iterator over the fields of the record.
    ///
    /// The iterator yields [`RecordValue`] structs
    pub fn values(self) -> RecordValueIter<'a> {
        RecordValueIter::new(self)
    }

    /// Returns `true` if the record has a field with the given name.
    pub fn has_field(self, name: &str) -> bool {
        let v = unsafe { tableGenRecordGetValue(self.raw, StringRef::from(name).to_raw()) };
        !v.is_null()
    }

    /// Returns the [`TableGenRecTyKind`](crate::raw::TableGenRecTyKind) of the
    /// field with the given name, or `None` if the field does not exist.
    pub fn field_type(self, name: &str) -> Option<crate::raw::TableGenRecTyKind::Type> {
        use crate::raw::TableGenRecTyKind::TableGenInvalidRecTyKind;
        let kind = unsafe { tableGenRecordGetFieldType(self.raw, StringRef::from(name).to_raw()) };
        if kind == TableGenInvalidRecTyKind {
            None
        } else {
            Some(kind)
        }
    }

    /// Dumps the record to stderr (for debugging).
    pub fn dump(self) {
        unsafe { tableGenRecordDump(self.raw) }
    }

    /// Returns the number of template arguments.
    pub fn num_template_args(self) -> usize {
        unsafe { tableGenRecordGetNumTemplateArgs(self.raw) }
    }

    /// Returns the template argument name at the given index.
    pub fn template_arg_name(self, index: usize) -> Option<&'a str> {
        unsafe { StringRef::from_option_raw(tableGenRecordGetTemplateArgName(self.raw, index)) }
            .and_then(|s| s.try_into().ok())
    }

    /// Returns an iterator over the template argument names.
    pub fn template_args(self) -> TemplateArgIter<'a> {
        let back = self.num_template_args();
        TemplateArgIter {
            record: self,
            index: 0,
            back,
        }
    }

    /// Returns the number of direct super classes.
    pub fn num_super_classes(self) -> usize {
        unsafe { tableGenRecordGetNumSuperClasses(self.raw) }
    }

    /// Returns the super class at the given index.
    pub fn super_class(self, index: usize) -> Option<Record<'a>> {
        let ptr = unsafe { tableGenRecordGetSuperClass(self.raw, index) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Record::from_raw(ptr) })
        }
    }

    /// Returns an iterator over the direct super classes.
    pub fn direct_super_classes(self) -> SuperClassIter<'a> {
        let back = self.num_super_classes();
        SuperClassIter {
            record: self,
            index: 0,
            back,
        }
    }

    /// Returns the integer value of the field with the given name.
    pub fn int_value(self, name: &str) -> Result<i64, Error> {
        let mut out: i64 = 0;
        let ok = unsafe {
            tableGenRecordGetValueAsInt(self.raw, StringRef::from(name).to_raw(), &mut out)
        };
        if ok > 0 {
            Ok(out)
        } else {
            Err(TableGenError::MissingValue(name.into()).with_location(self))
        }
    }

    /// Returns the string value of the field with the given name as a `&str`.
    pub fn str_value(self, name: &str) -> Result<&'a str, Error> {
        let raw = unsafe {
            tableGenRecordGetValueAsString(self.raw, StringRef::from(name).to_raw())
        };
        if raw.data.is_null() {
            Err(TableGenError::MissingValue(name.into()).with_location(self))
        } else {
            unsafe { StringRef::from_raw(raw) }
                .try_into()
                .map_err(TableGenError::from)
                .map_err(|e| e.with_location(self))
        }
    }

    /// Returns the bit (boolean) value of the field with the given name.
    pub fn bit_value(self, name: &str) -> Result<bool, Error> {
        let mut out: i8 = 0;
        let ok = unsafe {
            tableGenRecordGetValueAsBit(self.raw, StringRef::from(name).to_raw(), &mut out)
        };
        if ok > 0 {
            Ok(out != 0)
        } else {
            Err(TableGenError::MissingValue(name.into()).with_location(self))
        }
    }

    /// Returns the def value of the field with the given name as a [`Record`].
    pub fn def_value(self, name: &str) -> Result<Record<'a>, Error> {
        let ptr = unsafe {
            tableGenRecordGetValueAsDef(self.raw, StringRef::from(name).to_raw())
        };
        if ptr.is_null() {
            Err(TableGenError::MissingValue(name.into()).with_location(self))
        } else {
            Ok(unsafe { Record::from_raw(ptr) })
        }
    }

    /// Returns the dag value of the field with the given name.
    pub fn dag_value(self, name: &str) -> Result<DagInit<'a>, Error> {
        let ptr = unsafe {
            tableGenRecordGetValueAsDag(self.raw, StringRef::from(name).to_raw())
        };
        if ptr.is_null() {
            Err(TableGenError::MissingValue(name.into()).with_location(self))
        } else {
            Ok(unsafe { DagInit::from_raw(ptr) })
        }
    }

    /// Returns the bits init of the field with the given name.
    pub fn bits_init_value(self, name: &str) -> Result<BitsInit<'a>, Error> {
        let ptr = unsafe {
            tableGenRecordGetValueAsBitsInit(self.raw, StringRef::from(name).to_raw())
        };
        if ptr.is_null() {
            Err(TableGenError::MissingValue(name.into()).with_location(self))
        } else {
            Ok(unsafe { BitsInit::from_raw(ptr) })
        }
    }

    /// Returns the list init of the field with the given name.
    pub fn list_init_value(self, name: &str) -> Result<ListInit<'a>, Error> {
        let ptr = unsafe {
            tableGenRecordGetValueAsListInit(self.raw, StringRef::from(name).to_raw())
        };
        if ptr.is_null() {
            Err(TableGenError::MissingValue(name.into()).with_location(self))
        } else {
            Ok(unsafe { ListInit::from_raw(ptr) })
        }
    }

    /// Returns the list-of-defs value of the field with the given name.
    pub fn list_of_defs_value(self, name: &str) -> Result<Vec<Record<'a>>, Error> {
        let ptr = unsafe {
            tableGenRecordGetValueAsListOfDefs(self.raw, StringRef::from(name).to_raw())
        };
        if ptr.is_null() {
            return Err(TableGenError::MissingValue(name.into()).with_location(self));
        }
        let len = unsafe { crate::raw::tableGenRecordVectorSize(ptr) };
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            let rec = unsafe { crate::raw::tableGenRecordVectorGet(ptr, i) };
            result.push(unsafe { Record::from_raw(rec) });
        }
        unsafe { crate::raw::tableGenRecordVectorFree(ptr) };
        Ok(result)
    }

    /// Returns the list-of-ints value of the field with the given name.
    pub fn list_of_ints_value(self, name: &str) -> Result<Vec<i64>, Error> {
        let mut arr = std::ptr::null_mut();
        let mut len: usize = 0;
        let ok = unsafe {
            tableGenRecordGetValueAsListOfInts(
                self.raw,
                StringRef::from(name).to_raw(),
                &mut arr,
                &mut len,
            )
        };
        if ok == 0 {
            return Err(TableGenError::MissingValue(name.into()).with_location(self));
        }
        let result = unsafe { std::slice::from_raw_parts(arr, len) }.to_vec();
        unsafe { tableGenIntArrayFree(arr) };
        Ok(result)
    }

    /// Returns the list-of-strings value of the field with the given name.
    pub fn list_of_strings_value(self, name: &str) -> Result<Vec<&'a str>, Error> {
        let mut arr = std::ptr::null_mut();
        let mut len: usize = 0;
        let ok = unsafe {
            tableGenRecordGetValueAsListOfStrings(
                self.raw,
                StringRef::from(name).to_raw(),
                &mut arr,
                &mut len,
            )
        };
        if ok == 0 {
            return Err(TableGenError::MissingValue(name.into()).with_location(self));
        }
        let raw_refs = unsafe { std::slice::from_raw_parts(arr, len) };
        let mut result = Vec::with_capacity(len);
        for raw_ref in raw_refs {
            let s: &'a str = unsafe { StringRef::from_raw(*raw_ref) }
                .try_into()
                .map_err(TableGenError::from)
                .map_err(|e| e.with_location(self))?;
            result.push(s);
        }
        unsafe { tableGenStringRefArrayFree(arr) };
        Ok(result)
    }

    /// Returns the optional string value of the field, or `None` if unset.
    ///
    /// Returns `Err` if the field does not exist or is not a string/unset.
    pub fn optional_str_value(self, name: &str) -> Result<Option<&'a str>, Error> {
        let mut out = crate::raw::TableGenStringRef {
            data: std::ptr::null(),
            len: 0,
        };
        let ok = unsafe {
            tableGenRecordGetValueAsOptionalString(
                self.raw,
                StringRef::from(name).to_raw(),
                &mut out,
            )
        };
        if ok == 0 {
            return Err(TableGenError::MissingValue(name.into()).with_location(self));
        }
        if out.data.is_null() {
            Ok(None)
        } else {
            let s: &'a str = unsafe { StringRef::from_raw(out) }
                .try_into()
                .map_err(TableGenError::from)
                .map_err(|e| e.with_location(self))?;
            Ok(Some(s))
        }
    }

    /// Returns the optional def value of the field, or `None` if unset.
    pub fn optional_def_value(self, name: &str) -> Option<Record<'a>> {
        let ptr = unsafe {
            tableGenRecordGetValueAsOptionalDef(self.raw, StringRef::from(name).to_raw())
        };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Record::from_raw(ptr) })
        }
    }

    /// Returns true if the named field exists and has an unset value (`?`).
    pub fn is_value_unset(self, name: &str) -> bool {
        unsafe {
            tableGenRecordIsValueUnset(self.raw, StringRef::from(name).to_raw()) > 0
        }
    }

    /// Returns true if this record is a class definition (not a def).
    pub fn is_class(self) -> bool {
        unsafe { tableGenRecordIsClass(self.raw) > 0 }
    }

    /// Returns the [`DefInit`] for this record.
    pub fn def_init(self) -> DefInit<'a> {
        unsafe { DefInit::from_raw(tableGenRecordGetDefInit(self.raw)) }
    }

    /// Returns the unique numeric ID of this record.
    pub fn id(self) -> u32 {
        unsafe { tableGenRecordGetID(self.raw) }
    }

    /// Returns the name as a raw [`TypedInit`].
    pub fn name_init(self) -> TypedInit<'a> {
        unsafe { TypedInit::from_raw(tableGenRecordGetNameInit(self.raw)) }
    }

    /// Returns true if the given record is a direct superclass of this record.
    pub fn has_direct_super_class(self, super_class: Record<'a>) -> bool {
        unsafe { tableGenRecordHasDirectSuperClass(self.raw, super_class.raw) > 0 }
    }

    /// Returns the number of classes in this record's type.
    pub fn num_type_classes(self) -> usize {
        unsafe { tableGenRecordRecTyGetNumClasses(self.raw) }
    }

    /// Returns the class at the given index in this record's type.
    pub fn type_class(self, index: usize) -> Option<Record<'a>> {
        let ptr = unsafe { tableGenRecordRecTyGetClass(self.raw, index) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Record::from_raw(ptr) })
        }
    }

    /// Returns true if this record's type is a subclass of the given class.
    pub fn type_is_subclass_of(self, class: Record<'a>) -> bool {
        unsafe { tableGenRecordRecTyIsSubClassOf(self.raw, class.raw) > 0 }
    }
}

impl SourceLoc for Record<'_> {
    fn source_location(self) -> SourceLocation {
        unsafe { SourceLocation::from_raw(tableGenRecordGetLoc(self.raw)) }
    }
}

macro_rules! try_into {
    ($type:ty) => {
        impl<'a> TryFrom<RecordValue<'a>> for $type {
            type Error = Error;

            fn try_from(record_value: RecordValue<'a>) -> Result<Self, Self::Error> {
                Self::try_from(record_value.init).map_err(|e| e.set_location(record_value))
            }
        }
    };
}

try_into!(bool);
try_into!(Vec<bool>);
try_into!(Vec<BitInit<'a>>);
try_into!(i64);
try_into!(ListInit<'a>);
try_into!(DagInit<'a>);
try_into!(Record<'a>);
try_into!(String);
try_into!(&'a str);

impl<'a> From<RecordValue<'a>> for TypedInit<'a> {
    fn from(value: RecordValue<'a>) -> Self {
        value.init
    }
}

/// Struct that represents a field of a [`Record`].
///
/// Can be converted into a Rust type using the [`TryInto`] trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordValue<'a> {
    raw: TableGenRecordValRef,
    pub name: StringInit<'a>,
    pub init: TypedInit<'a>,
    _reference: PhantomData<&'a TableGenRecordRef>,
}

impl Display for RecordValue<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut data = (formatter, Ok(()));

        unsafe {
            tableGenRecordValPrint(
                self.raw,
                Some(print_callback),
                &mut data as *mut _ as *mut c_void,
            );
        }

        data.1
    }
}

impl std::hash::Hash for RecordValue<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl RecordValue<'_> {
    /// Creates a record from a raw object.
    ///
    /// # Safety
    ///
    /// The raw object must be valid.
    pub unsafe fn from_raw(ptr: TableGenRecordValRef) -> Self {
        let name = unsafe { StringInit::from_raw(tableGenRecordValGetNameInit(ptr)) };
        let value = unsafe { TypedInit::from_raw(tableGenRecordValGetValue(ptr)) };
        Self {
            name,
            init: value,
            raw: ptr,
            _reference: PhantomData,
        }
    }

    /// Dumps this record value to stderr (for debugging).
    pub fn dump(self) {
        unsafe { tableGenRecordValDump(self.raw) }
    }

    /// Returns true if this field is a template argument.
    pub fn is_template_arg(self) -> bool {
        unsafe { tableGenRecordValIsTemplateArg(self.raw) > 0 }
    }

    /// Returns true if nonconcrete values are allowed for this field.
    pub fn is_nonconcrete_ok(self) -> bool {
        unsafe { tableGenRecordValIsNonconcreteOK(self.raw) > 0 }
    }

    /// If this field is bits-typed, returns the bit width.
    pub fn bits_width(self) -> Option<usize> {
        let w = unsafe { tableGenRecordValGetBitsWidth(self.raw) };
        if w == 0 { None } else { Some(w) }
    }

    /// If this field is list-typed, returns the element type kind.
    pub fn list_element_type(self) -> Option<crate::raw::TableGenRecTyKind::Type> {
        let k = unsafe { tableGenRecordValGetListElementType(self.raw) };
        if k == RawInvalidRecTyKind { None } else { Some(k) }
    }
}

impl SourceLoc for RecordValue<'_> {
    fn source_location(self) -> SourceLocation {
        unsafe { SourceLocation::from_raw(tableGenRecordValGetLoc(self.raw)) }
    }
}

/// Iterator over the fields of a [`Record`].
#[derive(Debug, Clone)]
pub struct RecordValueIter<'a> {
    record: TableGenRecordRef,
    current: TableGenRecordValRef,
    _reference: PhantomData<&'a TableGenRecordRef>,
}

impl<'a> RecordValueIter<'a> {
    fn new(record: Record<'a>) -> RecordValueIter<'a> {
        unsafe {
            RecordValueIter {
                record: record.raw,
                current: tableGenRecordGetFirstValue(record.raw),
                _reference: PhantomData,
            }
        }
    }
}

impl<'a> Iterator for RecordValueIter<'a> {
    type Item = RecordValue<'a>;

    fn next(&mut self) -> Option<RecordValue<'a>> {
        if self.current.is_null() {
            return None;
        }
        let res = unsafe { RecordValue::from_raw(self.current) };
        self.current = unsafe { tableGenRecordValNext(self.record, self.current) };
        Some(res)
    }
}

impl std::iter::FusedIterator for RecordValueIter<'_> {}

/// Iterator over the template argument names of a [`Record`].
#[derive(Debug, Clone)]
pub struct TemplateArgIter<'a> {
    record: Record<'a>,
    index: usize,
    back: usize,
}

impl<'a> Iterator for TemplateArgIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.back {
            return None;
        }
        let name = self.record.template_arg_name(self.index);
        self.index += 1;
        name
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a> DoubleEndedIterator for TemplateArgIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index >= self.back {
            return None;
        }
        self.back -= 1;
        match self.record.template_arg_name(self.back) {
            Some(name) => Some(name),
            None => {
                self.back += 1;
                None
            }
        }
    }
}

impl ExactSizeIterator for TemplateArgIter<'_> {}

impl std::iter::FusedIterator for TemplateArgIter<'_> {}

/// Iterator over the direct super classes of a [`Record`].
#[derive(Debug, Clone)]
pub struct SuperClassIter<'a> {
    record: Record<'a>,
    index: usize,
    back: usize,
}

impl<'a> Iterator for SuperClassIter<'a> {
    type Item = Record<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.back {
            return None;
        }
        let class = self.record.super_class(self.index);
        self.index += 1;
        class
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a> DoubleEndedIterator for SuperClassIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index >= self.back {
            return None;
        }
        self.back -= 1;
        match self.record.super_class(self.back) {
            Some(class) => Some(class),
            None => {
                self.back += 1;
                None
            }
        }
    }
}

impl ExactSizeIterator for SuperClassIter<'_> {}

impl std::iter::FusedIterator for SuperClassIter<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TableGenParser;

    #[test]
    fn record() {
        let rk = TableGenParser::new()
            .add_source(
                r#"
                class A;
                class B;
                class C;

                def D1: A;
                def D2: A, B;
                def : B, C;
                "#,
            )
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let d2 = rk.def("D2").expect("D2 exists");
        assert!(d2.subclass_of("A"));
        assert!(d2.subclass_of("B"));
        assert!(!d2.subclass_of("C"));
        assert!(!d2.subclass_of("D"));
        let anon = rk
            .defs()
            .map(|(_name, def)| def)
            .find(|d| d.anonymous())
            .expect("anonymous class exists");
        assert!(!anon.subclass_of("A"));
        assert!(anon.subclass_of("B"));
        assert!(anon.subclass_of("C"));
    }

    #[test]
    fn single_value() {
        let rk = TableGenParser::new()
            .add_source(
                r#"
                def A {
                    int size = 42;
                }
                "#,
            )
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let a = rk.def("A").expect("def A exists");
        assert_eq!(a.name(), Ok("A"));
        assert_eq!(a.int_value("size"), Ok(42));
        assert_eq!(
            a.value("size")
                .and_then(|v| {
                    assert!(v.name.to_str() == Ok("size"));
                    v.init.as_int().map_err(|e| e.set_location(v))
                })
                .and_then(|i| {
                    i64::try_from(i)
                        .map_err(|e| e.with_location(crate::error::SourceLocation::none()))
                }),
            Ok(42)
        );
    }

    #[test]
    fn values() {
        let rk = TableGenParser::new()
            .add_source(
                r#"
                def A {
                    int a = 5;
                    string n = "hello";
                }
                "#,
            )
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let a = rk.def("A").expect("def A exists");
        let values = a.values();
        assert_eq!(values.clone().count(), 2);
        for v in values {
            match v.init {
                TypedInit::Int(i) => {
                    assert_eq!(v.name.to_str(), Ok("a"));
                    assert_eq!(i64::try_from(i).unwrap(), 5);
                }
                TypedInit::String(i) => {
                    assert_eq!(v.name.to_str(), Ok("n"));
                    assert_eq!(i.to_str(), Ok("hello"));
                }
                _ => panic!("unexpected type"),
            }
        }
    }

    #[test]
    fn empty_record_values() {
        let rk = TableGenParser::new()
            .add_source("def Empty;")
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let r = rk.def("Empty").expect("def Empty exists");
        assert_eq!(r.values().count(), 0);
        // Calling next() on an already-exhausted iterator must not invoke UB.
        let mut iter = r.values();
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    #[test]
    fn print_error() {
        let rk = TableGenParser::new()
            .add_source(
                r#"
                class C<int test> {
                    int a = test;
                }
                def A : C<4>;
                "#,
            )
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let a = rk.def("A").expect("def A exists");
        if let Err(e) = a.string_value("a") {
            // With source info
            let msg = format!("{}", e.clone().add_source_info(rk.source_info()));
            let msg = msg.trim();
            // LLVM 22+ changed PrintMessage formatting for in-memory buffers.
            #[cfg(any(
                feature = "llvm16-0",
                feature = "llvm17-0",
                feature = "llvm18-0",
                feature = "llvm19-0",
                feature = "llvm20-0",
                feature = "llvm21-0"
            ))]
            assert_eq!(
                msg,
                r#"
                  error: invalid conversion from Int to alloc::string::String
                    int a = test;
                        ^
                "#
                .trim()
            );
            #[cfg(feature = "llvm22-0")]
            assert!(
                msg.contains("error: invalid conversion from Int to alloc::string::String"),
                "unexpected error message: {msg}"
            );

            // Without source info
            drop(rk);
            assert_eq!(
                format!("{}", e).trim(),
                r#"
                  invalid conversion from Int to alloc::string::String
                "#
                .trim()
            );

            // With incorrect source info
            let rk = TableGenParser::new()
                .add_source("def A;")
                .unwrap()
                .parse()
                .expect("valid tablegen");
            assert_eq!(
                format!("{}", e.add_source_info(rk.source_info())).trim(),
                "invalid conversion from Int to alloc::string::String\nfailed to print source information: invalid source location"
                .trim()
            );
        } else {
            panic!("expected error")
        }
    }

    #[test]
    fn has_field() {
        let rk = TableGenParser::new()
            .add_source("def A { int x = 1; }")
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let a = rk.def("A").expect("def A exists");
        assert!(a.has_field("x"));
        assert!(!a.has_field("y"));
    }

    #[test]
    fn field_type() {
        use crate::raw::TableGenRecTyKind::{
            TableGenBitRecTyKind, TableGenIntRecTyKind, TableGenStringRecTyKind,
        };
        let rk = TableGenParser::new()
            .add_source("def A { int i = 1; string s = \"hi\"; bit b = 1; }")
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let a = rk.def("A").expect("def A exists");
        assert_eq!(a.field_type("i"), Some(TableGenIntRecTyKind));
        assert_eq!(a.field_type("s"), Some(TableGenStringRecTyKind));
        assert_eq!(a.field_type("b"), Some(TableGenBitRecTyKind));
        assert_eq!(a.field_type("nonexistent"), None);
    }

    #[test]
    fn template_args() {
        let rk = TableGenParser::new()
            .add_source("class Foo<int x, string y>;")
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let foo = rk.class("Foo").expect("class Foo exists");
        assert_eq!(foo.num_template_args(), 2);
        assert_eq!(foo.template_arg_name(0), Some("Foo:x"));
        assert_eq!(foo.template_arg_name(1), Some("Foo:y"));
        assert_eq!(foo.template_arg_name(2), None);
        let names: Vec<_> = foo.template_args().collect();
        assert_eq!(names, vec!["Foo:x", "Foo:y"]);
        // Double-ended
        let mut iter = foo.template_args();
        assert_eq!(iter.next_back(), Some("Foo:y"));
        assert_eq!(iter.next(), Some("Foo:x"));
        assert!(iter.next().is_none());
    }

    #[test]
    fn template_args_on_def() {
        let rk = TableGenParser::new()
            .add_source("class A; def D: A;")
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let d = rk.def("D").expect("def D exists");
        assert_eq!(d.num_template_args(), 0);
        assert_eq!(d.template_arg_name(0), None);
        assert_eq!(d.template_args().count(), 0);
    }

    #[test]
    fn template_args_no_params() {
        let rk = TableGenParser::new()
            .add_source("class Foo;")
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let foo = rk.class("Foo").expect("class Foo exists");
        assert_eq!(foo.num_template_args(), 0);
        assert_eq!(foo.template_args().count(), 0);
        let mut iter = foo.template_args();
        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());
    }

    #[test]
    fn template_args_len_tracks() {
        let rk = TableGenParser::new()
            .add_source("class Foo<int a, int b, int c>;")
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let foo = rk.class("Foo").expect("class Foo exists");
        let mut iter = foo.template_args();
        assert_eq!(iter.len(), 3);
        iter.next();
        assert_eq!(iter.len(), 2);
        iter.next_back();
        assert_eq!(iter.len(), 1);
        iter.next();
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());
    }

    #[test]
    fn super_classes() {
        let rk = TableGenParser::new()
            .add_source(
                r#"
                class A;
                class B;
                class C;
                def D: A, B, C;
                "#,
            )
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let d = rk.def("D").expect("def D exists");
        assert_eq!(d.num_super_classes(), 3);
        assert_eq!(d.super_class(0).unwrap().name().unwrap(), "A");
        assert_eq!(d.super_class(1).unwrap().name().unwrap(), "B");
        assert_eq!(d.super_class(2).unwrap().name().unwrap(), "C");
        assert!(d.super_class(3).is_none());
        let names: Vec<_> = d
            .direct_super_classes()
            .map(|r| r.name().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["A", "B", "C"]);
        // Double-ended
        let mut iter = d.direct_super_classes();
        assert_eq!(iter.next_back().unwrap().name().unwrap(), "C");
        assert_eq!(iter.next().unwrap().name().unwrap(), "A");
        assert_eq!(iter.next().unwrap().name().unwrap(), "B");
        assert!(iter.next().is_none());
    }

    #[test]
    fn super_classes_on_class() {
        let rk = TableGenParser::new()
            .add_source("class A; class B: A; class C: A, B;")
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let b = rk.class("B").expect("class B exists");
        assert_eq!(b.num_super_classes(), 1);
        assert_eq!(b.super_class(0).unwrap().name().unwrap(), "A");
        let c = rk.class("C").expect("class C exists");
        assert_eq!(c.num_super_classes(), 2);
        let names: Vec<_> = c
            .direct_super_classes()
            .map(|r| r.name().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["A", "B"]);
    }

    #[test]
    fn super_classes_empty() {
        let rk = TableGenParser::new()
            .add_source("def D;")
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let d = rk.def("D").expect("def D exists");
        assert_eq!(d.num_super_classes(), 0);
        assert!(d.super_class(0).is_none());
        assert_eq!(d.direct_super_classes().count(), 0);
        let mut iter = d.direct_super_classes();
        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());
    }

    #[test]
    fn super_classes_len_tracks() {
        let rk = TableGenParser::new()
            .add_source("class A; class B; class C; def D: A, B, C;")
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let d = rk.def("D").expect("def D exists");
        let mut iter = d.direct_super_classes();
        assert_eq!(iter.len(), 3);
        iter.next();
        assert_eq!(iter.len(), 2);
        iter.next_back();
        assert_eq!(iter.len(), 1);
        iter.next();
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());
    }
}
