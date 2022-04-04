use crate::vertex_buffer::{VertexBuffer};
use fxhash::FxHasher;
use std::hash::Hasher;
use std::hash::Hash;
use crate::core::{
        sparse::AtomicIndex,
};
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;

use crate::core::math::{TriangleDefinition};
use fyrox::renderer::{cache::CacheEntry, framework};
use crate::{
    core::{
        algebra::{Vector2, Vector3, Vector4},
        arrayvec::ArrayVec,
        byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt},
        futures::io::Error,
        visitor::prelude::*,
    },
};

/// Data source of a surface. Each surface can share same data source, this is used
/// in instancing technique to render multiple instances of same model at different
/// places.
#[derive(Debug, Clone, Default)]
pub struct SurfaceData {
    /// Current vertex buffer.
    pub vertex_buffer: VertexBuffer,
    /// Current geometry buffer.
    pub geometry_buffer: TriangleBuffer,
    // If true - indicates that surface was generated and does not have reference
    // resource. Procedural data will be serialized.
    is_procedural: bool,
    pub(in crate) cache_entry: AtomicIndex<CacheEntry<framework::geometry_buffer::GeometryBuffer>>,
}





/// A buffer for data that defines connections between vertices.
#[derive(Visit, Default, Clone, Debug)]
pub struct TriangleBuffer {
    triangles: Vec<TriangleDefinition>,
    data_hash: u64,
}


impl TriangleBuffer {
    /// Creates new triangle buffer with given set of triangles.
    pub fn new(triangles: Vec<TriangleDefinition>) -> Self {
        let hash = calculate_triangle_buffer_hash(&triangles);

        Self {
            triangles,
            data_hash: hash,
        }
    }

    /// Creates new ref iterator.
    pub fn iter(&self) -> impl Iterator<Item = &TriangleDefinition> {
        self.triangles.iter()
    }

    /// Returns a ref to inner data with triangles.
    pub fn triangles_ref(&self) -> &[TriangleDefinition] {
        &self.triangles
    }

    /// Sets a new set of triangles.
    pub fn set_triangles(&mut self, triangles: Vec<TriangleDefinition>) {
        self.data_hash = calculate_triangle_buffer_hash(&triangles);
        self.triangles = triangles;
    }

    /// Returns amount of triangles in the buffer.
    pub fn len(&self) -> usize {
        self.triangles.len()
    }

    /// Returns true if the buffer is empty, false - otherwise.
    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    /// Returns cached data hash. Cached value is guaranteed to be in actual state.
    pub fn data_hash(&self) -> u64 {
        self.data_hash
    }

    /// See VertexBuffer::modify for more info.
    pub fn modify(&mut self) -> TriangleBufferRefMut<'_> {
        TriangleBufferRefMut {
            triangle_buffer: self,
        }
    }
}


fn calculate_triangle_buffer_hash(triangles: &[TriangleDefinition]) -> u64 {
    let mut hasher = FxHasher::default();
    triangles.hash(&mut hasher);
    hasher.finish()
}


pub struct TriangleBufferRefMut<'a> {
    triangle_buffer: &'a mut TriangleBuffer,
}

impl<'a> Deref for TriangleBufferRefMut<'a> {
    type Target = TriangleBuffer;

    fn deref(&self) -> &Self::Target {
        self.triangle_buffer
    }
}

impl<'a> DerefMut for TriangleBufferRefMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.triangle_buffer
    }
}

impl<'a> Drop for TriangleBufferRefMut<'a> {
    fn drop(&mut self) {
        self.triangle_buffer.data_hash =
            calculate_triangle_buffer_hash(&self.triangle_buffer.triangles);
    }
}

impl<'a> TriangleBufferRefMut<'a> {
    /// Returns mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TriangleDefinition> {
        self.triangles.iter_mut()
    }

    /// Adds new triangle in the buffer.
    pub fn push(&mut self, triangle: TriangleDefinition) {
        self.triangles.push(triangle)
    }

    /// Clears the buffer.
    pub fn clear(&mut self) {
        self.triangles.clear();
    }
}

impl<'a> Index<usize> for TriangleBufferRefMut<'a> {
    type Output = TriangleDefinition;

    fn index(&self, index: usize) -> &Self::Output {
        &self.triangle_buffer.triangles[index]
    }
}

impl<'a> IndexMut<usize> for TriangleBufferRefMut<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.triangle_buffer.triangles[index]
    }
}
