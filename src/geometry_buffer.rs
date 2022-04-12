use crate::{
    native_buffer::{NativeBuffer,GeometryBufferKind,NativeBufferBuilder},
    pipeline_state::PipelineState,
    surface_data::{SurfaceData},
};
use crate::{
    core::{
        algebra::{Vector2, Vector3, Vector4},
        arrayvec::ArrayVec,
        byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt},
        futures::io::Error,
        visitor::prelude::*,
    },
};

use fyrox::utils::{array_as_u8_slice};
use serde::Deserialize;

use crate::core::math::TriangleEdge;
use crate::native_buffer::FrameworkError;
use glow::HasContext;

use crate::{
    core::{math::TriangleDefinition, scope_profile},
};

use std::{cell::Cell, marker::PhantomData, mem::size_of};

pub struct GeometryBuffer {
    state: *mut PipelineState,
    vertex_array_object: glow::VertexArray,
    buffers: Vec<NativeBuffer>,
    element_buffer_object: glow::Buffer,
    element_count: Cell<usize>,
    element_kind: ElementKind,
    // Force compiler to not implement Send and Sync, because OpenGL is not thread-safe.
    thread_mark: PhantomData<*const u8>,
}

impl GeometryBuffer{
    pub fn from_surface_data(
        data: &SurfaceData,
        kind: GeometryBufferKind,
        state: &mut PipelineState,
    ) -> Self {
        let geometry_buffer = GeometryBufferBuilder::new(ElementKind::Triangle)
            .with_buffer_builder(NativeBufferBuilder::from_vertex_buffer(&data.vertex_buffer, kind))
            .build(state)
            .unwrap();

        geometry_buffer
            .bind(state)
            .set_triangles(data.geometry_buffer.triangles_ref());

        geometry_buffer
    }


    pub fn bind<'a>(&'a self, state: &'a mut PipelineState) -> GeometryBufferBinding<'a> {
        scope_profile!();

        state.set_vertex_array_object(Some(self.vertex_array_object));

        // Element buffer object binding is stored inside vertex array object, so
        // it does not modifies state.
        unsafe {
            state
                .gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.element_buffer_object));
        }

        GeometryBufferBinding {
            state,
            buffer: self,
        }
    }
}


pub struct GeometryBufferBinding<'a> {
    state: &'a mut PipelineState,
    buffer: &'a GeometryBuffer,
}

impl<'a> GeometryBufferBinding<'a> {

    unsafe fn set_elements(&self, data: &[u8]) {
        scope_profile!();

        self.state
            .gl
            .buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data, glow::DYNAMIC_DRAW);
    }

    pub fn set_triangles(self, triangles: &[TriangleDefinition]) -> Self {
        scope_profile!();

        assert_eq!(self.buffer.element_kind, ElementKind::Triangle);
        self.buffer.element_count.set(triangles.len());

        unsafe { self.set_elements(array_as_u8_slice(triangles)) }

        self
    }
    pub fn draw(&self) -> DrawCallStatistics {
        scope_profile!();

        let start_index = 0;
        let index_per_element = self.buffer.element_kind.index_per_element();
        let index_count = self.buffer.element_count.get() * index_per_element;

        unsafe { self.draw_internal(start_index, index_count) }

        DrawCallStatistics {
            triangles: self.buffer.element_count.get(),
        }
    }

    unsafe fn draw_internal(&self, start_index: usize, index_count: usize) {
        scope_profile!();

        if index_count > 0 {
            let indices = (start_index * size_of::<u32>()) as i32;
            self.state.gl.draw_elements(
                self.mode(),
                index_count as i32,
                glow::UNSIGNED_INT,
                indices,
            );
        }
    }


    fn mode(&self) -> u32 {
        match self.buffer.element_kind {
            ElementKind::Triangle => glow::TRIANGLES,
            ElementKind::Line => glow::LINES,
        }
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ElementKind {
    Triangle,
    Line,
}

impl ElementKind {
    fn index_per_element(self) -> usize {
        match self {
            ElementKind::Triangle => 3,
            ElementKind::Line => 2,
        }
    }
}


#[derive(Debug, Copy, Clone, Default)]
pub struct DrawCallStatistics {
    pub triangles: usize,
}


pub struct GeometryBufferBuilder {
    element_kind: ElementKind,
    buffers: Vec<NativeBufferBuilder>,
}

impl GeometryBufferBuilder {
    pub fn new(element_kind: ElementKind) -> Self {
        Self {
            element_kind,
            buffers: Default::default(),
        }
    }

    pub fn with_buffer_builder(mut self, builder: NativeBufferBuilder) -> Self {
        self.buffers.push(builder);
        self
    }

    pub fn build(self, state: &mut PipelineState) -> Result<GeometryBuffer, FrameworkError> {
        scope_profile!();

        let vao = unsafe { state.gl.create_vertex_array()? };
        let ebo = unsafe { state.gl.create_buffer()? };

        state.set_vertex_array_object(Some(vao));

        let mut buffers = Vec::new();
        for builder in self.buffers {
            buffers.push(builder.build(state)?);
        }

        Ok(GeometryBuffer {
            state,
            vertex_array_object: vao,
            buffers,
            element_buffer_object: ebo,
            element_count: Cell::new(0),
            element_kind: self.element_kind,
            thread_mark: PhantomData,
        })
    }
}


#[derive(Deserialize, Visit, Debug, PartialEq, Clone)]
pub struct DrawParameters {
    pub cull_face: Option<CullFace>,
    pub color_write: ColorMask,
    pub depth_write: bool,
    pub stencil_test: Option<StencilFunc>,
    pub depth_test: bool,
    pub blend: Option<BlendFunc>,
    pub stencil_op: StencilOp,
}

impl Default for DrawParameters {
    fn default() -> Self {
        Self {
            cull_face: Some(CullFace::Back),
            color_write: Default::default(),
            depth_write: true,
            stencil_test: None,
            depth_test: true,
            blend: None,
            stencil_op: Default::default(),
        }
    }
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Hash, Debug, Deserialize, Visit)]
#[repr(u32)]
pub enum CullFace {
    Back = glow::BACK,
    Front = glow::FRONT,
}

impl Default for CullFace {
    fn default() -> Self {
        Self::Back
    }
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Hash, Debug, Deserialize, Visit)]
pub struct ColorMask {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
    pub alpha: bool,
}

impl Default for ColorMask {
    fn default() -> Self {
        Self {
            red: true,
            green: true,
            blue: true,
            alpha: true,
        }
    }
}

impl ColorMask {
    pub fn all(value: bool) -> Self {
        Self {
            red: value,
            green: value,
            blue: value,
            alpha: value,
        }
    }
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Hash, Debug, Deserialize, Visit)]
pub struct StencilFunc {
    pub func: CompareFunc,
    pub ref_value: u32,
    pub mask: u32,
}

impl Default for StencilFunc {
    fn default() -> Self {
        Self {
            func: CompareFunc::Always,
            ref_value: 0,
            mask: 0xFFFF_FFFF,
        }
    }
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Ord, Hash, Visit, Deserialize, Debug)]
#[repr(u32)]
pub enum CompareFunc {
    /// Never passes.
    Never = glow::NEVER,

    /// Passes if the incoming value is less than the stored value.
    Less = glow::LESS,

    /// Passes if the incoming value is equal to the stored value.
    Equal = glow::EQUAL,

    /// Passes if the incoming value is less than or equal to the stored value.
    LessOrEqual = glow::LEQUAL,

    /// Passes if the incoming value is greater than the stored value.
    Greater = glow::GREATER,

    /// Passes if the incoming value is not equal to the stored value.
    NotEqual = glow::NOTEQUAL,

    /// Passes if the incoming value is greater than or equal to the stored value.
    GreaterOrEqual = glow::GEQUAL,

    /// Always passes.
    Always = glow::ALWAYS,
}

impl Default for CompareFunc {
    fn default() -> Self {
        Self::LessOrEqual
    }
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash, Deserialize, Visit, Debug)]
pub struct BlendFunc {
    pub sfactor: BlendFactor,
    pub dfactor: BlendFactor,
}

impl Default for BlendFunc {
    fn default() -> Self {
        Self {
            sfactor: BlendFactor::One,
            dfactor: BlendFactor::Zero,
        }
    }
}


#[derive(Copy, Clone, Hash, PartialOrd, PartialEq, Eq, Ord, Deserialize, Visit, Debug)]
#[repr(u32)]
pub enum BlendFactor {
    Zero = glow::ZERO,
    One = glow::ONE,
    SrcColor = glow::SRC_COLOR,
    OneMinusSrcColor = glow::ONE_MINUS_SRC_COLOR,
    DstColor = glow::DST_COLOR,
    OneMinusDstColor = glow::ONE_MINUS_DST_COLOR,
    SrcAlpha = glow::SRC_ALPHA,
    OneMinusSrcAlpha = glow::ONE_MINUS_SRC_ALPHA,
    DstAlpha = glow::DST_ALPHA,
    OneMinusDstAlpha = glow::ONE_MINUS_DST_ALPHA,
    ConstantColor = glow::CONSTANT_COLOR,
    OneMinusConstantColor = glow::ONE_MINUS_CONSTANT_COLOR,
    ConstantAlpha = glow::CONSTANT_ALPHA,
    OneMinusConstantAlpha = glow::ONE_MINUS_CONSTANT_ALPHA,
    SrcAlphaSaturate = glow::SRC_ALPHA_SATURATE,
    Src1Color = glow::SRC1_COLOR,
    OneMinusSrc1Color = glow::ONE_MINUS_SRC1_COLOR,
    Src1Alpha = glow::SRC1_ALPHA,
    OneMinusSrc1Alpha = glow::ONE_MINUS_SRC1_ALPHA,
}

impl Default for BlendFactor {
    fn default() -> Self {
        Self::Zero
    }
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Hash, Debug, Deserialize, Visit)]
pub struct StencilOp {
    pub fail: StencilAction,
    pub zfail: StencilAction,
    pub zpass: StencilAction,
    pub write_mask: u32,
}

impl Default for StencilOp {
    fn default() -> Self {
        Self {
            fail: Default::default(),
            zfail: Default::default(),
            zpass: Default::default(),
            write_mask: 0xFFFF_FFFF,
        }
    }
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Hash, Debug, Deserialize, Visit)]
#[repr(u32)]
pub enum StencilAction {
    /// Keeps the current value.
    Keep = glow::KEEP,

    /// Sets the stencil buffer value to 0.
    Zero = glow::ZERO,

    /// Sets the stencil buffer value to ref value.
    Replace = glow::REPLACE,

    /// Increments the current stencil buffer value.
    /// Clamps to the maximum representable unsigned value.
    Incr = glow::INCR,

    /// Increments the current stencil buffer value.
    /// Wraps stencil buffer value to zero when incrementing the maximum representable
    /// unsigned value.
    IncrWrap = glow::INCR_WRAP,

    /// Decrements the current stencil buffer value.
    /// Clamps to 0.
    Decr = glow::DECR,

    /// Decrements the current stencil buffer value.
    /// Wraps stencil buffer value to the maximum representable unsigned value when
    /// decrementing a stencil buffer value of zero.
    DecrWrap = glow::DECR_WRAP,

    /// Bitwise inverts the current stencil buffer value.
    Invert = glow::INVERT,
}

impl Default for StencilAction {
    fn default() -> Self {
        Self::Keep
    }
}
