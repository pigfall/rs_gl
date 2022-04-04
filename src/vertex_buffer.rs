use fxhash::FxHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;
use std::mem::MaybeUninit;
use fyrox::utils::value_as_u8_slice;
use std::marker::PhantomData;

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

    /// Returns a reference to underlying data buffer slice.
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }
    /// Return vertex size of the buffer.
    pub fn vertex_size_in_byte(&self) -> u8 {
        self.vertex_size_in_byte
    }
    /// Returns vertex buffer layout.
    pub fn layout(&self) -> &[VertexAttribute] {
        &self.dense_layout
    }

    pub fn vertex_count(&self)->u32{
        return self.vertex_count as _;
    }

    /// Returns a read accessor of n-th vertex.
    pub fn get(&self, n: usize) -> Option<VertexViewRef<'_>> {
        let offset = n * self.vertex_size_in_byte as usize;
        if offset < self.data.len() {
            Some(VertexViewRef {
                vertex_data: &self.data[offset..(offset + self.vertex_size_in_byte as usize)],
                sparse_layout: &self.sparse_layout,
            })
        } else {
            None
        }
    }

    pub fn modify(&mut self) -> VertexBufferRefMut<'_> {
        VertexBufferRefMut {
            vertex_buffer: self,
        }
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


/// An error that may occur during fetching using vertex read/write accessor.
#[derive(Debug, thiserror::Error)]
pub enum VertexFetchError {
    /// Trying to read/write non-existent attribute.
    #[error("No attribute with such usage: {0:?}")]
    NoSuchAttribute(VertexAttributeUsage),
    /// IO error.
    #[error("An i/o error has occurred {0:?}")]
    Io(std::io::Error),
}

impl From<std::io::Error> for VertexFetchError {
    fn from(e: Error) -> Self {
        Self::Io(e)
    }
}


/// Read accessor for a vertex with some layout.
#[derive(Debug)]
pub struct VertexViewRef<'a> {
    vertex_data: &'a [u8],
    sparse_layout: &'a [Option<VertexAttribute>],
}

impl<'a> PartialEq for VertexViewRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_data == other.vertex_data
    }
}


/// A trait for read-only vertex data accessor.
pub trait VertexReadTrait {
    #[doc(hidden)]
    fn data_layout_ref(&self) -> (&[u8], &[Option<VertexAttribute>]);

    /// Tries to read an attribute with given usage as a pair of two f32.
    #[inline(always)]
    fn read_2_f32(&self, usage: VertexAttributeUsage) -> Result<Vector2<f32>, VertexFetchError> {
        let (data, layout) = self.data_layout_ref();
        if let Some(attribute) = layout.get(usage as usize).unwrap() {
            let x = (&data[(attribute.offset as usize)..]).read_f32::<LittleEndian>()?;
            let y = (&data[(attribute.offset as usize + 4)..]).read_f32::<LittleEndian>()?;
            Ok(Vector2::new(x, y))
        } else {
            Err(VertexFetchError::NoSuchAttribute(usage))
        }
    }

    /// Tries to read an attribute with given usage as a pair of three f32.
    #[inline(always)]
    fn read_3_f32(&self, usage: VertexAttributeUsage) -> Result<Vector3<f32>, VertexFetchError> {
        let (data, layout) = self.data_layout_ref();
        if let Some(attribute) = layout.get(usage as usize).unwrap() {
            let x = (&data[(attribute.offset as usize)..]).read_f32::<LittleEndian>()?;
            let y = (&data[(attribute.offset as usize + 4)..]).read_f32::<LittleEndian>()?;
            let z = (&data[(attribute.offset as usize + 8)..]).read_f32::<LittleEndian>()?;
            Ok(Vector3::new(x, y, z))
        } else {
            Err(VertexFetchError::NoSuchAttribute(usage))
        }
    }

    /// Tries to read an attribute with given usage as a pair of four f32.
    #[inline(always)]
    fn read_4_f32(&self, usage: VertexAttributeUsage) -> Result<Vector4<f32>, VertexFetchError> {
        let (data, layout) = self.data_layout_ref();
        if let Some(attribute) = layout.get(usage as usize).unwrap() {
            let x = (&data[(attribute.offset as usize)..]).read_f32::<LittleEndian>()?;
            let y = (&data[(attribute.offset as usize + 4)..]).read_f32::<LittleEndian>()?;
            let z = (&data[(attribute.offset as usize + 8)..]).read_f32::<LittleEndian>()?;
            let w = (&data[(attribute.offset as usize + 12)..]).read_f32::<LittleEndian>()?;
            Ok(Vector4::new(x, y, z, w))
        } else {
            Err(VertexFetchError::NoSuchAttribute(usage))
        }
    }

    /// Tries to read an attribute with given usage as a pair of four u8.
    #[inline(always)]
    fn read_4_u8(&self, usage: VertexAttributeUsage) -> Result<Vector4<u8>, VertexFetchError> {
        let (data, layout) = self.data_layout_ref();
        if let Some(attribute) = layout.get(usage as usize).unwrap() {
            let offset = attribute.offset as usize;
            let x = data[offset];
            let y = data[offset + 1];
            let z = data[offset + 2];
            let w = data[offset + 3];
            Ok(Vector4::new(x, y, z, w))
        } else {
            Err(VertexFetchError::NoSuchAttribute(usage))
        }
    }
}

impl<'a> VertexReadTrait for VertexViewRef<'a> {
    fn data_layout_ref(&self) -> (&[u8], &[Option<VertexAttribute>]) {
        (self.vertex_data, self.sparse_layout)
    }
}


/// See VertexBuffer::modify for more info.
pub struct VertexBufferRefMut<'a> {
    vertex_buffer: &'a mut VertexBuffer,
}

impl<'a> Drop for VertexBufferRefMut<'a> {
    fn drop(&mut self) {
        // Recalculate data hash.
        self.vertex_buffer.data_hash = calculate_data_hash(&self.vertex_buffer.data);
    }
}

impl<'a> Deref for VertexBufferRefMut<'a> {
    type Target = VertexBuffer;

    fn deref(&self) -> &Self::Target {
        self.vertex_buffer
    }
}

impl<'a> DerefMut for VertexBufferRefMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.vertex_buffer
    }
}

impl<'a> VertexBufferRefMut<'a> {
    /// Tries to append a vertex to the buffer.
    ///
    /// # Safety and validation
    ///
    /// This method accepts any type that has appropriate size, the size must be equal
    /// with the size defined by layout. The Copy trait bound is required to ensure that
    /// the type does not have any custom destructors.
    pub fn push_vertex<T: Copy>(&mut self, vertex: &T) -> Result<(), ValidationError> {
        if std::mem::size_of::<T>() == self.vertex_buffer.vertex_size_in_byte as usize {
            self.vertex_buffer
                .data
                .extend_from_slice(value_as_u8_slice(vertex));
            self.vertex_buffer.vertex_count += 1;
            Ok(())
        } else {
            Err(ValidationError::InvalidVertexSize {
                expected: self.vertex_buffer.vertex_size_in_byte,
                actual: std::mem::size_of::<T>() as u8,
            })
        }
    }

    /// Removes last vertex from the buffer.
    pub fn remove_last_vertex(&mut self) {
        self.vertex_buffer
            .data
            .drain((self.vertex_buffer.data.len() - self.vertex_buffer.vertex_size_in_byte as usize)..);
        self.vertex_buffer.vertex_count -= 1;
    }

    /// Copies data of last vertex from the buffer to an instance of variable of a type.
    ///
    /// # Safety and validation
    ///
    /// This method accepts any type that has appropriate size, the size must be equal
    /// with the size defined by layout. The Copy trait bound is required to ensure that
    /// the type does not have any custom destructors.
    pub fn pop_vertex<T: Copy>(&mut self) -> Result<T, ValidationError> {
        if std::mem::size_of::<T>() == self.vertex_buffer.vertex_size_in_byte as usize
            && self.vertex_buffer.data.len() >= self.vertex_buffer.vertex_size_in_byte as usize
        {
            unsafe {
                let mut v = MaybeUninit::<T>::uninit();
                std::ptr::copy_nonoverlapping(
                    self.vertex_buffer.data.as_ptr().add(
                        self.vertex_buffer.data.len() - self.vertex_buffer.vertex_size_in_byte as usize,
                    ),
                    v.as_mut_ptr() as *mut u8,
                    self.vertex_buffer.vertex_size_in_byte as usize,
                );
                self.vertex_buffer.data.drain(
                    (self.vertex_buffer.data.len() - self.vertex_buffer.vertex_size_in_byte as usize)..,
                );
                self.vertex_buffer.vertex_count -= 1;
                Ok(v.assume_init())
            }
        } else {
            Err(ValidationError::InvalidVertexSize {
                expected: self.vertex_buffer.vertex_size_in_byte,
                actual: std::mem::size_of::<T>() as u8,
            })
        }
    }

    /// Tries to cast internal data buffer to a slice of given type. It may fail if
    /// size of type is not equal with claimed size (which is set by the layout).
    pub fn cast_data_mut<T: Copy>(&mut self) -> Result<&mut [T], ValidationError> {
        if std::mem::size_of::<T>() == self.vertex_buffer.vertex_size_in_byte as usize {
            Ok(unsafe {
                std::slice::from_raw_parts_mut(
                    self.vertex_buffer.data.as_mut_ptr() as *const T as *mut T,
                    self.vertex_buffer.data.len() / std::mem::size_of::<T>(),
                )
            })
        } else {
            Err(ValidationError::InvalidVertexSize {
                expected: self.vertex_buffer.vertex_size_in_byte,
                actual: std::mem::size_of::<T>() as u8,
            })
        }
    }

    /// Creates iterator that emits read/write accessors for vertices.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = VertexViewMut<'_>> + '_ {
        unsafe {
            VertexViewMutIterator {
                ptr: self.vertex_buffer.data.as_mut_ptr(),
                end: self.data.as_mut_ptr().add(
                    self.vertex_buffer.vertex_size_in_byte as usize
                        * self.vertex_buffer.vertex_count as usize,
                ),
                vertex_size: self.vertex_buffer.vertex_size_in_byte,
                sparse_layout: &self.vertex_buffer.sparse_layout,
                marker: PhantomData,
            }
        }
    }

    /// Returns a read/write accessor of n-th vertex.
    pub fn get_mut(&mut self, n: usize) -> Option<VertexViewMut<'_>> {
        let offset = n * self.vertex_buffer.vertex_size_in_byte as usize;
        if offset < self.vertex_buffer.data.len() {
            Some(VertexViewMut {
                vertex_data: &mut self.vertex_buffer.data
                    [offset..(offset + self.vertex_buffer.vertex_size_in_byte as usize)],
                sparse_layout: &self.vertex_buffer.sparse_layout,
            })
        } else {
            None
        }
    }

    /// Duplicates n-th vertex and puts it at the back of the buffer.
    pub fn duplicate(&mut self, n: usize) {
        // Vertex cannot be larger than 256 bytes, so having temporary array of
        // such size is ok.
        let mut temp = ArrayVec::<u8, 256>::new();
        temp.try_extend_from_slice(
            &self.vertex_buffer.data[(n * self.vertex_buffer.vertex_size_in_byte as usize)
                ..((n + 1) * self.vertex_buffer.vertex_size_in_byte as usize)],
        )
        .unwrap();
        self.vertex_buffer.data.extend_from_slice(temp.as_slice());
        self.vertex_buffer.vertex_count += 1;
    }

    /// Adds new attribute at the end of layout, reorganizes internal data storage to be
    /// able to contain new attribute. Default value of the new attribute in the buffer
    /// becomes `fill_value`. Graphically this could be represented like so:
    ///
    /// Add secondary texture coordinates:
    ///  Before: P1_N1_TC1_P2_N2_TC2...
    ///  After: P1_N1_TC1_TC2(fill_value)_P2_N2_TC2_TC2(fill_value)...
    pub fn add_attribute<T: Copy>(
        &mut self,
        descriptor: VertexAttributeDescriptor,
        fill_value: T,
    ) -> Result<(), ValidationError> {
        if self.vertex_buffer.sparse_layout[descriptor.usage as usize].is_some() {
            Err(ValidationError::DuplicatedAttributeDescriptor)
        } else {
            let vertex_attribute = VertexAttribute {
                usage: descriptor.usage,
                data_type: descriptor.data_type,
                size: descriptor.size,
                divisor: descriptor.divisor,
                offset: self.vertex_buffer.vertex_size_in_byte,
                shader_location: descriptor.shader_location,
            };
            self.vertex_buffer.sparse_layout[descriptor.usage as usize] = Some(vertex_attribute);
            self.vertex_buffer.dense_layout.push(vertex_attribute);

            let mut new_data = Vec::new();

            for chunk in self
                .vertex_buffer
                .data
                .chunks_exact(self.vertex_buffer.vertex_size_in_byte as usize)
            {
                let mut temp = ArrayVec::<u8, 256>::new();
                temp.try_extend_from_slice(chunk).unwrap();
                temp.try_extend_from_slice(value_as_u8_slice(&fill_value))
                    .unwrap();
                new_data.extend_from_slice(&temp);
            }

            self.vertex_buffer.data = new_data;

            self.vertex_buffer.vertex_size_in_byte += std::mem::size_of::<T>() as u8;

            Ok(())
        }
    }

    /// Clears the buffer making it empty.
    pub fn clear(&mut self) {
        self.data.clear();
        self.vertex_count = 0;
    }
}


/// Read/write accessor for a vertex with some layout.
#[derive(Debug)]
pub struct VertexViewMut<'a> {
    vertex_data: &'a mut [u8],
    sparse_layout: &'a [Option<VertexAttribute>],
}

impl<'a> PartialEq for VertexViewMut<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_data == other.vertex_data
    }
}


impl<'a> VertexReadTrait for VertexViewMut<'a> {
    fn data_layout_ref(&self) -> (&[u8], &[Option<VertexAttribute>]) {
        (self.vertex_data, self.sparse_layout)
    }
}

impl<'a> VertexWriteTrait for VertexViewMut<'a> {
    fn data_layout_mut(&mut self) -> (&mut [u8], &[Option<VertexAttribute>]) {
        (self.vertex_data, self.sparse_layout)
    }

    fn write_2_f32(
        &mut self,
        usage: VertexAttributeUsage,
        value: Vector2<f32>,
    ) -> Result<(), VertexFetchError> {
        let (data, layout) = self.data_layout_mut();
        if let Some(attribute) = layout.get(usage as usize).unwrap() {
            (&mut data[(attribute.offset as usize)..]).write_f32::<LittleEndian>(value.x)?;
            (&mut data[(attribute.offset as usize + 4)..]).write_f32::<LittleEndian>(value.y)?;
            Ok(())
        } else {
            Err(VertexFetchError::NoSuchAttribute(usage))
        }
    }

    fn write_3_f32(
        &mut self,
        usage: VertexAttributeUsage,
        value: Vector3<f32>,
    ) -> Result<(), VertexFetchError> {
        let (data, layout) = self.data_layout_mut();
        if let Some(attribute) = layout.get(usage as usize).unwrap() {
            (&mut data[(attribute.offset as usize)..]).write_f32::<LittleEndian>(value.x)?;
            (&mut data[(attribute.offset as usize + 4)..]).write_f32::<LittleEndian>(value.y)?;
            (&mut data[(attribute.offset as usize + 8)..]).write_f32::<LittleEndian>(value.z)?;
            Ok(())
        } else {
            Err(VertexFetchError::NoSuchAttribute(usage))
        }
    }

    fn write_4_f32(
        &mut self,
        usage: VertexAttributeUsage,
        value: Vector4<f32>,
    ) -> Result<(), VertexFetchError> {
        let (data, layout) = self.data_layout_mut();
        if let Some(attribute) = layout.get(usage as usize).unwrap() {
            (&mut data[(attribute.offset as usize)..]).write_f32::<LittleEndian>(value.x)?;
            (&mut data[(attribute.offset as usize + 4)..]).write_f32::<LittleEndian>(value.y)?;
            (&mut data[(attribute.offset as usize + 8)..]).write_f32::<LittleEndian>(value.z)?;
            (&mut data[(attribute.offset as usize + 12)..]).write_f32::<LittleEndian>(value.w)?;
            Ok(())
        } else {
            Err(VertexFetchError::NoSuchAttribute(usage))
        }
    }

    fn write_4_u8(
        &mut self,
        usage: VertexAttributeUsage,
        value: Vector4<u8>,
    ) -> Result<(), VertexFetchError> {
        let (data, layout) = self.data_layout_mut();
        if let Some(attribute) = layout.get(usage as usize).unwrap() {
            data[attribute.offset as usize] = value.x;
            data[(attribute.offset + 1) as usize] = value.y;
            data[(attribute.offset + 2) as usize] = value.z;
            data[(attribute.offset + 3) as usize] = value.w;
            Ok(())
        } else {
            Err(VertexFetchError::NoSuchAttribute(usage))
        }
    }
}


struct VertexViewMutIterator<'a> {
    ptr: *mut u8,
    sparse_layout: &'a [Option<VertexAttribute>],
    end: *mut u8,
    vertex_size: u8,
    marker: PhantomData<&'a mut u8>,
}

impl<'a> Iterator for VertexViewMutIterator<'a> {
    type Item = VertexViewMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.end {
            None
        } else {
            unsafe {
                let data = std::slice::from_raw_parts_mut(self.ptr, self.vertex_size as usize);
                let view = VertexViewMut {
                    vertex_data: data,
                    sparse_layout: self.sparse_layout,
                };
                self.ptr = self.ptr.add(self.vertex_size as usize);
                Some(view)
            }
        }
    }
}


/// A trait for read/write vertex data accessor.
pub trait VertexWriteTrait: VertexReadTrait {
    #[doc(hidden)]
    fn data_layout_mut(&mut self) -> (&mut [u8], &[Option<VertexAttribute>]);

    /// Tries to write an attribute with given usage as a pair of two f32.
    fn write_2_f32(
        &mut self,
        usage: VertexAttributeUsage,
        value: Vector2<f32>,
    ) -> Result<(), VertexFetchError>;

    /// Tries to write an attribute with given usage as a pair of three f32.
    fn write_3_f32(
        &mut self,
        usage: VertexAttributeUsage,
        value: Vector3<f32>,
    ) -> Result<(), VertexFetchError>;

    /// Tries to write an attribute with given usage as a pair of four f32.
    fn write_4_f32(
        &mut self,
        usage: VertexAttributeUsage,
        value: Vector4<f32>,
    ) -> Result<(), VertexFetchError>;

    /// Tries to write an attribute with given usage as a pair of four u8.
    fn write_4_u8(
        &mut self,
        usage: VertexAttributeUsage,
        value: Vector4<u8>,
    ) -> Result<(), VertexFetchError>;
}
