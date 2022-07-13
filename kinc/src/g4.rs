bitflags::bitflags! {
    pub struct ClearMode: u32 {
        const COLOR = KINC_G4_CLEAR_COLOR as u32;
        const DEPTH = KINC_G4_CLEAR_DEPTH as u32;
        const STENCIL = KINC_G4_CLEAR_STENCIL as u32;
        const ALL = Self::COLOR.bits | Self::DEPTH.bits | Self::STENCIL.bits;
    }
}
use std::{
    ffi::CString,
    ops::{Deref, DerefMut},
};

use crate::{sys::*, GetRaw, IntoRaw, Window};

pub struct RenderPass<'a> {
    window: &'a Window,
}

impl<'a> RenderPass<'a> {
    pub fn set_index_buffer(&mut self, index_buffer: &IndexBuffer) {
        unsafe { kinc_g4_set_index_buffer(index_buffer.get_raw()) }
    }

    pub fn set_vertex_buffer(&mut self, vertex_buffer: &VertexBuffer) {
        unsafe { kinc_g4_set_vertex_buffer(vertex_buffer.get_raw()) }
    }

    pub fn set_pipeline(&mut self, pipeline: &Pipeline) {
        unsafe { kinc_g4_set_pipeline(pipeline.get_raw()) }
    }

    pub fn draw_indexed_vertices(&mut self) {
        unsafe { kinc_g4_draw_indexed_vertices() }
    }

    pub fn draw_indexed_vertices_from_to(&mut self, start: i32, count: i32) {
        unsafe { kinc_g4_draw_indexed_vertices_from_to(start, count) }
    }

    pub fn draw_indexed_vertices_from_to_from(
        &mut self,
        start: i32,
        count: i32,
        vertex_offset: i32,
    ) {
        unsafe { kinc_g4_draw_indexed_vertices_from_to_from(start, count, vertex_offset) }
    }

    pub fn clear(&mut self, flags: ClearMode, color: u32, depth: f32, stencil: i32) {
        unsafe {
            kinc_g4_clear(flags.bits(), color, depth, stencil);
        }
    }

    pub fn end(self) {}
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            unsafe {
                kinc_g4_end(self.window.window);
            }
        }
    }
}

pub struct Graphics4;

impl Graphics4 {
    pub fn begin<'a>(&'a mut self, window: &'a Window) -> RenderPass<'a> {
        unsafe {
            kinc_g4_begin(window.window);
        }

        RenderPass { window }
    }

    pub fn swap_buffers(&mut self) -> Result<(), SwapBufferError> {
        unsafe {
            if kinc_g4_swap_buffers() {
                Ok(())
            } else {
                Err(SwapBufferError)
            }
        }
    }
}

pub enum Usage {
    Static,
    Dynamic,
    Readable,
}

impl IntoRaw<kinc_g4_usage_t> for Usage {
    fn into_raw(self) -> kinc_g4_usage_t {
        match self {
            Static => kinc_g4_usage_KINC_G4_USAGE_STATIC,
            Dynamic => kinc_g4_usage_KINC_G4_USAGE_DYNAMIC,
            Readeable => kinc_g4_usage_KINC_G4_USAGE_READABLE,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VertexData {
    None = 0,
    F32_1X = 1,
    F32_2X = 2,
    F32_3X = 3,
    F32_4X = 4,
    F32_4X4 = 5,
    I8_1X = 6,
    U8_1X = 7,
    I8_1xNormalized = 8,
    U8_1xNormalized = 9,
    I8_2X = 10,
    U8_2X = 11,
    I8_2xNormalized = 12,
    U8_2xNormalized = 13,
    I8_4X = 14,
    U8_4X = 15,
    I8_4xNormalized = 16,
    U8_4xNormalized = 17,
    I16_1X = 18,
    U16_1X = 19,
    I16_1xNormalized = 20,
    U16_1xNormalized = 21,
    I16_2X = 22,
    U16_2X = 23,
    I16_2xNormalized = 24,
    U16_2xNormalized = 25,
    I16_4X = 26,
    U16_4X = 27,
    I16_4xNormalized = 28,
    U16_4xNormalized = 29,
    I32_1X = 30,
    U32_1X = 31,
    I32_2X = 32,
    U32_2X = 33,
    I32_3X = 34,
    U32_3X = 35,
    I32_4X = 36,
    U32_4X = 37,
}

impl VertexData {
    pub fn size(&self) -> i32 {
        match self {
            Self::None => 0,
            Self::F32_1X => 4,
            Self::F32_2X => 8,
            Self::F32_3X => 12,
            Self::F32_4X => 16,
            Self::F32_4X4 => 64,
            Self::I8_1X | Self::U8_1X | Self::I8_1xNormalized | Self::U8_1xNormalized => 1,
            Self::I8_2X | Self::U8_2X | Self::I8_2xNormalized | Self::U8_2xNormalized => 2,
            Self::I8_4X | Self::U8_4X | Self::I8_4xNormalized | Self::U8_4xNormalized => 4,
            Self::I16_1X | Self::U16_1X | Self::I16_1xNormalized | Self::U16_1xNormalized => 2,
            Self::I16_2X | Self::U16_2X | Self::I16_2xNormalized | Self::U16_2xNormalized => 4,
            Self::I16_4X | Self::U16_4X | Self::I16_4xNormalized | Self::U16_4xNormalized => 8,
            Self::I32_1X | Self::U32_1X => 4,
            Self::I32_2X | Self::U32_2X => 8,
            Self::I32_3X | Self::U32_3X => 12,
            Self::I32_4X | Self::U32_4X => 16,
        }
    }
}

pub struct VertexElement<'a> {
    name: &'a str,
    data: VertexData,
}

#[derive(Default)]
pub struct VertexStructureBuilder<'a> {
    elements: Vec<VertexElement<'a>>,
    instanced: bool,
}

impl<'a> VertexStructureBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(mut self, name: &'a str, data: VertexData) -> Self {
        self.elements.push(VertexElement { name, data });
        self
    }

    pub fn instanced(mut self, instanced: bool) -> Self {
        self.instanced = instanced;
        self
    }

    pub fn build(self) -> VertexStructure {
        unsafe {
            let mut vertex_structure: VertexStructure = std::mem::zeroed();
            kinc_g4_vertex_structure_init(vertex_structure.get_raw());
            for element in self.elements.iter() {
                let name = CString::new(element.name).unwrap();
                kinc_g4_vertex_structure_add(
                    vertex_structure.get_raw(),
                    name.as_ptr(),
                    element.data as u32,
                );
            }
            vertex_structure.vertex_structure.instanced = self.instanced;
            vertex_structure
        }
    }
}

pub struct VertexStructure {
    vertex_structure: kinc_g4_vertex_structure_t,
}

impl VertexStructure {
    pub fn size(&self) -> i32 {
        return self.vertex_structure.size;
    }
}

impl GetRaw<kinc_g4_vertex_structure> for VertexStructure {
    fn get_raw(&self) -> *mut kinc_g4_vertex_structure {
        &self.vertex_structure as *const _ as *mut _
    }
}

pub struct VertexBufferDesc {
    pub count: i32,
    pub vertex_structure: VertexStructure,
    pub usage: Usage,
    pub instance_data_step_rate: i32,
}

pub struct VertexBuffer {
    vertex_buffer: kinc_g4_vertex_buffer,
}

pub struct VertexLockResult<'a, T> {
    data: *mut T,
    count: i32,
    vertex_buffer: &'a VertexBuffer,
}

impl<T> Deref for VertexLockResult<'_, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(
                self.data,
                (self.count * self.vertex_buffer.stride()) as usize / std::mem::size_of::<T>(),
            )
        }
    }
}

impl<T> DerefMut for VertexLockResult<'_, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data,
                (self.count * self.vertex_buffer.stride()) as usize,
            )
        }
    }
}

impl<T> Drop for VertexLockResult<'_, T> {
    fn drop(&mut self) {
        unsafe {
            kinc_g4_vertex_buffer_unlock(self.vertex_buffer.get_raw(), self.count as i32);
        }
    }
}

impl VertexBuffer {
    pub fn new(desc: VertexBufferDesc) -> Self {
        unsafe {
            let mut vertex_buffer: kinc_g4_vertex_buffer = std::mem::zeroed();
            kinc_g4_vertex_buffer_init(
                &mut vertex_buffer as *mut _,
                desc.count,
                desc.vertex_structure.get_raw(),
                desc.usage.into_raw(),
                desc.instance_data_step_rate,
            );
            let this = Self { vertex_buffer };
            // This is only needed because otherwise the GL backend will throw errors when settings the vertex buffer.
            // A potential alternative would be to keep track of wether the buffer has been locked and unlocked on the Rust side,
            // and panic if it is not.
            // But that would make things even more complicated...
            this.lock(0, desc.count)
                .deref_mut()
                .iter_mut()
                .for_each(|x| *x = 0.0);

            this
        }
    }

    pub fn count(&self) -> i32 {
        unsafe { kinc_g4_vertex_buffer_count(self.get_raw()) }
    }

    pub fn stride(&self) -> i32 {
        unsafe { kinc_g4_vertex_buffer_stride(self.get_raw()) }
    }

    pub fn lock<T>(&self, start: i32, count: i32) -> VertexLockResult<T> {
        unsafe {
            let ptr = kinc_g4_vertex_buffer_lock(self.get_raw(), start, count);
            VertexLockResult {
                data: ptr.cast(),
                count,
                vertex_buffer: self,
            }
        }
    }

    pub fn lock_all<T>(&self) -> VertexLockResult<T> {
        self.lock(0, self.count())
    }
}

impl GetRaw<kinc_g4_vertex_buffer> for VertexBuffer {
    fn get_raw(&self) -> *mut kinc_g4_vertex_buffer {
        &self.vertex_buffer as *const kinc_g4_vertex_buffer as *mut kinc_g4_vertex_buffer
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe {
            kinc_g4_vertex_buffer_destroy(self.get_raw());
        }
    }
}

pub trait ValidIndexFormat {}

impl ValidIndexFormat for u16 {}
impl ValidIndexFormat for u32 {}

pub struct IndexLockResult<'a, T: ValidIndexFormat> {
    data: *mut T,
    count: i32,
    index_buffer: &'a IndexBuffer,
}

impl<T: ValidIndexFormat> Deref for IndexLockResult<'_, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(
                self.data,
                self.count as usize,
            )
        }
    }
}

impl<T: ValidIndexFormat> DerefMut for IndexLockResult<'_, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data, self.count as usize) }
    }
}

impl<T: ValidIndexFormat> Drop for IndexLockResult<'_, T> {
    fn drop(&mut self) {
        unsafe {
            kinc_g4_index_buffer_unlock(self.index_buffer.get_raw());
        }
    }
}

#[repr(u32)]
pub enum IndexBufferFormat {
    U16 = kinc_g4_index_buffer_format_KINC_G4_INDEX_BUFFER_FORMAT_16BIT,
    U32 = kinc_g4_index_buffer_format_KINC_G4_INDEX_BUFFER_FORMAT_32BIT,
}

pub struct IndexBuffer {
    index_buffer: kinc_g4_index_buffer,
}

impl IndexBuffer {
    pub fn new(count: i32, usage: Usage, format: IndexBufferFormat) -> Self {
        unsafe {
            let mut index_buffer: kinc_g4_index_buffer = std::mem::zeroed();
            kinc_g4_index_buffer_init(
                &mut index_buffer as *mut _,
                count,
                format as u32,
                usage.into_raw(),
            );
            Self { index_buffer }
        }
    }

    pub fn count(&self) -> i32 {
        unsafe { kinc_g4_index_buffer_count(self.get_raw()) }
    }

    pub fn lock<T: ValidIndexFormat>(&self) -> IndexLockResult<'_, T> {
        unsafe {
            let ptr = kinc_g4_index_buffer_lock(self.get_raw());
            IndexLockResult {
                data: ptr.cast(),
                count: self.count(),
                index_buffer: self,
            }
        }
    }
}

impl GetRaw<kinc_g4_index_buffer> for IndexBuffer {
    fn get_raw(&self) -> *mut kinc_g4_index_buffer {
        &self.index_buffer as *const kinc_g4_index_buffer as *mut kinc_g4_index_buffer
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        unsafe {
            kinc_g4_index_buffer_destroy(self.get_raw());
        }
    }
}

pub struct Texture {
    texture: kinc_g4_texture,
}

impl GetRaw<kinc_g4_texture> for Texture {
    fn get_raw(&self) -> *mut kinc_g4_texture {
        &self.texture as *const kinc_g4_texture as *mut kinc_g4_texture
    }
}

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
            BlendingFactor::InvDestAlpha => kinc_g4_blending_factor_t_KINC_G4_BLEND_INV_DEST_ALPHA,
            BlendingFactor::SourceColor => kinc_g4_blending_factor_t_KINC_G4_BLEND_SOURCE_COLOR,
            BlendingFactor::DestColor => kinc_g4_blending_factor_t_KINC_G4_BLEND_DEST_COLOR,
            BlendingFactor::InvSourceColor => {
                kinc_g4_blending_factor_t_KINC_G4_BLEND_INV_SOURCE_COLOR
            }
            BlendingFactor::InvDestColor => kinc_g4_blending_factor_t_KINC_G4_BLEND_INV_DEST_COLOR,
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
            BlendingOperation::Subtract => kinc_g4_blending_operation_t_KINC_G4_BLENDOP_SUBTRACT,
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
            StencilAction::IncrementWrap => kinc_g4_stencil_action_t_KINC_G4_STENCIL_INCREMENT_WRAP,
            StencilAction::Decrement => kinc_g4_stencil_action_t_KINC_G4_STENCIL_DECREMENT,
            StencilAction::DecrementWrap => kinc_g4_stencil_action_t_KINC_G4_STENCIL_DECREMENT_WRAP,
            StencilAction::Invert => kinc_g4_stencil_action_t_KINC_G4_STENCIL_INVERT,
        }
    }
}
pub struct Pipeline {
    pipeline: kinc_g4_pipeline,
}

impl GetRaw<kinc_g4_pipeline> for Pipeline {
    fn get_raw(&self) -> *mut kinc_g4_pipeline {
        &self.pipeline as *const kinc_g4_pipeline as *mut kinc_g4_pipeline
    }
}

pub struct Stencil {
    pub mode: CompareMode,
    pub both_pass: StencilAction,
    pub depth_fail: StencilAction,
    pub fail: StencilAction,
}

pub struct PipelineBuilder<'a> {
    vertex_shader: &'a Shader,
    fragment_shader: &'a Shader,
    geometry_shader: Option<&'a Shader>,
    tessellation_control_shader: Option<&'a Shader>,
    tessellation_evaluation_shader: Option<&'a Shader>,

    cull_mode: CullMode,
    depth_mode: Option<CompareMode>,

    front_stencil: Option<Stencil>,
    back_stencil: Option<Stencil>,

    stencil_reference_value: i32,
    stencil_read_mask: i32,
    stencil_write_mask: i32,
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
            front_stencil: None,
            back_stencil: None,
            stencil_reference_value: 0,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
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
            pipeline.tessellation_evaluation_shader = self.tessellation_evaluation_shader.get_raw();
            pipeline.cull_mode = self.cull_mode.into_raw();

            if let Some(depth_mode) = self.depth_mode {
                pipeline.depth_write = true;
                pipeline.depth_mode = depth_mode.into_raw();
            }

            // TODO: the rest
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
