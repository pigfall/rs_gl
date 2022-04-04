use crate::{
    native_buffer::{NativeBuffer},
    pipeline_state::PipelineState,
};
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
