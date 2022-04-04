use crate::vertex_buffer::{VertexBuffer,VertexAttributeDataType};
pub struct NativeBufferBuilder {
    element_size: usize,
    kind: GeometryBufferKind,
    attributes: Vec<AttributeDefinition>,
    data: *const u8,
    data_size: usize,
}

impl NativeBufferBuilder{
    pub fn from_vertex_buffer(buffer: &VertexBuffer, kind: GeometryBufferKind) -> Self {
        Self {
            element_size: buffer.vertex_size_in_byte() as usize,
            kind,
            attributes: buffer
                .layout()
                .iter()
                .map(|a| AttributeDefinition {
                    location: a.shader_location as u32,
                    kind: match (a.data_type, a.size) {
                        (VertexAttributeDataType::F32, 1) => AttributeKind::Float,
                        (VertexAttributeDataType::F32, 2) => AttributeKind::Float2,
                        (VertexAttributeDataType::F32, 3) => AttributeKind::Float3,
                        (VertexAttributeDataType::F32, 4) => AttributeKind::Float4,
                        (VertexAttributeDataType::U32, 1) => AttributeKind::UnsignedInt,
                        (VertexAttributeDataType::U32, 2) => AttributeKind::UnsignedInt2,
                        (VertexAttributeDataType::U32, 3) => AttributeKind::UnsignedInt3,
                        (VertexAttributeDataType::U32, 4) => AttributeKind::UnsignedInt4,
                        (VertexAttributeDataType::U16, 1) => AttributeKind::UnsignedShort,
                        (VertexAttributeDataType::U16, 2) => AttributeKind::UnsignedShort2,
                        (VertexAttributeDataType::U16, 3) => AttributeKind::UnsignedShort3,
                        (VertexAttributeDataType::U16, 4) => AttributeKind::UnsignedShort4,
                        (VertexAttributeDataType::U8, 1) => AttributeKind::UnsignedByte,
                        (VertexAttributeDataType::U8, 2) => AttributeKind::UnsignedByte2,
                        (VertexAttributeDataType::U8, 3) => AttributeKind::UnsignedByte3,
                        (VertexAttributeDataType::U8, 4) => AttributeKind::UnsignedByte4,
                        _ => unreachable!(),
                    },
                    normalized: false,
                    divisor: 0,
                })
                .collect(),
            data: buffer.raw_data().as_ptr(),
            data_size: buffer.raw_data().len(),
        }
    }
}

pub struct AttributeDefinition {
    pub location: u32,
    pub kind: AttributeKind,
    pub normalized: bool,
    pub divisor: u32,
}


pub enum AttributeKind {
    Float,
    Float2,
    Float3,
    Float4,

    UnsignedByte,
    UnsignedByte2,
    UnsignedByte3,
    UnsignedByte4,

    UnsignedShort,
    UnsignedShort2,
    UnsignedShort3,
    UnsignedShort4,

    UnsignedInt,
    UnsignedInt2,
    UnsignedInt3,
    UnsignedInt4,
}


#[derive(Copy, Clone)]
#[repr(u32)]
pub enum GeometryBufferKind {
    StaticDraw = glow::STATIC_DRAW,
    DynamicDraw = glow::DYNAMIC_DRAW,
}
