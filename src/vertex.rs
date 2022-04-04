
use crate::{
    core::{
        algebra::{Vector2, Vector3, Vector4,Matrix4},
    },
};

use crate::vertex_buffer::{VertexAttributeDescriptor, VertexAttributeUsage, VertexAttributeDataType};

/// A vertex for static meshes.
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)] // OpenGL expects this structure packed as in C
pub struct StaticVertex {
    /// Position of vertex in local coordinates.
    pub position: Vector3<f32>,
    /// Texture coordinates.
    pub tex_coord: Vector2<f32>,
    /// Normal in local coordinates.
    pub normal: Vector3<f32>,
    /// Tangent vector in local coordinates.
    pub tangent: Vector4<f32>,
}


impl StaticVertex {
    /// Creates new vertex from given position and texture coordinates.
    pub fn from_pos_uv(position: Vector3<f32>, tex_coord: Vector2<f32>) -> Self {
        Self {
            position,
            tex_coord,
            normal: Vector3::new(0.0, 1.0, 0.0),
            tangent: Vector4::default(),
        }
    }

    /// Creates new vertex from given position and texture coordinates.
    pub fn from_pos_uv_normal(
        position: Vector3<f32>,
        tex_coord: Vector2<f32>,
        normal: Vector3<f32>,
    ) -> Self {
        Self {
            position,
            tex_coord,
            normal,
            tangent: Vector4::default(),
        }
    }

    /// Returns layout of the vertex.
    pub fn layout() -> &'static [VertexAttributeDescriptor] {
        static LAYOUT: [VertexAttributeDescriptor; 4] = [
            VertexAttributeDescriptor {
                usage: VertexAttributeUsage::Position,
                data_type: VertexAttributeDataType::F32,
                size: 3,
                divisor: 0,
                shader_location: 0,
            },
            VertexAttributeDescriptor {
                usage: VertexAttributeUsage::TexCoord0,
                data_type: VertexAttributeDataType::F32,
                size: 2,
                divisor: 0,
                shader_location: 1,
            },
            VertexAttributeDescriptor {
                usage: VertexAttributeUsage::Normal,
                data_type: VertexAttributeDataType::F32,
                size: 3,
                divisor: 0,
                shader_location: 2,
            },
            VertexAttributeDescriptor {
                usage: VertexAttributeUsage::Tangent,
                data_type: VertexAttributeDataType::F32,
                size: 4,
                divisor: 0,
                shader_location: 3,
            },
        ];
        &LAYOUT
    }
}
