#![no_std]
#![allow(clippy::from_over_into)]
#![warn(clippy::missing_safety_doc)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod g4;
mod sys;

use core::{cell::UnsafeCell, ffi::CStr, mem::MaybeUninit, ptr::NonNull};
// use std::process::Termination;
use g4::Graphics4;

pub use krafix::compile_shader as krafix_compile;

#[cfg(all(feature = "opengl", any(target_os = "linux", target_os = "android")))]
#[macro_export]
macro_rules! compile_shader {
    ($t:ident, $source:expr) => {
        $crate::krafix_compile!($t, essl, $source)
    };
}
#[cfg(all(feature = "opengl", target_os = "windows"))]
#[macro_export]
macro_rules! compile_shader {
    ($t:ident, $source:expr) => {
        $crate::krafix_compile!($t, glsl, $source)
    };
}
#[cfg(all(feature = "metal", not(feature = "opengl")))]
#[macro_export]
macro_rules! compile_shader {
    ($t:ident, $source:expr) => {
        $crate::krafix_compile!($t, metal, $source)
    };
}
#[cfg(all(feature = "vulkan", not(feature = "opengl")))]
#[macro_export]
macro_rules! compile_shader {
    ($t:ident, $source:expr) => {
        $crate::krafix_compile!($t, spirv, $source)
    };
}
#[cfg(any(feature = "d3d12", feature = "d3d11"))]
#[macro_export]
macro_rules! compile_shader {
    ($t:ident, $source:expr) => {
        $crate::krafix_compile!($t, d3d11, $source)
    };
}

use crate::sys::*;

trait GetRaw<T> {
    fn get_raw(&self) -> *mut T;
}

impl<T, N> GetRaw<N> for Option<&T>
where
    T: GetRaw<N>,
{
    fn get_raw(&self) -> *mut N {
        match self {
            Some(x) => x.get_raw(),
            None => core::ptr::null_mut(),
        }
    }
}

bitflags::bitflags! {
    pub struct WindowFeatures:u32 {
        const RESIZEABLE = KINC_WINDOW_FEATURE_RESIZEABLE;
        const MINIMIZABLE = KINC_WINDOW_FEATURE_MINIMIZABLE;
        const MAXIMIZABLE = KINC_WINDOW_FEATURE_MAXIMIZABLE;
        const BORDERLESS = KINC_WINDOW_FEATURE_BORDERLESS;
        const ON_TOP = KINC_WINDOW_FEATURE_ON_TOP ;
    }
}

impl Default for WindowFeatures {
    fn default() -> WindowFeatures {
        WindowFeatures::RESIZEABLE | WindowFeatures::MINIMIZABLE | WindowFeatures::MAXIMIZABLE
    }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowMode {
    Window,
    Fullscreen,
    FullscreenExlusive,
}

impl Default for WindowMode {
    fn default() -> Self {
        WindowMode::Window
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct WindowOptions<'a> {
    title: &'a str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    display_index: i32,
    visible: bool,
    window_features: WindowFeatures,
    mode: WindowMode,
}

impl<'a> WindowOptions<'a> {
    pub fn new() -> Self {
        Self {
            title: "",
            x: 0,
            y: 0,
            width: 500,
            height: 500,
            display_index: 0,
            visible: true,
            window_features: Default::default(),
            mode: Default::default(),
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub fn x(mut self, x: i32) -> Self {
        self.x = x;
        self
    }

    pub fn y(mut self, y: i32) -> Self {
        self.y = y;
        self
    }

    pub fn width(mut self, width: i32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: i32) -> Self {
        self.height = height;
        self
    }

    pub fn display_index(mut self, display_index: i32) -> Self {
        self.display_index = display_index;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn window_features(mut self, window_features: WindowFeatures) -> Self {
        self.window_features = window_features;
        self
    }

    pub fn mode(mut self, mode: WindowMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Into<kinc_window_options> for WindowOptions<'_> {
    fn into(self) -> kinc_window_options {
        kinc_window_options {
            title: self.title.as_ptr().cast(),
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            display_index: self.display_index,
            visible: self.visible,
            window_features: KINC_WINDOW_FEATURE_RESIZEABLE as i32,
            mode: match self.mode {
                WindowMode::Window => kinc_window_mode_t_KINC_WINDOW_MODE_WINDOW,
                WindowMode::Fullscreen => kinc_window_mode_t_KINC_WINDOW_MODE_FULLSCREEN,
                WindowMode::FullscreenExlusive => {
                    kinc_window_mode_t_KINC_WINDOW_MODE_EXCLUSIVE_FULLSCREEN
                }
            },
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct FramebufferOptions {
    pub frequency: i32,
    pub vertical_sync: bool,
    pub color_bits: i32,
    pub depth_bits: i32,
    pub stencil_bits: i32,
    pub samples_per_pixel: i32,
}

impl Into<kinc_framebuffer_options> for FramebufferOptions {
    fn into(self) -> kinc_framebuffer_options {
        kinc_framebuffer_options {
            frequency: self.frequency,
            vertical_sync: self.vertical_sync,
            color_bits: self.color_bits,
            depth_bits: self.depth_bits,
            stencil_bits: self.stencil_bits,
            samples_per_pixel: self.samples_per_pixel,
        }
    }
}

static STATIC_DATA: StaticData = StaticData::new();

struct StaticData {
    data: UnsafeCell<MaybeUninit<(Kinc, NonNull<dyn Callbacks>)>>,
}

impl StaticData {
    const fn new() -> Self {
        Self {
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
    /// # Safety
    /// This function should be called on the same thread as `kinc_start`.
    /// `app` should fullfill the necessary conditions for `NonNull::as_mut`
    unsafe fn init(&self, data: Kinc, app: NonNull<dyn Callbacks>) {
        // Safety: the pointer gotten from self.data is valid, and the mutable reference created is unique
        (unsafe { &mut *self.data.get() }).write((data, app));
    }

    /// # Safety
    /// This function must be called from a Kinc-invoked callback.
    unsafe fn with(&'static self, f: impl FnOnce(&mut Kinc, &mut (dyn Callbacks + 'static))) {
        // Safety: Kinc callbacks are called from the same thread
        // they are called after `Self::init` is called, thus the data is initialized
        let (kinc, app) = unsafe { (*self.data.get()).assume_init_mut() };
        // Safety: the pointer can be safely turned into a reference, since it is derived from a (still-valid) reference.
        f(kinc, unsafe { app.as_mut() });
    }
}

// Safety: if the safety preconditions of the struct's methods are respected, the shared data will always be accessed from the same thread
unsafe impl Sync for StaticData {}

extern "C" fn _update_cb() {
    // Safety: this is a Kinc-invoked callback
    unsafe {
        STATIC_DATA.with(|data, callbacks| {
            callbacks.update(data);
        });
    }
}

pub struct KincBuilder<'a> {
    name: &'a CStr,
    width: i32,
    height: i32,
    window_options: Option<WindowOptions<'a>>,
    framebuffer_options: Option<FramebufferOptions>,
}

impl<'a> KincBuilder<'a> {
    pub fn new(name: &'a CStr, width: i32, height: i32) -> Self {
        Self {
            name,
            width,
            height,
            window_options: None,
            framebuffer_options: None,
        }
    }

    pub fn window_options(mut self, window_options: WindowOptions<'a>) -> Self {
        self.window_options = Some(window_options);
        self
    }

    pub fn framebuffer_options(mut self, framebuffer_options: FramebufferOptions) -> Self {
        self.framebuffer_options = Some(framebuffer_options);
        self
    }

    pub fn build(self) -> (Kinc, Window) {
        // Safety: name is valid and lives long enough to be used in the kinc_init call
        // The window_options and framebuffer_options are either null or valid and live long enough to be used in the kinc_init call
        unsafe {
            kinc_init(
                self.name.as_ptr().cast(),
                self.width,
                self.height,
                match self.window_options {
                    None => core::ptr::null_mut(),
                    Some(options) => &mut options.into() as *mut kinc_window_options,
                },
                match self.framebuffer_options {
                    None => core::ptr::null_mut(),
                    Some(options) => &mut options.into() as *mut kinc_framebuffer_options,
                },
            );
        }

        (Kinc, Window { window: 0 })
    }
}

pub struct Kinc;

impl Kinc {
    pub fn default_window(&self) -> Window {
        Window { window: 0 }
    }

    pub fn g4(&self) -> g4::Graphics4 {
        Graphics4
    }

    pub fn start(self, mut callbacks: impl Callbacks + 'static) {
        // Safety: the update callbacks that use `STATIC_DATA` will always be called from the same thread as `kinc_start`,
        // and the callbacks will not be called after `kinc_start` returns, and thus will never get an invalid `callbacks` objects.
        unsafe {
            STATIC_DATA.init(self, NonNull::new_unchecked(&mut callbacks));
            kinc_set_update_callback(Some(_update_cb));
            kinc_start();
        }
    }
}

pub struct Window {
    window: i32,
}

impl Window {}

pub trait Callbacks {
    fn update(&mut self, _kinc: &mut Kinc) {}
}

#[cfg(any(target_os = "android", target_os = "ios"))]
extern "Rust" {
    fn rust_kickstart();
}

// #[cfg(target_os = "android")]
// extern "C" {
//     fn kinc_internal_android_init(na: *mut u8, savedState: *mut u8, savedStateSize: *mut u8);
// }

// #[cfg(target_os = "android")]
// #[export_name = "ANativeActivity_onCreate"]
// extern "C" fn android_native_activity_on_create(na: *mut u8, savedState: *mut u8, savedStateSize: *mut u8) {
//     unsafe {
//         kinc_internal_android_init(na, savedState, savedStateSize);
//     }
// }

#[cfg(any(target_os = "android", target_os = "ios"))]
#[export_name = "kickstart"]
extern "C" fn kickstart(_argc: core::ffi::c_int, _argv: *mut *mut core::ffi::c_char) {
    unsafe { rust_kickstart() }
}
