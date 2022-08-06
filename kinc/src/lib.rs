#![warn(clippy::missing_safety_doc)]
pub mod g4;
mod sys;

use std::{borrow::BorrowMut, cell::RefCell, ffi::CString};

use g4::Graphics4;

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
            None => std::ptr::null_mut(),
        }
    }
}

trait IntoRaw<T> {
    fn into_raw(self) -> T;
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

impl IntoRaw<kinc_window_options> for WindowOptions<'_> {
    fn into_raw(self) -> kinc_window_options {
        kinc_window_options {
            title: self.title.as_ptr() as *const ::std::os::raw::c_char,
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

impl IntoRaw<kinc_framebuffer_options> for FramebufferOptions {
    fn into_raw(self) -> kinc_framebuffer_options {
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

#[derive(Default)]
struct StaticData {
    kinc: RefCell<Option<Box<Kinc>>>,
    app: RefCell<Option<Box<dyn Callbacks>>>,
}

impl StaticData {
    const fn new() -> Self {
        Self {
            kinc: RefCell::new(None),
            app: RefCell::new(None),
        }
    }
    /// # Safety
    /// This function should be called from the same thread as [`StaticData::with()`] will be called from.
    unsafe fn init(&'static self, data: Kinc, app: impl Callbacks + 'static) {
        self.kinc.replace(Some(Box::new(data)));
        self.app.replace(Some(Box::new(app)));
    }

    /// # Safety
    /// This function must be called *after* `init` has been called
    /// It should also always be called from the same thread.
    unsafe fn with(&'static self, f: impl FnOnce(&mut Kinc, &mut (dyn Callbacks + 'static))) {
        if let Some(kinc) = self.kinc.borrow_mut().as_mut() {
            let mut app = self.app.borrow_mut();
            if let Some(app) = app.as_mut() {
                f(kinc, app.borrow_mut());
            }
        }
    }
}

unsafe impl Sync for StaticData {}

extern "C" fn _update_cb() {
    // Safety: the update callback will always be called from the main thread
    // The update callback won't be called before `kinc_start` has been called
    // which only happens after `STATIC_DATA.init` has been called.
    unsafe {
        STATIC_DATA.with(|data, callbacks| {
            callbacks.update(data);
        });
    }
}

pub struct KincBuilder<'a> {
    name: &'a str,
    width: i32,
    height: i32,
    window_options: Option<WindowOptions<'a>>,
    framebuffer_options: Option<FramebufferOptions>,
}

impl<'a> KincBuilder<'a> {
    pub fn new(name: &'a str, width: i32, height: i32) -> Self {
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
        let name = CString::new(self.name).unwrap();
        // Safety: name is valid and lives long enough to be used in the kinc_init call
        // The window_options and framebuffer_options are either null or valid and live long enough to be used in the kinc_init call
        unsafe {
            kinc_init(
                name.as_ptr(),
                self.width,
                self.height,
                match self.window_options {
                    None => std::ptr::null_mut(),
                    Some(options) => &mut options.into_raw() as *mut kinc_window_options,
                },
                match self.framebuffer_options {
                    None => std::ptr::null_mut(),
                    Some(options) => &mut options.into_raw() as *mut kinc_framebuffer_options,
                },
            );
        }

        (
            Kinc {
                // update_callback: self.update_callback,
            },
            Window { window: 0 },
        )
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

    pub fn start(self, callbacks: impl Callbacks + 'static) {
        // Safety: the update callbacks that use `STATIC_DATA` will always be called from the same thread as `kinc_start`.
        unsafe {
            STATIC_DATA.init(self, callbacks);
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
