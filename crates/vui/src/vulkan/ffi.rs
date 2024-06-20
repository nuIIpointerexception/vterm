use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

/// # Safety
///
/// This function is unsafe because it dereferences the pointers in the input
/// slice.
///
/// # Panics
///
/// This function panics if any of the input pointers are null.
pub unsafe fn to_os_ptrs(strings: &Vec<*const i8>) -> (Vec<CString>, Vec<*const c_char>) {
    let mut c_strings = Vec::with_capacity(strings.len());
    let mut c_ptrs = Vec::with_capacity(strings.len());

    for s in strings {
        let c_str = CStr::from_ptr(*s);
        c_strings.push(c_str.to_owned());
        c_ptrs.push(c_str.as_ptr());
    }

    (c_strings, c_ptrs)
}
