bitflags::bitflags! {
    pub struct ClearMode: u32 {
        const COLOR = KINC_G4_CLEAR_COLOR as u32;
        const DEPTH = KINC_G4_CLEAR_DEPTH as u32;
        const STENCIL = KINC_G4_CLEAR_STENCIL as u32;
        const ALL = Self::COLOR.bits | Self::DEPTH.bits | Self::STENCIL.bits;
    }
}

use core::{
    cell::UnsafeCell,
    ffi::{c_void, CStr},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use crate::{sys::*, GetRaw, Window};

pub struct RenderPass<'a> {
    window: &'a Window,
}

impl<'a> RenderPass<'a> {
    pub fn set_render_targets(&mut self, render_targets: &[&RenderTarget]) {
        unsafe {
            kinc_g4_set_render_targets(
                render_targets.as_ptr().cast_mut().cast(),
                render_targets.len().try_into().unwrap(),
            )
        }
    }

    pub fn set_index_buffer(&mut self, index_buffer: &IndexBuffer) {
        // Safety: index_buffer is a valid index buffer.
        unsafe { kinc_g4_set_index_buffer(index_buffer.get_raw()) }
    }

    pub fn set_vertex_buffer(&mut self, vertex_buffer: &VertexBuffer) {
        // Safety: vertex_buffer is a valid vertex buffer.
        unsafe { kinc_g4_set_vertex_buffer(vertex_buffer.get_raw()) }
    }

    pub fn set_pipeline(&mut self, pipeline: &Pipeline) {
        // Safety: pipeline is a valid pipeline.
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

impl Drop for RenderPass<'_> {
    fn drop(&mut self) {
        unsafe {
            kinc_g4_end(self.window.window);
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

impl Into<kinc_g4_usage_t> for Usage {
    fn into(self) -> kinc_g4_usage_t {
        match self {
            Self::Static => kinc_g4_usage_KINC_G4_USAGE_STATIC,
            Self::Dynamic => kinc_g4_usage_KINC_G4_USAGE_DYNAMIC,
            Self::Readable => kinc_g4_usage_KINC_G4_USAGE_READABLE,
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

impl Into<kinc_g4_vertex_data_t> for VertexData {
    fn into(self) -> kinc_g4_vertex_data_t {
        match self {
            Self::None => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_NONE,
            Self::F32_1X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_F32_1X,
            Self::F32_2X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_F32_2X,
            Self::F32_3X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_F32_3X,
            Self::F32_4X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_F32_4X,
            Self::F32_4X4 => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_F32_4X4,
            Self::I8_1X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I8_1X,
            Self::U8_1X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U8_1X,
            Self::I8_1xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I8_1X_NORMALIZED,
            Self::U8_1xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U8_1X_NORMALIZED,
            Self::I8_2X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I8_2X,
            Self::U8_2X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U8_2X,
            Self::I8_2xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I8_2X_NORMALIZED,
            Self::U8_2xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U8_2X_NORMALIZED,
            Self::I8_4X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I8_4X,
            Self::U8_4X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U8_4X,
            Self::I8_4xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I8_4X_NORMALIZED,
            Self::U8_4xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U8_4X_NORMALIZED,
            Self::I16_1X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I16_1X,
            Self::U16_1X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U16_1X,
            Self::I16_1xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I16_1X_NORMALIZED,
            Self::U16_1xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U16_1X_NORMALIZED,
            Self::I16_2X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I16_2X,
            Self::U16_2X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U16_2X,
            Self::I16_2xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I16_2X_NORMALIZED,
            Self::U16_2xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U16_2X_NORMALIZED,
            Self::I16_4X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I16_4X,
            Self::U16_4X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U16_4X,
            Self::I16_4xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I16_4X_NORMALIZED,
            Self::U16_4xNormalized => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U16_4X_NORMALIZED,
            Self::I32_1X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I32_1X,
            Self::U32_1X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U32_1X,
            Self::I32_2X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I32_2X,
            Self::U32_2X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U32_2X,
            Self::I32_3X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I32_3X,
            Self::U32_3X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U32_3X,
            Self::I32_4X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_I32_4X,
            Self::U32_4X => kinc_g4_vertex_data_KINC_G4_VERTEX_DATA_U32_4X,
        }
    }
}

pub struct VertexStructureBuilder<'a> {
    vertex_structure: VertexStructure<'a>,
}

impl<'a> Default for VertexStructureBuilder<'a> {
    fn default() -> Self {
        Self {
            vertex_structure: VertexStructure::new(),
        }
    }
}

impl<'a> VertexStructureBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(mut self, name: &'a CStr, data: VertexData) -> Self {
        unsafe {
            kinc_g4_vertex_structure_add(
                self.vertex_structure.vertex_structure.get_mut(),
                name.as_ptr(),
                data.into(),
            )
        }
        self
    }

    pub fn instanced(mut self, instanced: bool) -> Self {
        self.vertex_structure.vertex_structure.get_mut().instanced = instanced;
        self
    }

    pub fn build(self) -> VertexStructure<'a> {
        self.vertex_structure
    }
}

#[derive(Debug)]
pub struct VertexStructure<'a> {
    vertex_structure: UnsafeCell<kinc_g4_vertex_structure_t>,
    _phantom: core::marker::PhantomData<&'a CStr>,
}

impl<'a> VertexStructure<'a> {
    fn new() -> Self {
        let struc = unsafe {
            let mut struc = MaybeUninit::zeroed();
            kinc_g4_vertex_structure_init(struc.as_mut_ptr());
            struc.assume_init()
        };
        Self {
            vertex_structure: UnsafeCell::new(struc),
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn size(&self) -> i32 {
        unsafe { (*self.vertex_structure.get()).size }
    }
}

impl Clone for VertexStructure<'_> {
    fn clone(&self) -> Self {
        unsafe {
            Self {
                vertex_structure: UnsafeCell::new(*self.vertex_structure.get()),
                _phantom: core::marker::PhantomData,
            }
        }
    }
}

impl GetRaw<kinc_g4_vertex_structure> for VertexStructure<'_> {
    fn get_raw(&self) -> *mut kinc_g4_vertex_structure {
        self.vertex_structure.get()
    }
}

pub struct VertexBufferDesc<'a> {
    pub count: i32,
    pub vertex_structure: VertexStructure<'a>,
    pub usage: Usage,
    pub instance_data_step_rate: i32,
}

pub struct VertexBuffer {
    vertex_buffer: UnsafeCell<kinc_g4_vertex_buffer>,
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
            core::slice::from_raw_parts(
                self.data,
                (self.count * self.vertex_buffer.stride()) as usize / core::mem::size_of::<T>(),
            )
        }
    }
}

impl<T> DerefMut for VertexLockResult<'_, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.data,
                (self.count * self.vertex_buffer.stride()) as usize / core::mem::size_of::<T>(),
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
    pub fn new(desc: VertexBufferDesc<'_>) -> Self {
        // Safety: usage of zeroed() + the kinc init function should be sufficient to initialize the vertex buffer
        let vertex_buffer = unsafe {
            let mut vb: MaybeUninit<kinc_g4_vertex_buffer> = MaybeUninit::zeroed();
            kinc_g4_vertex_buffer_init(
                vb.as_mut_ptr(),
                desc.count,
                desc.vertex_structure.get_raw(),
                desc.usage.into(),
                desc.instance_data_step_rate,
            );
            vb.assume_init()
        };
        let mut this = Self {
            vertex_buffer: UnsafeCell::new(vertex_buffer),
        };
        // This is only needed because otherwise the GL backend will throw errors when settings the vertex buffer.
        // A potential alternative would be to keep track of wether the buffer has been locked and unlocked on the Rust side,
        // and panic if it is not.
        // But that would make things even more complicated...
        drop(this.lock::<f32>(0, desc.count));

        this
    }

    pub fn count(&self) -> i32 {
        // Safety: self.get_raw gives a valid pointer
        unsafe { kinc_g4_vertex_buffer_count(self.get_raw()) }
    }

    pub fn stride(&self) -> i32 {
        // Safety: self.get_raw gives a valid pointer
        unsafe { kinc_g4_vertex_buffer_stride(self.get_raw()) }
    }

    /// # Panics
    /// If `start < 0` or `start > self.count()`
    /// or if `count <= 0` or `count > self.count`
    /// or if `start + count >= self.count()`
    pub fn lock<T>(&mut self, start: i32, count: i32) -> VertexLockResult<T> {
        assert!(start >= 0);
        assert!(start < self.count());
        assert!(count > 0);
        assert!(count <= self.count());
        assert!(start + count <= self.count());
        // Safety: self.get_raw gives a valid pointer, and kinc_g4_vertex_buffer_lock *should* return a valid pointer, given the conditions asserted above
        let ptr = unsafe { kinc_g4_vertex_buffer_lock(self.get_raw(), start, count) };
        VertexLockResult {
            data: ptr.cast(),
            count,
            vertex_buffer: self,
        }
    }

    pub fn lock_all<T>(&mut self) -> VertexLockResult<T> {
        self.lock(0, self.count())
    }
}

impl GetRaw<kinc_g4_vertex_buffer> for VertexBuffer {
    fn get_raw(&self) -> *mut kinc_g4_vertex_buffer {
        self.vertex_buffer.get()
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
        unsafe { core::slice::from_raw_parts(self.data, self.count as usize) }
    }
}

impl<T: ValidIndexFormat> DerefMut for IndexLockResult<'_, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data, self.count as usize) }
    }
}

impl<T: ValidIndexFormat> Drop for IndexLockResult<'_, T> {
    fn drop(&mut self) {
        unsafe {
            kinc_g4_index_buffer_unlock(self.index_buffer.get_raw());
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IndexBufferFormat {
    U16,
    U32,
}

impl Into<kinc_g4_index_buffer_format_t> for IndexBufferFormat {
    fn into(self) -> kinc_g4_index_buffer_format_t {
        match self {
            IndexBufferFormat::U16 => kinc_g4_index_buffer_format_KINC_G4_INDEX_BUFFER_FORMAT_16BIT,
            IndexBufferFormat::U32 => kinc_g4_index_buffer_format_KINC_G4_INDEX_BUFFER_FORMAT_32BIT,
        }
    }
}

pub struct IndexBuffer {
    index_buffer: UnsafeCell<kinc_g4_index_buffer>,
}

impl IndexBuffer {
    pub fn new(count: i32, usage: Usage, format: IndexBufferFormat) -> Self {
        unsafe {
            let mut index_buffer: MaybeUninit<kinc_g4_index_buffer> = MaybeUninit::zeroed();
            kinc_g4_index_buffer_init(
                index_buffer.as_mut_ptr(),
                count,
                format.into(),
                usage.into(),
            );
            Self {
                index_buffer: UnsafeCell::new(index_buffer.assume_init()),
            }
        }
    }

    pub fn count(&self) -> i32 {
        unsafe { kinc_g4_index_buffer_count(self.get_raw()) }
    }

    pub fn lock<T: ValidIndexFormat>(&self) -> IndexLockResult<'_, T> {
        let ptr = unsafe { kinc_g4_index_buffer_lock(self.get_raw(), 0, self.count()) };
        IndexLockResult {
            data: ptr.cast(),
            count: self.count(),
            index_buffer: self,
        }
    }
}

impl GetRaw<kinc_g4_index_buffer> for IndexBuffer {
    fn get_raw(&self) -> *mut kinc_g4_index_buffer {
        self.index_buffer.get()
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
    texture: UnsafeCell<kinc_g4_texture>,
}

impl GetRaw<kinc_g4_texture> for Texture {
    fn get_raw(&self) -> *mut kinc_g4_texture {
        self.texture.get()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Geometry,
    TessellationControl,
    TessellationEvaluation,
}

impl Into<kinc_g4_shader_type_t> for ShaderType {
    fn into(self) -> kinc_g4_shader_type_t {
        match self {
            ShaderType::Vertex => kinc_g4_shader_type_KINC_G4_SHADER_TYPE_VERTEX,
            ShaderType::Fragment => kinc_g4_shader_type_KINC_G4_SHADER_TYPE_FRAGMENT,
            ShaderType::Geometry => kinc_g4_shader_type_KINC_G4_SHADER_TYPE_GEOMETRY,
            ShaderType::TessellationControl => {
                kinc_g4_shader_type_KINC_G4_SHADER_TYPE_TESSELLATION_CONTROL
            }
            ShaderType::TessellationEvaluation => {
                kinc_g4_shader_type_KINC_G4_SHADER_TYPE_TESSELLATION_EVALUATION
            }
        }
    }
}

pub struct Shader {
    shader: UnsafeCell<kinc_g4_shader_t>,
}

impl Shader {
    pub fn new(code: &[u8], t: ShaderType) -> Self {
        unsafe {
            let mut shader = MaybeUninit::zeroed();
            kinc_g4_shader_init(
                shader.as_mut_ptr(),
                // kinc_g4_shader_init will not mutate the source passed to it (hopefully).
                code.as_ptr().cast::<c_void>() as *mut _,
                code.len(),
                t.into(),
            );
            Self {
                shader: UnsafeCell::new(shader.assume_init()),
            }
        }
    }
}

impl GetRaw<kinc_g4_shader_t> for Shader {
    fn get_raw(&self) -> *mut kinc_g4_shader_t {
        self.shader.get()
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        // Safety: self.get_raw is a valid pointer to an initialized shader object
        unsafe { kinc_g4_shader_destroy(self.get_raw()) }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RenderTargetFormat {
    I32,
    F64,
    I32Red,
    F128,
    I16Depth,
    I8Red,
    F16Red,
}

impl Into<kinc_g4_render_target_format_t> for RenderTargetFormat {
    fn into(self) -> kinc_g4_render_target_format_t {
        match self {
            RenderTargetFormat::I32 => {
                kinc_g4_render_target_format_KINC_G4_RENDER_TARGET_FORMAT_32BIT
            }
            RenderTargetFormat::F64 => {
                kinc_g4_render_target_format_KINC_G4_RENDER_TARGET_FORMAT_64BIT_FLOAT
            }
            RenderTargetFormat::I32Red => {
                kinc_g4_render_target_format_KINC_G4_RENDER_TARGET_FORMAT_32BIT_RED_FLOAT
            }
            RenderTargetFormat::F128 => {
                kinc_g4_render_target_format_KINC_G4_RENDER_TARGET_FORMAT_128BIT_FLOAT
            }
            RenderTargetFormat::I16Depth => {
                kinc_g4_render_target_format_KINC_G4_RENDER_TARGET_FORMAT_16BIT_DEPTH
            }
            RenderTargetFormat::I8Red => {
                kinc_g4_render_target_format_KINC_G4_RENDER_TARGET_FORMAT_8BIT_RED
            }
            RenderTargetFormat::F16Red => {
                kinc_g4_render_target_format_KINC_G4_RENDER_TARGET_FORMAT_16BIT_RED_FLOAT
            }
        }
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
    fn into(self) -> kinc_g4_blending_factor_t {
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

impl Into<kinc_g4_blending_operation_t> for BlendingOperation {
    fn into(self) -> kinc_g4_blending_operation_t {
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

impl Into<kinc_g4_compare_mode_t> for CompareMode {
    fn into(self) -> kinc_g4_compare_mode_t {
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
    fn into(self) -> kinc_g4_cull_mode_t {
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
    fn into(self) -> kinc_g4_stencil_action_t {
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
    pipeline: UnsafeCell<kinc_g4_pipeline>,
}

impl GetRaw<kinc_g4_pipeline> for Pipeline {
    fn get_raw(&self) -> *mut kinc_g4_pipeline {
        self.pipeline.get()
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        // Safety: self.get_raw is a valid pointer to an initialized pipeline
        unsafe { kinc_g4_pipeline_destroy(self.get_raw()) }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Stencil {
    pub mode: CompareMode,
    pub both_pass: StencilAction,
    pub depth_fail: StencilAction,
    pub fail: StencilAction,
}

#[derive(Debug, Copy, Clone)]
pub struct Blending {
    pub source: BlendingFactor,
    pub destination: BlendingFactor,
    pub operation: BlendingOperation,
}

#[derive(Debug, Copy, Clone)]
pub struct ColorAttachment {
    pub write_red: bool,
    pub write_green: bool,
    pub write_blue: bool,
    pub write_alpha: bool,
    pub format: RenderTargetFormat,
}

impl ColorAttachment {
    const fn default() -> Self {
        ColorAttachment {
            write_red: true,
            write_green: true,
            write_blue: true,
            write_alpha: true,
            format: RenderTargetFormat::I32,
        }
    }
}

impl Default for ColorAttachment {
    fn default() -> Self {
        Self::default()
    }
}

pub struct PipelineBuilder<'a> {
    vertex_shader: &'a Shader,
    fragment_shader: &'a Shader,
    geometry_shader: Option<&'a Shader>,
    tessellation_control_shader: Option<&'a Shader>,
    tessellation_evaluation_shader: Option<&'a Shader>,

    input_layout: &'a [VertexStructure<'a>],

    cull_mode: CullMode,
    depth_mode: Option<CompareMode>,

    front_stencil: Option<Stencil>,
    back_stencil: Option<Stencil>,

    stencil_reference_value: i32,
    stencil_read_mask: i32,
    stencil_write_mask: i32,

    blending: Option<Blending>,
    alpha_blending: Option<Blending>,

    color_attachments: &'a [ColorAttachment],
    depth_attachment_bits: i32,
    stencil_attachment_bits: i32,

    conservative_rasterization: bool,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(
        vertex_shader: &'a Shader,
        fragment_shader: &'a Shader,
        input_layout: &'a [VertexStructure],
        color_attachments: &'a [ColorAttachment],
    ) -> Self {
        Self {
            vertex_shader,
            fragment_shader,
            geometry_shader: None,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            input_layout,
            cull_mode: CullMode::Nothing,
            depth_mode: None,
            front_stencil: None,
            back_stencil: None,
            stencil_reference_value: 0,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
            blending: None,
            alpha_blending: None,
            color_attachments,
            depth_attachment_bits: 0,
            stencil_attachment_bits: 0,
            conservative_rasterization: false,
        }
    }

    pub fn geometry_shader(mut self, geometry_shader: &'a Shader) -> Self {
        self.geometry_shader = Some(geometry_shader);
        self
    }

    pub fn tessellation_control_shader(mut self, tessellation_control_shader: &'a Shader) -> Self {
        self.tessellation_control_shader = Some(tessellation_control_shader);
        self
    }

    pub fn tessellation_evaluation_shader(
        mut self,
        tessellation_evaluation_shader: &'a Shader,
    ) -> Self {
        self.tessellation_evaluation_shader = Some(tessellation_evaluation_shader);
        self
    }

    pub fn cull_mode(mut self, cull_mode: CullMode) -> Self {
        self.cull_mode = cull_mode;
        self
    }

    pub fn depth_mode(mut self, depth_mode: Option<CompareMode>) -> Self {
        self.depth_mode = depth_mode;
        self
    }

    pub fn front_stencil(mut self, front_stencil: Option<Stencil>) -> Self {
        self.front_stencil = front_stencil;
        self
    }

    pub fn back_stencil(mut self, back_stencil: Option<Stencil>) -> Self {
        self.back_stencil = back_stencil;
        self
    }

    pub fn stencil_reference_value(mut self, stencil_reference_value: i32) -> Self {
        self.stencil_reference_value = stencil_reference_value;
        self
    }

    pub fn stencil_read_mask(mut self, stencil_read_mask: i32) -> Self {
        self.stencil_read_mask = stencil_read_mask;
        self
    }

    pub fn stencil_write_mask(mut self, stencil_write_mask: i32) -> Self {
        self.stencil_write_mask = stencil_write_mask;
        self
    }

    pub fn blending(mut self, blending: Option<Blending>) -> Self {
        self.blending = blending;
        self
    }

    pub fn alpha_blending(mut self, alpha_blending: Option<Blending>) -> Self {
        self.alpha_blending = alpha_blending;
        self
    }

    pub fn depth_attachment_bits(mut self, depth_attachment_bits: i32) -> Self {
        self.depth_attachment_bits = depth_attachment_bits;
        self
    }

    pub fn stencil_attachment_bits(mut self, stencil_attachment_bits: i32) -> Self {
        self.stencil_attachment_bits = stencil_attachment_bits;
        self
    }

    pub fn conservative_rasterization(mut self, conservative_rasterization: bool) -> Self {
        self.conservative_rasterization = conservative_rasterization;
        self
    }

    pub fn build(self) -> Pipeline {
        let mut pipeline = unsafe {
            let mut pipeline = MaybeUninit::zeroed();
            kinc_g4_pipeline_init(pipeline.as_mut_ptr());
            pipeline.assume_init()
        };
        for (i, vertex_structure) in self.input_layout.iter().enumerate() {
            if i < 16 {
                pipeline.input_layout[i] = vertex_structure.vertex_structure.get()
            }
        }

        pipeline.vertex_shader = self.vertex_shader.get_raw();
        pipeline.fragment_shader = self.fragment_shader.get_raw();
        pipeline.geometry_shader = self.geometry_shader.get_raw();
        pipeline.tessellation_control_shader = self.tessellation_control_shader.get_raw();
        pipeline.tessellation_evaluation_shader = self.tessellation_evaluation_shader.get_raw();
        pipeline.cull_mode = self.cull_mode.into();

        if let Some(depth_mode) = self.depth_mode {
            pipeline.depth_write = true;
            pipeline.depth_mode = depth_mode.into();
        }

        if let Some(s) = self.front_stencil {
            pipeline.stencil_front_mode = s.mode.into();
            pipeline.stencil_front_both_pass = s.both_pass.into();
            pipeline.stencil_front_depth_fail = s.depth_fail.into();
            pipeline.stencil_front_fail = s.fail.into();
        }

        if let Some(s) = self.back_stencil {
            pipeline.stencil_back_mode = s.mode.into();
            pipeline.stencil_back_both_pass = s.both_pass.into();
            pipeline.stencil_back_depth_fail = s.depth_fail.into();
            pipeline.stencil_back_fail = s.fail.into();
        }

        pipeline.stencil_reference_value = self.stencil_reference_value;
        pipeline.stencil_read_mask = self.stencil_read_mask;
        pipeline.stencil_write_mask = self.stencil_write_mask;

        if let Some(blending) = self.blending {
            pipeline.blend_source = blending.source.into();
            pipeline.blend_destination = blending.destination.into();
            pipeline.blend_operation = blending.operation.into();
        }

        if let Some(blending) = self.alpha_blending {
            pipeline.alpha_blend_source = blending.source.into();
            pipeline.alpha_blend_destination = blending.destination.into();
            pipeline.alpha_blend_operation = blending.operation.into();
        }

        pipeline.color_attachment_count = self.color_attachments.len() as i32;
        for (i, color_attachment) in self.color_attachments.iter().enumerate() {
            pipeline.color_write_mask_red[i] = color_attachment.write_red;
            pipeline.color_write_mask_green[i] = color_attachment.write_green;
            pipeline.color_write_mask_blue[i] = color_attachment.write_blue;
            pipeline.color_write_mask_alpha[i] = color_attachment.write_alpha;
            pipeline.color_attachment[i] = color_attachment.format.into();
        }
        pipeline.depth_attachment_bits = self.depth_attachment_bits;
        pipeline.stencil_attachment_bits = self.stencil_attachment_bits;
        pipeline.conservative_rasterization = self.conservative_rasterization;
        unsafe {
            kinc_g4_pipeline_compile(&mut pipeline);
        }
        // We extend the life time of self to ensure that it's data (eg the strings in the vertex structures)
        // is still valid when calling kinc_g4_pipeline_compile
        #[allow(clippy::drop_non_drop)]
        drop(self);

        Pipeline {
            pipeline: UnsafeCell::new(pipeline),
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct RenderTarget {
    target: UnsafeCell<kinc_g4_render_target>,
}

impl GetRaw<kinc_g4_render_target> for RenderTarget {
    fn get_raw(&self) -> *mut kinc_g4_render_target {
        self.target.get()
    }
}

impl Drop for RenderTarget {
    fn drop(&mut self) {
        unsafe { kinc_g4_render_target_destroy(self.get_raw()) }
    }
}

#[derive(Debug)]
pub struct SwapBufferError;

impl core::fmt::Display for SwapBufferError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "swap_buffers failed")
    }
}

#[cfg(std)]
impl std::error::Error for SwapBufferError {}
