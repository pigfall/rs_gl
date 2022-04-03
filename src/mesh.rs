use crate::vector::Vector3;
use crate::gl::Gl;
use crate::types::{TargetBindBuffer,BufferDataUsage,VertexComponentDataType,VertexAttribPointerShouleBeNormalized};
use nalgebra_glm as glm;

pub struct Mesh {
    pub position: Vec<glm::Vec3>,
    pub vbo: glow::Buffer,
    pub vao: glow::VertexArray,
}

impl Mesh{
    pub fn new(gl: &Gl,position: Vec<glm::Vec3>)->Self{
        let vbo = gl.gen_buffer().unwrap();
        let target = TargetBindBuffer::array_buffer();
        let vao = gl.gen_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(target,Some(vbo));
        gl.buffer_data_u8_slice(
            target,
            &(position.iter().flat_map(|v|v.iter().flat_map(|e|e.to_le_bytes())).collect::<Vec<u8>>())[..],
            BufferDataUsage::StaticDraw(),);
        gl.vertex_attrib_pointer(
            0,
            3,
            VertexComponentDataType::float32(),
             VertexAttribPointerShouleBeNormalized::false_value(),
             0,
             0,
            );
        gl.enable_vertex_attrib_array(0);
        gl.bind_vertex_array(None);
        gl.bind_buffer(target,None);
        Self{
            position:position,
            vbo:vbo,
            vao:vao,
        }
    }
}
