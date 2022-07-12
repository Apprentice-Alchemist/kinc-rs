mod sys;

use std::ffi::CString;

use crate::sys::*;

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
#[derive(Default)]
pub struct FramebufferOptions {
    pub frequency: i32,
    pub vertical_sync: bool,
    pub color_bits: i32,
    pub depth_bits: i32,
    pub stencil_bits: i32,
    pub samples_per_pixel: i32,
}

impl FramebufferOptions {
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

static mut RS_UPDATE_CALLBACK: Option<fn(&Kinc)> = None;

unsafe extern "C" fn _update_cb() {
    match RS_UPDATE_CALLBACK {
        Some(cb) => cb(&Kinc {}),
        None => (),
    }
}

fn set_update_callback(callback: Option<fn(&Kinc)>) {
    unsafe {
        RS_UPDATE_CALLBACK = callback;
        kinc_set_update_callback(Some(_update_cb));
    }
}

pub struct KincBuilder<'a> {
    name: &'a str,
    width: i32,
    height: i32,
    window_options: Option<WindowOptions<'a>>,
    framebuffer_options: Option<FramebufferOptions>,
    update_callback: Option<fn(&Kinc)>,
}

impl<'a> KincBuilder<'a> {
    pub fn new(name: &'a str, width: i32, height: i32) -> Self {
        Self {
            name,
            width,
            height,
            window_options: None,
            framebuffer_options: None,
            update_callback: None,
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

    pub fn update_callback(mut self, callback: fn(&Kinc)) -> Self {
        self.update_callback = Some(callback);
        self
    }

    pub fn build(self) -> (Kinc, Window) {
        let name = CString::new(self.name).unwrap();
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
        set_update_callback(self.update_callback);
        (Kinc {}, Window { window: 0 })
    }
}

pub struct Kinc {}

impl Kinc {
    pub fn default_window(&self) -> Window {
        Window { window: 0 }
    }

    pub fn start(self) {
        unsafe {
            kinc_start();
        }
    }
}

pub struct Window {
    window: i32,
}

impl Window {
    pub fn g4(&self) -> Graphics4 {
        Graphics4 {
            window: self.window,
        }
    }
}

bitflags::bitflags! {
    pub struct ClearMode: u32 {
        const COLOR = KINC_G4_CLEAR_COLOR as u32;
        const DEPTH = KINC_G4_CLEAR_DEPTH as u32;
        const STENCIL = KINC_G4_CLEAR_STENCIL as u32;
        const ALL = Self::COLOR.bits | Self::DEPTH.bits | Self::STENCIL.bits;
    }
}

pub struct Graphics4 {
    window: i32,
}

impl Graphics4 {
    pub fn begin(&self) {
        unsafe {
            kinc_g4_begin(self.window);
        }
    }

    pub fn clear(&self, flags: ClearMode, color: u32, depth: f32, stencil: i32) {
        unsafe {
            kinc_g4_clear(flags.bits(), color, depth, stencil);
        }
    }

    pub fn end(&self) {
        unsafe {
            kinc_g4_begin(self.window);
        }
    }
}

pub mod g4 {
    use crate::sys::*;

    pub fn swap_buffers() -> Result<(), ()> {
        unsafe {
            if kinc_g4_swap_buffers() {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}
