#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod kinc {

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    pub fn init() {
        println!("Init");
        unsafe {
            kinc_init(
                "Window".as_ptr() as *const ::std::os::raw::c_char,
                500,
                500,
                std::ptr::null::<kinc_window_options>() as *mut _,
                std::ptr::null::<kinc_framebuffer_options>() as *mut _,
            );
            kinc_start();
        }
    }
}
