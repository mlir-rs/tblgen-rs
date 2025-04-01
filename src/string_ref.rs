use std::{marker::PhantomData, str::Utf8Error};

use crate::raw::TableGenStringRef;

#[derive(Debug, Clone, Copy)]
pub struct StringRef<'a> {
    raw: TableGenStringRef,
    _reference: PhantomData<&'a TableGenStringRef>,
}

impl StringRef<'_> {
    pub unsafe fn to_raw(self) -> TableGenStringRef {
        self.raw
    }

    pub unsafe fn from_raw(raw: TableGenStringRef) -> Self {
        Self {
            raw,
            _reference: PhantomData,
        }
    }

    pub unsafe fn from_option_raw(raw: TableGenStringRef) -> Option<Self> {
        if !raw.data.is_null() {
            Some(unsafe { Self::from_raw(raw) })
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        <&str as TryFrom<Self>>::try_from(*self)
    }
}

impl<'a> From<&'a str> for StringRef<'a> {
    fn from(value: &'a str) -> Self {
        unsafe {
            StringRef::from_raw(TableGenStringRef {
                data: value.as_ptr() as *const _,
                len: value.len(),
            })
        }
    }
}

impl<'a> TryFrom<StringRef<'a>> for &'a str {
    type Error = Utf8Error;

    fn try_from(value: StringRef<'a>) -> Result<Self, Self::Error> {
        std::str::from_utf8(value.into())
    }
}

impl<'a> From<StringRef<'a>> for &'a [u8] {
    fn from(value: StringRef<'a>) -> Self {
        unsafe { std::slice::from_raw_parts(value.raw.data as *const _, value.raw.len) }
    }
}
