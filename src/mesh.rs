use crate::vector::Vector3;
use crate::gl::Gl;
use crate::types::{TargetBindBuffer,BufferDataUsage,VertexComponentDataType,VertexAttribPointerShouleBeNormalized};
use nalgebra_glm as glm;

#[repr(C)]
pub struct Vector{
    pub position: glm::Vec3,
    pub text_coord: glm::Vec3,
}

pub struct Mesh {
    pub vertices: Vec<Vector>,
    pub vbo: glow::Buffer,
    pub vao: glow::VertexArray,
}

impl Mesh{
    pub fn new(gl: &Gl,vertices: Vec<Vector>)->Self{
        let vbo = gl.gen_buffer().unwrap();
        let target = TargetBindBuffer::array_buffer();
        let vao = gl.gen_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(target,Some(vbo));
        let data_ptr = unsafe{
            let bytes = vertices.as_ptr() as *const u8;
            std::slice::from_raw_parts(bytes,vertices.len() * std::mem::size_of::<Vector>())
        };
        gl.buffer_data_u8_slice(
            target,
            data_ptr,
            BufferDataUsage::StaticDraw(),);
        gl.vertex_attrib_pointer(
            0,
            3,
            VertexComponentDataType::float32(),
             VertexAttribPointerShouleBeNormalized::false_value(),
             std::mem::size_of::<Vector>() as _,
             0,
            );
        gl.enable_vertex_attrib_array(0);
        gl.bind_vertex_array(None);
        gl.bind_buffer(target,None);
        Self{
            vertices:vertices,
            vbo:vbo,
            vao:vao,
        }
    }
}
