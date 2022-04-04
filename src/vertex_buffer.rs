use fxhash::FxHasher;
use std::hash::Hash;
use std::hash::Hasher;
use crate::{
    core::{
        algebra::{Vector2, Vector3, Vector4},
        arrayvec::ArrayVec,
        byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt},
        futures::io::Error,
        visitor::prelude::*,
    },
};


#[derive(Clone, Visit, Default, Debug)]
pub struct VertexBuffer {
    dense_layout: Vec<VertexAttribute>,
    sparse_layout: [Option<VertexAttribute>; 13],
    vertex_size_in_byte: u8, 
    vertex_count: u32,
    data: Vec<u8>,
    data_hash: u64,
}

impl VertexBuffer{
    /// Creates new vertex buffer from provided data and with given layout.
    pub fn new<T: Copy>(
        vertex_count: usize,
        layout: &[VertexAttributeDescriptor],
        mut data: Vec<T>,
    ) -> Result<Self, ValidationError> {
        let length = data.len() * std::mem::size_of::<T>();
        let capacity = data.capacity() * std::mem::size_of::<T>();

        let bytes =
            unsafe { Vec::<u8>::from_raw_parts(data.as_mut_ptr() as *mut u8, length, capacity) };

        std::mem::forget(data);

        // Validate for duplicates and invalid layout.
        for descriptor in layout {
            for other_descriptor in layout {
                if !std::ptr::eq(descriptor, other_descriptor) {
                    if descriptor.usage == other_descriptor.usage {
                        return Err(ValidationError::DuplicatedAttributeDescriptor);
                    } else if descriptor.shader_location == other_descriptor.shader_location {
                        return Err(ValidationError::ConflictingShaderLocations(
                            descriptor.shader_location as usize,
                        ));
                    }
                }
            }
        }

        let mut dense_layout = Vec::new();

        // Validate everything as much as possible and calculate vertex size.
        let mut sparse_layout = [None; VertexAttributeUsage::Count as usize];
        let mut vertex_size_bytes = 0u8;
        for attribute in layout.iter() {
            if attribute.size < 1 || attribute.size > 4 {
                return Err(ValidationError::InvalidAttributeSize(
                    attribute.size as usize,
                ));
            }

            let vertex_attribute = VertexAttribute {
                usage: attribute.usage,
                data_type: attribute.data_type,
                size: attribute.size,
                divisor: attribute.divisor,
                offset: vertex_size_bytes,
                shader_location: attribute.shader_location,
            };

            dense_layout.push(vertex_attribute);

            // Map dense to sparse layout to increase performance.
            sparse_layout[attribute.usage as usize] = Some(vertex_attribute);

            vertex_size_bytes += attribute.size * attribute.data_type.size();
        }

        let expected_data_size = vertex_count * vertex_size_bytes as usize;
        if expected_data_size != bytes.len() {
            return Err(ValidationError::InvalidDataSize {
                expected: expected_data_size,
                actual: bytes.len(),
            });
        }

        Ok(Self {
            vertex_size_in_byte: vertex_size_bytes,
            vertex_count: vertex_count as u32,
            data_hash: calculate_data_hash(&bytes),
            data: bytes,
            sparse_layout,
            dense_layout,
        })
    }
}

/// Vertex attribute is a simple "bridge" between raw data and its interpretation. In
/// other words it defines how to treat raw data in vertex shader.
#[derive(Visit, Copy, Clone, Default, Debug)]
pub struct VertexAttribute {
    /// Claimed usage of the attribute. It could be Position, Normal, etc.
    pub usage: VertexAttributeUsage,
    /// Data type of every component of the attribute. It could be F32, U32, U16, etc.
    pub data_type: VertexAttributeDataType,
    /// Size of attribute expressed in components. For example, for `Position` it could
    /// be 3 - which means there are 3 components in attribute of `data_type`.
    pub size: u8,
    /// Sets a "fetch rate" for vertex shader at which it will read vertex attribute:
    ///  0 - per vertex (default)
    ///  1 - per instance
    ///  2 - per 2 instances and so on.
    pub divisor: u8,
    /// Offset in bytes from beginning of the vertex.
    pub offset: u8,
    /// Defines location of the attribute in a shader (`layout(location = x) attrib;`)
    pub shader_location: u8,
}

/// An usage for vertex attribute. It is a fixed set, but there are plenty
/// room for any custom data - it may be fit into `TexCoordN` attributes.
#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Ord, Hash, Visit, Debug)]
#[repr(u32)]
pub enum VertexAttributeUsage {
    /// Vertex position. Usually Vector2<f32> or Vector3<f32>.
    Position = 0,
    /// Vertex normal. Usually Vector3<f32>, more rare Vector3<u16> (F16).
    Normal = 1,
    /// Vertex tangent. Usually Vector3<f32>.
    Tangent = 2,
    /// First texture coordinates. Usually Vector2<f32>.
    /// It may be used for everything else, not only for texture coordinates.
    TexCoord0 = 3,
    /// Second texture coordinates.
    TexCoord1 = 4,
    /// Third texture coordinates.
    TexCoord2 = 5,
    /// Fourth texture coordinates.
    TexCoord3 = 6,
    /// Fifth texture coordinates.
    TexCoord4 = 7,
    /// Sixth texture coordinates.
    TexCoord5 = 8,
    /// Seventh texture coordinates.
    TexCoord6 = 9,
    /// Eighth texture coordinates.
    TexCoord7 = 10,
    /// Bone weights. Usually Vector4<f32>.
    BoneWeight = 11,
    /// Bone indices. Usually Vector4<u8>.
    BoneIndices = 12,
    /// Maximum amount of attribute kinds.
    Count,
}

/// Data type for a vertex attribute component.
#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Ord, Hash, Visit, Debug)]
#[repr(u8)]
pub enum VertexAttributeDataType {
    /// 32-bit floating-point.
    F32,
    /// 32-bit unsigned integer.
    U32,
    /// 16-bit unsigned integer.
    U16,
    /// 8-bit unsigned integer.
    U8,
}

/// Input vertex attribute descriptor used to construct layouts and feed vertex buffer.
#[derive(Debug)]
pub struct VertexAttributeDescriptor {
    /// Claimed usage of the attribute. It could be Position, Normal, etc.
    pub usage: VertexAttributeUsage,
    /// Data type of every component of the attribute. It could be F32, U32, U16, etc.
    pub data_type: VertexAttributeDataType,
    /// Size of attribute expressed in components. For example, for `Position` it could
    /// be 3 - which means there are 3 components in attribute of `data_type`.
    pub size: u8,
    /// Sets a "fetch rate" for vertex shader at which it will read vertex attribute:
    ///  0 - per vertex (default)
    ///  1 - per instance
    ///  2 - per 2 instances and so on.
    pub divisor: u8,
    /// Defines location of the attribute in a shader (`layout(location = x) attrib;`)
    pub shader_location: u8,
}

/// An error that may occur during input data and layout validation.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// Attribute size must be either 1, 2, 3 or 4.
    #[error("Invalid attribute size {0}. Must be either 1, 2, 3 or 4")]
    InvalidAttributeSize(usize),

    /// Data size is not correct.
    #[error("Invalid data size. Expected {}, got {}.", expected, actual)]
    InvalidDataSize {
        /// Expected data size in bytes.
        expected: usize,
        /// Actual data size in bytes.
        actual: usize,
    },

    /// Trying to add vertex of incorrect size.
    #[error("Invalid vertex size. Expected {}, got {}.", expected, actual)]
    InvalidVertexSize {
        /// Expected vertex size.
        expected: u8,
        /// Actual vertex size.
        actual: u8,
    },

    /// A duplicate of a descriptor was found.
    #[error("A duplicate of a descriptor was found.")]
    DuplicatedAttributeDescriptor,

    /// Duplicate shader locations were found.
    #[error("Duplicate shader locations were found {0}.")]
    ConflictingShaderLocations(usize),
}


fn calculate_data_hash(data: &[u8]) -> u64 {
    let mut hasher = FxHasher::default();
    data.hash(&mut hasher);
    hasher.finish()
}


impl VertexAttributeDataType {
    /// Returns size of data in bytes.
    pub fn size(self) -> u8 {
        match self {
            VertexAttributeDataType::F32 | VertexAttributeDataType::U32 => 4,
            VertexAttributeDataType::U16 => 2,
            VertexAttributeDataType::U8 => 1,
        }
    }
}


impl Default for VertexAttributeUsage {
    fn default() -> Self {
        Self::Position
    }
}


impl Default for VertexAttributeDataType {
    fn default() -> Self {
        Self::F32
    }
}
