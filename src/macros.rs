/* Handy macros to convert strings to C strings */

#[allow(unused_macros)]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [i8] as *const i8
    };
}

#[allow(unused_macros)]
macro_rules! cstr_mut {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *mut [i8] as *mut i8
    };
}

#[allow(unused_macros)]
macro_rules! ucstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [u8] as *const u8
    };
}

#[allow(unused_macros)]
macro_rules! ucstr_mut {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *mut [u8] as *mut u8
    };
}
