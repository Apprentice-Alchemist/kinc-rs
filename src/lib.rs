mod sys;

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

#[derive(Default)]
pub struct WindowOptions<'a> {
    pub title: &'a str,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub display_index: i32,
    pub visible: bool,
    pub window_features: WindowFeatures,
    pub mode: WindowMode,
}

impl<'a> WindowOptions<'a> {
    fn into_raw(&self) -> kinc_window_options {
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
    fn into_raw(&self) -> kinc_framebuffer_options {
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

static mut RS_UPDATE_CALLBACK: Option<fn()> = None;

unsafe extern "C" fn _update_cb() {
    match RS_UPDATE_CALLBACK {
        None => {}
        Some(cb) => cb(),
    }
}

pub fn set_update_callback(callback: fn()) {
    unsafe {
        RS_UPDATE_CALLBACK = Some(callback);
    }
    unsafe {
        kinc_set_update_callback(Some(_update_cb));
    }
}

pub fn init(
    name: &str,
    width: i32,
    height: i32,
    win: Option<WindowOptions>,
    frame: Option<FramebufferOptions>,
) {
    unsafe {
        kinc_init(
            name.as_bytes().as_ptr() as *const ::std::os::raw::c_char,
            width,
            height,
            match win {
                None => std::ptr::null_mut(),
                Some(options) => &mut options.into_raw() as *mut kinc_window_options,
            },
            match frame {
                None => std::ptr::null_mut(),
                Some(options) => &mut options.into_raw() as *mut kinc_framebuffer_options,
            },
        );
    }
}

pub fn start() {
    unsafe {
        kinc_start();
    }
}

pub mod g4 {
    use crate::sys::*;
    pub fn begin() {
        unsafe {
            kinc_g4_begin(0);
        }
    }

    pub fn end() {
        unsafe {
            kinc_g4_begin(0);
        }
    }

    pub fn swap_buffers() -> bool {
        unsafe { kinc_g4_swap_buffers() }
    }
}
