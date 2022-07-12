mod sys;

use std::ffi::CString;

use crate::sys::*;

trait GetRaw<T> {
    fn get_raw(&self) -> *mut T;
}

impl<T, N> GetRaw<N> for Option<&T>
where
    T: GetRaw<N>,
{
    fn get_raw(&self) -> *mut N {
        match *self {
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
    use crate::{sys::*, GetRaw, IntoRaw};

    pub struct VertexBuffer;
    pub struct IndexBuffer;
    pub struct Texture;
    pub struct Shader {
        shader: kinc_g4_shader_t,
    }

    impl GetRaw<kinc_g4_shader_t> for Shader {
        fn get_raw(&self) -> *mut kinc_g4_shader_t {
            &self.shader as *const kinc_g4_shader_t as *mut kinc_g4_shader_t
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub enum BlendingFactor {
        One,
        Zero,
        SourceAlpha,
        DestAlpha,
        InvSourceAlpha,
        InvDestAlpha,
        SourceColor,
        DestColor,
        InvSourceColor,
        InvDestColor,
    }

    impl BlendingFactor {
        fn into_raw(self) -> kinc_g4_blending_factor_t {
            match self {
                BlendingFactor::One => kinc_g4_blending_factor_t_KINC_G4_BLEND_ONE,
                BlendingFactor::Zero => kinc_g4_blending_factor_t_KINC_G4_BLEND_ZERO,
                BlendingFactor::SourceAlpha => kinc_g4_blending_factor_t_KINC_G4_BLEND_SOURCE_ALPHA,
                BlendingFactor::DestAlpha => kinc_g4_blending_factor_t_KINC_G4_BLEND_DEST_ALPHA,
                BlendingFactor::InvSourceAlpha => {
                    kinc_g4_blending_factor_t_KINC_G4_BLEND_INV_SOURCE_ALPHA
                }
                BlendingFactor::InvDestAlpha => {
                    kinc_g4_blending_factor_t_KINC_G4_BLEND_INV_DEST_ALPHA
                }
                BlendingFactor::SourceColor => kinc_g4_blending_factor_t_KINC_G4_BLEND_SOURCE_COLOR,
                BlendingFactor::DestColor => kinc_g4_blending_factor_t_KINC_G4_BLEND_DEST_COLOR,
                BlendingFactor::InvSourceColor => {
                    kinc_g4_blending_factor_t_KINC_G4_BLEND_INV_SOURCE_COLOR
                }
                BlendingFactor::InvDestColor => {
                    kinc_g4_blending_factor_t_KINC_G4_BLEND_INV_DEST_COLOR
                }
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub enum BlendingOperation {
        Add,
        Subtract,
        ReverseSubtract,
        Min,
        Max,
    }

    impl crate::IntoRaw<kinc_g4_blending_operation_t> for BlendingOperation {
        fn into_raw(self) -> kinc_g4_blending_operation_t {
            match self {
                BlendingOperation::Add => kinc_g4_blending_operation_t_KINC_G4_BLENDOP_ADD,
                BlendingOperation::Subtract => {
                    kinc_g4_blending_operation_t_KINC_G4_BLENDOP_SUBTRACT
                }
                BlendingOperation::ReverseSubtract => {
                    kinc_g4_blending_operation_t_KINC_G4_BLENDOP_REVERSE_SUBTRACT
                }
                BlendingOperation::Min => kinc_g4_blending_operation_t_KINC_G4_BLENDOP_MIN,
                BlendingOperation::Max => kinc_g4_blending_operation_t_KINC_G4_BLENDOP_MAX,
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub enum CompareMode {
        Always,
        Never,
        Equal,
        NotEqual,
        Less,
        LessEqual,
        Greater,
        GreaterEqual,
    }

    impl crate::IntoRaw<kinc_g4_compare_mode_t> for CompareMode {
        fn into_raw(self) -> kinc_g4_compare_mode_t {
            match self {
                CompareMode::Always => kinc_g4_compare_mode_t_KINC_G4_COMPARE_ALWAYS,
                CompareMode::Never => kinc_g4_compare_mode_t_KINC_G4_COMPARE_NEVER,
                CompareMode::Equal => kinc_g4_compare_mode_t_KINC_G4_COMPARE_EQUAL,
                CompareMode::NotEqual => kinc_g4_compare_mode_t_KINC_G4_COMPARE_NOT_EQUAL,
                CompareMode::Less => kinc_g4_compare_mode_t_KINC_G4_COMPARE_LESS,
                CompareMode::LessEqual => kinc_g4_compare_mode_t_KINC_G4_COMPARE_LESS_EQUAL,
                CompareMode::Greater => kinc_g4_compare_mode_t_KINC_G4_COMPARE_GREATER,
                CompareMode::GreaterEqual => kinc_g4_compare_mode_t_KINC_G4_COMPARE_GREATER_EQUAL,
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub enum CullMode {
        Clockwise,
        CounterClockwise,
        Nothing,
    }

    impl CullMode {
        fn into_raw(self) -> kinc_g4_cull_mode_t {
            match self {
                CullMode::Clockwise => kinc_g4_cull_mode_t_KINC_G4_CULL_CLOCKWISE,
                CullMode::CounterClockwise => kinc_g4_cull_mode_t_KINC_G4_CULL_COUNTER_CLOCKWISE,
                CullMode::Nothing => kinc_g4_cull_mode_t_KINC_G4_CULL_NOTHING,
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub enum StencilAction {
        Keep,
        Zero,
        Replace,
        Increment,
        IncrementWrap,
        Decrement,
        DecrementWrap,
        Invert,
    }

    impl StencilAction {
        fn into_raw(self) -> kinc_g4_stencil_action_t {
            match self {
                StencilAction::Keep => kinc_g4_stencil_action_t_KINC_G4_STENCIL_KEEP,
                StencilAction::Zero => kinc_g4_stencil_action_t_KINC_G4_STENCIL_ZERO,
                StencilAction::Replace => kinc_g4_stencil_action_t_KINC_G4_STENCIL_REPLACE,
                StencilAction::Increment => kinc_g4_stencil_action_t_KINC_G4_STENCIL_INCREMENT,
                StencilAction::IncrementWrap => {
                    kinc_g4_stencil_action_t_KINC_G4_STENCIL_INCREMENT_WRAP
                }
                StencilAction::Decrement => kinc_g4_stencil_action_t_KINC_G4_STENCIL_DECREMENT,
                StencilAction::DecrementWrap => {
                    kinc_g4_stencil_action_t_KINC_G4_STENCIL_DECREMENT_WRAP
                }
                StencilAction::Invert => kinc_g4_stencil_action_t_KINC_G4_STENCIL_INVERT,
            }
        }
    }
    pub struct Pipeline {
        pipeline: kinc_g4_pipeline_t,
    }

    pub struct PipelineBuilder<'a> {
        vertex_shader: &'a Shader,
        fragment_shader: &'a Shader,
        geometry_shader: Option<&'a Shader>,
        tessellation_control_shader: Option<&'a Shader>,
        tessellation_evaluation_shader: Option<&'a Shader>,

        cull_mode: CullMode,
        depth_mode: Option<CompareMode>,
    }

    impl<'a> PipelineBuilder<'a> {
        pub fn new(vertex_shader: &'a Shader, fragment_shader: &'a Shader) -> Self {
            Self {
                vertex_shader,
                fragment_shader,
                geometry_shader: None,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                cull_mode: CullMode::Nothing,
                depth_mode: None,
            }
        }

        pub fn build(self) -> Pipeline {
            unsafe {
                let mut pipeline: kinc_g4_pipeline_t = core::mem::zeroed();
                kinc_g4_pipeline_init(&mut pipeline as *mut kinc_g4_pipeline_t);
                pipeline.vertex_shader = self.vertex_shader.get_raw();
                pipeline.fragment_shader = self.fragment_shader.get_raw();
                pipeline.geometry_shader = self.geometry_shader.get_raw();
                pipeline.tessellation_control_shader = self.tessellation_control_shader.get_raw();
                pipeline.tessellation_evaluation_shader =
                    self.tessellation_evaluation_shader.get_raw();
                pipeline.cull_mode = self.cull_mode.into_raw();

                if let Some(depth_mode) = self.depth_mode {
                    pipeline.depth_write = true;
                    pipeline.depth_mode = depth_mode.into_raw();
                }
                kinc_g4_pipeline_compile(&mut pipeline as *mut kinc_g4_pipeline_t);

                Pipeline { pipeline }
            }
        }
    }

    #[derive(Debug)]
    pub struct SwapBufferError;

    impl std::fmt::Display for SwapBufferError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "swap_buffers failed")
        }
    }

    impl std::error::Error for SwapBufferError {}

    pub fn swap_buffers() -> Result<(), SwapBufferError> {
        unsafe {
            if kinc_g4_swap_buffers() {
                Ok(())
            } else {
                Err(SwapBufferError)
            }
        }
    }
}
