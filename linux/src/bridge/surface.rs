//! The surface format, set before Qt opens a window.

#[cxx_qt::bridge]
pub mod ffi {
    #[namespace = "totaldrop"]
    unsafe extern "C++" {
        include!("totaldrop-linux/src/bridge/surface.h");

        /// Ask for 8 bits per colour channel; see `surface.h`. Call before
        /// the application object exists.
        fn use_true_colour();
    }
}

pub use ffi::use_true_colour;
