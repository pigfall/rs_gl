use crate::{
    native_buffer::{NativeBuffer,GeometryBufferKind,NativeBufferBuilder},
    pipeline_state::PipelineState,
    surface_data::{SurfaceData},
};

use fyrox::utils::{array_as_u8_slice};

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
