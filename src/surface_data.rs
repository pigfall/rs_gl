use crate::vertex_buffer::{VertexBuffer,VertexAttributeUsage,VertexFetchError};
use crate::vertex_buffer::VertexWriteTrait;
use crate::vertex_buffer::VertexReadTrait;
use crate::vertex::{StaticVertex};
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
        algebra::{Vector2, Vector3, Vector4,Matrix4,Point3},
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

impl SurfaceData{

    /// Creates new data source using given vertices and indices.
    pub fn new(
        vertex_buffer: VertexBuffer,
        triangles: TriangleBuffer,
        is_procedural: bool,
    ) -> Self {
        Self {
            vertex_buffer,
            geometry_buffer: triangles,
            is_procedural,
            cache_entry: AtomicIndex::unassigned(),
        }
    }
    pub fn make_cube(transform: Matrix4<f32>) -> Self {
        let vertices = vec![
            // Front
            StaticVertex {
                position: Vector3::new(-0.5, -0.5, 0.5),
                normal: Vector3::z(),
                tex_coord: Vector2::default(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(-0.5, 0.5, 0.5),
                normal: Vector3::z(),
                tex_coord: Vector2::y(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, 0.5, 0.5),
                normal: Vector3::z(),
                tex_coord: Vector2::new(1.0, 1.0),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, -0.5, 0.5),
                normal: Vector3::z(),
                tex_coord: Vector2::x(),
                tangent: Vector4::default(),
            },
            // Back
            StaticVertex {
                position: Vector3::new(-0.5, -0.5, -0.5),
                normal: -Vector3::z(),
                tex_coord: Vector2::default(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(-0.5, 0.5, -0.5),
                normal: -Vector3::z(),
                tex_coord: Vector2::y(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, 0.5, -0.5),
                normal: -Vector3::z(),
                tex_coord: Vector2::new(1.0, 1.0),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, -0.5, -0.5),
                normal: -Vector3::z(),
                tex_coord: Vector2::x(),
                tangent: Vector4::default(),
            },
            // Left
            StaticVertex {
                position: Vector3::new(-0.5, -0.5, -0.5),
                normal: -Vector3::x(),
                tex_coord: Vector2::default(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(-0.5, 0.5, -0.5),
                normal: -Vector3::x(),
                tex_coord: Vector2::y(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(-0.5, 0.5, 0.5),
                normal: -Vector3::x(),
                tex_coord: Vector2::new(1.0, 1.0),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(-0.5, -0.5, 0.5),
                normal: -Vector3::x(),
                tex_coord: Vector2::x(),
                tangent: Vector4::default(),
            },
            // Right
            StaticVertex {
                position: Vector3::new(0.5, -0.5, -0.5),
                normal: Vector3::x(),
                tex_coord: Vector2::default(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, 0.5, -0.5),
                normal: Vector3::x(),
                tex_coord: Vector2::y(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, 0.5, 0.5),
                normal: Vector3::x(),
                tex_coord: Vector2::new(1.0, 1.0),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, -0.5, 0.5),
                normal: Vector3::x(),
                tex_coord: Vector2::x(),
                tangent: Vector4::default(),
            },
            // Top
            StaticVertex {
                position: Vector3::new(-0.5, 0.5, 0.5),
                normal: Vector3::y(),
                tex_coord: Vector2::default(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(-0.5, 0.5, -0.5),
                normal: Vector3::y(),
                tex_coord: Vector2::y(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, 0.5, -0.5),
                normal: Vector3::y(),
                tex_coord: Vector2::new(1.0, 1.0),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, 0.5, 0.5),
                normal: Vector3::y(),
                tex_coord: Vector2::x(),
                tangent: Vector4::default(),
            },
            // Bottom
            StaticVertex {
                position: Vector3::new(-0.5, -0.5, 0.5),
                normal: -Vector3::y(),
                tex_coord: Vector2::default(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(-0.5, -0.5, -0.5),
                normal: -Vector3::y(),
                tex_coord: Vector2::y(),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, -0.5, -0.5),
                normal: -Vector3::y(),
                tex_coord: Vector2::new(1.0, 1.0),
                tangent: Vector4::default(),
            },
            StaticVertex {
                position: Vector3::new(0.5, -0.5, 0.5),
                normal: -Vector3::y(),
                tex_coord: Vector2::x(),
                tangent: Vector4::default(),
            },
        ];

        let triangles = vec![
            TriangleDefinition([2, 1, 0]),
            TriangleDefinition([3, 2, 0]),
            TriangleDefinition([4, 5, 6]),
            TriangleDefinition([4, 6, 7]),
            TriangleDefinition([10, 9, 8]),
            TriangleDefinition([11, 10, 8]),
            TriangleDefinition([12, 13, 14]),
            TriangleDefinition([12, 14, 15]),
            TriangleDefinition([18, 17, 16]),
            TriangleDefinition([19, 18, 16]),
            TriangleDefinition([20, 21, 22]),
            TriangleDefinition([20, 22, 23]),
        ];

        let mut data = Self::new(
            VertexBuffer::new(vertices.len(), StaticVertex::layout(), vertices).unwrap(),
            TriangleBuffer::new(triangles),
            true,
        );
        data.calculate_tangents().unwrap();
        data.transform_geometry(&transform).unwrap();
        data
    }

    pub fn calculate_tangents(&mut self) -> Result<(), VertexFetchError> {
        let mut tan1 = vec![Vector3::default(); self.vertex_buffer.vertex_count() as usize];
        let mut tan2 = vec![Vector3::default(); self.vertex_buffer.vertex_count() as usize];

        for triangle in self.geometry_buffer.iter() {
            let i1 = triangle[0] as usize;
            let i2 = triangle[1] as usize;
            let i3 = triangle[2] as usize;

            let view1 = &self.vertex_buffer.get(i1).unwrap();
            let view2 = &self.vertex_buffer.get(i2).unwrap();
            let view3 = &self.vertex_buffer.get(i3).unwrap();

            let v1 = view1.read_3_f32(VertexAttributeUsage::Position)?;
            let v2 = view2.read_3_f32(VertexAttributeUsage::Position)?;
            let v3 = view3.read_3_f32(VertexAttributeUsage::Position)?;

            let w1 = view1.read_3_f32(VertexAttributeUsage::TexCoord0)?;
            let w2 = view2.read_3_f32(VertexAttributeUsage::TexCoord0)?;
            let w3 = view3.read_3_f32(VertexAttributeUsage::TexCoord0)?;

            let x1 = v2.x - v1.x;
            let x2 = v3.x - v1.x;
            let y1 = v2.y - v1.y;
            let y2 = v3.y - v1.y;
            let z1 = v2.z - v1.z;
            let z2 = v3.z - v1.z;

            let s1 = w2.x - w1.x;
            let s2 = w3.x - w1.x;
            let t1 = w2.y - w1.y;
            let t2 = w3.y - w1.y;

            let r = 1.0 / (s1 * t2 - s2 * t1);

            let sdir = Vector3::new(
                (t2 * x1 - t1 * x2) * r,
                (t2 * y1 - t1 * y2) * r,
                (t2 * z1 - t1 * z2) * r,
            );

            tan1[i1] += sdir;
            tan1[i2] += sdir;
            tan1[i3] += sdir;

            let tdir = Vector3::new(
                (s1 * x2 - s2 * x1) * r,
                (s1 * y2 - s2 * y1) * r,
                (s1 * z2 - s2 * z1) * r,
            );
            tan2[i1] += tdir;
            tan2[i2] += tdir;
            tan2[i3] += tdir;
        }

        let mut vertex_buffer_mut = self.vertex_buffer.modify();
        for (mut view, (t1, t2)) in vertex_buffer_mut.iter_mut().zip(tan1.into_iter().zip(tan2)) {
            let normal = view.read_3_f32(VertexAttributeUsage::Normal)?;

            // Gram-Schmidt orthogonalize
            let tangent = (t1 - normal.scale(normal.dot(&t1)))
                .try_normalize(f32::EPSILON)
                .unwrap_or_else(|| Vector3::new(0.0, 1.0, 0.0));
            let handedness = normal.cross(&t1).dot(&t2).signum();
            view.write_4_f32(
                VertexAttributeUsage::Tangent,
                Vector4::new(tangent.x, tangent.y, tangent.z, handedness),
            )?;
        }

        Ok(())
    }


    pub fn transform_geometry(&mut self, transform: &Matrix4<f32>) -> Result<(), VertexFetchError> {
        // Discard scale by inverse and transpose given transform (M^-1)^T
        let normal_matrix = transform.try_inverse().unwrap_or_default().transpose();

        let mut vertex_buffer_mut = self.vertex_buffer.modify();
        for mut view in vertex_buffer_mut.iter_mut() {
            let position = view.read_3_f32(VertexAttributeUsage::Position)?;
            view.write_3_f32(
                VertexAttributeUsage::Position,
                transform.transform_point(&Point3::from(position)).coords,
            )?;
            let normal = view.read_3_f32(VertexAttributeUsage::Normal)?;
            view.write_3_f32(
                VertexAttributeUsage::Normal,
                normal_matrix.transform_vector(&normal),
            )?;
            let tangent = view.read_4_f32(VertexAttributeUsage::Tangent)?;
            let new_tangent = normal_matrix.transform_vector(&tangent.xyz());
            // Keep sign (W).
            view.write_4_f32(
                VertexAttributeUsage::Tangent,
                Vector4::new(new_tangent.x, new_tangent.y, new_tangent.z, tangent.w),
            )?;
        }

        Ok(())
    }
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
