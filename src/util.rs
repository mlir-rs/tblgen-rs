use std::{
    ffi::c_void,
    fmt::{self, Formatter},
};

use crate::{error::TableGenError, raw::TableGenStringRef, string_ref::StringRef};

pub(crate) unsafe extern "C" fn print_callback(string: TableGenStringRef, data: *mut c_void) {
    let (formatter, result) = unsafe { &mut *(data as *mut (&mut Formatter, fmt::Result)) };

    if result.is_err() {
        return;
    }

    *result = (|| {
        write!(
            formatter,
            "{}",
            TryInto::<&str>::try_into(unsafe { StringRef::from_raw(string) })
                .map_err(|_| fmt::Error)?
        )
    })();
}

pub(crate) unsafe extern "C" fn print_string_callback(
    string: TableGenStringRef,
    data: *mut c_void,
) {
    let (writer, result) = unsafe { &mut *(data as *mut (String, Result<(), TableGenError>)) };

    if result.is_err() {
        return;
    }

    *result = (|| {
        unsafe {
            writer.push_str(StringRef::from_raw(string).as_str()?);
        }

        Ok(())
    })();
}
