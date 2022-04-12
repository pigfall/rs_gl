use serde::{Deserialize, Serialize};
use crate::geometry_buffer::DrawParameters ;
use std::{
    borrow::Cow,
    io::Cursor,
    path::{Path, PathBuf},
};
use crate::{
    core::{
        algebra::{Vector2, Vector3, Vector4,Matrix2,Matrix3,Matrix4},
        arrayvec::ArrayVec,
        byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt},
        futures::io::Error,
        visitor::prelude::*,
        io::{self, FileLoadError},
    },
};

/// A definition of the shader.
#[derive(Default, Deserialize, Debug, PartialEq)]
pub struct ShaderDefinition {
    /// A name of the shader.
    pub name: String,
    /// A set of render passes.
    pub passes: Vec<RenderPassDefinition>,
    /// A set of property definitions.
    pub properties: Vec<PropertyDefinition>,
}

impl ShaderDefinition {
    pub fn from_buf(buf: Vec<u8>) -> Result<Self, ShaderError> {
        Ok(ron::de::from_reader(Cursor::new(buf))?)
    }

    pub fn from_str(str: &str) -> Result<Self, ShaderError> {
        Ok(ron::de::from_str(str)?)
    }
}


/// A render pass definition. See [`Shader`] docs for more info about render passes.
#[derive(Default, Deserialize, Debug, PartialEq)]
pub struct RenderPassDefinition {
    /// A name of render pass.
    pub name: String,
    /// A set of parameters that will be used in a render pass.
    pub draw_parameters: DrawParameters,
    /// A source code of vertex shader.
    pub vertex_shader: String,
    /// A source code of fragment shader.
    pub fragment_shader: String,
}



/// Shader property definition.
#[derive(Default, Deserialize, Debug, PartialEq)]
pub struct PropertyDefinition {
    /// A name of the property.
    pub name: String,
    /// A kind of property with default value.
    pub kind: PropertyKind,
}


/// Shader property with default value.
#[derive(Deserialize, Debug, PartialEq)]
pub enum PropertyKind {
    /// Real number.
    Float(f32),

    /// Real number array.
    FloatArray(Vec<f32>),

    /// Integer number.
    Int(i32),

    /// Integer number array.
    IntArray(Vec<i32>),

    /// Natural number.
    UInt(u32),

    /// Natural number array.
    UIntArray(Vec<u32>),

    /// Boolean value.
    Bool(bool),

    /// Two-dimensional vector.
    Vector2(Vector2<f32>),

    /// Two-dimensional vector array.
    Vector2Array(Vec<Vector2<f32>>),

    /// Three-dimensional vector.
    Vector3(Vector3<f32>),

    /// Three-dimensional vector array.
    Vector3Array(Vec<Vector3<f32>>),

    /// Four-dimensional vector.
    Vector4(Vector4<f32>),

    /// Four-dimensional vector array.
    Vector4Array(Vec<Vector4<f32>>),

    /// 2x2 Matrix.
    Matrix2(Matrix2<f32>),

    /// 2x2 Matrix array.
    Matrix2Array(Vec<Matrix2<f32>>),

    /// 3x3 Matrix.
    Matrix3(Matrix3<f32>),

    /// 3x3 Matrix array.
    Matrix3Array(Vec<Matrix3<f32>>),

    /// 4x4 Matrix.
    Matrix4(Matrix4<f32>),

    /// 4x4 Matrix array.
    Matrix4Array(Vec<Matrix4<f32>>),

    /// An sRGB color.
    ///
    /// # Conversion
    ///
    /// The colors you see on your monitor are in sRGB color space, this is fine for simple cases
    /// of rendering, but not for complex things like lighting. Such things require color to be
    /// linear. Value of this variant will be automatically **converted to linear color space**
    /// before it passed to shader.
    Color {
        /// Default Red.
        r: u8,

        /// Default Green.
        g: u8,

        /// Default Blue.
        b: u8,

        /// Default Alpha.
        a: u8,
    },

    /// A texture.
    Sampler {
        /// Optional path to default texture.
        default: Option<PathBuf>,

        /// Default fallback value. See [`SamplerFallback`] for more info.
        fallback: SamplerFallback,
    },
}


impl Default for PropertyKind {
    fn default() -> Self {
        Self::Float(0.0)
    }
}


/// A set of possible error variants that can occur during shader loading.
#[derive(Debug, thiserror::Error)]
pub enum ShaderError {
    /// An i/o error has occurred.
    #[error("A file load error has occurred {0:?}")]
    Io(FileLoadError),

    /// A parsing error has occurred.
    #[error("A parsing error has occurred {0:?}")]
    ParseError(ron::Error),
}

impl From<ron::Error> for ShaderError {
    fn from(e: ron::Error) -> Self {
        Self::ParseError(e)
    }
}

impl From<FileLoadError> for ShaderError {
    fn from(e: FileLoadError) -> Self {
        Self::Io(e)
    }
}


#[derive(Deserialize, Debug, PartialEq, Clone, Copy, Visit)]
pub enum SamplerFallback {
    /// A 1x1px white texture.
    White,
    /// A 1x1px texture with (0, 1, 0) vector.
    Normal,
    /// A 1x1px black texture.
    Black,
}

impl Default for SamplerFallback {
    fn default() -> Self {
        Self::White
    }
}
