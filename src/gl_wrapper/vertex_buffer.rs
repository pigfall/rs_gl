pub use glow::HasContext;

#[derive(Copy,Clone)]
pub struct VertexAttributeDescriptor{
    pub shader_location:u32,
    pub num_of_demision:i32,
    pub data_type: VertexAttributeDataType,
    pub normalized: bool,
}

impl VertexAttributeDescriptor{
    pub fn size_in_byte(&self)->u32{
        return (self.num_of_demision as u32) * self.data_type.size_in_byte();
    }
}

#[derive(Copy,Clone)]
#[repr(u32)]
pub enum VertexAttributeDataType{
    F32 =glow::FLOAT,
}

impl VertexAttributeDataType{
    pub fn size_in_byte(&self)->u32{
        match self{
        &VertexAttributeDataType::F32=>{
            return std::mem::size_of::<f32>() as u32;
        }
        }
    }
}

pub struct VertexBuffer {
    layout_of_attrib_vertex_one_pass: Vec<VertexAttributeDescriptor>,
    data_of_list_of_attrib_vertex_one_pass: Vec<u8>,
}


impl VertexBuffer{
    pub fn from_bytes(bytes:Vec<u8>,layout:Vec<VertexAttributeDescriptor>)->Self{
        return Self{
            data_of_list_of_attrib_vertex_one_pass:bytes,
            layout_of_attrib_vertex_one_pass:layout,
        }
    }
    pub fn one_pass_size_in_byte(&self)->u32{
        let mut size = 0u32;
        for attrib in &self.layout_of_attrib_vertex_one_pass{
            size += attrib.size_in_byte();
        }
        return size;
    }
}

pub struct NativeBuffer{
    vbo: glow::Buffer,
}

pub struct NativeBufferBuilder<'a>{
    vertex_buffer: &'a VertexBuffer,
    buffer_data_usage:NativeBufferBufferDataUsage,
}

#[derive(Clone,Copy)]
#[repr(u32)]
pub enum NativeBufferBufferDataUsage{
    STATIC_DRAW  = glow::STATIC_DRAW,
}


impl  <'a> NativeBufferBuilder<'a> {
    pub fn from_vertex_buffer(vertex_buffer:&'a VertexBuffer,buffer_data_usage: NativeBufferBufferDataUsage)->Self{
        return Self{
            vertex_buffer:vertex_buffer,
            buffer_data_usage:buffer_data_usage,
        }
    }

    pub fn build(&self, state: &mut PipelineState)->NativeBuffer{
        unsafe{
            let vbo = state.gl.create_buffer().unwrap();
            state.gl.bind_buffer(glow::ARRAY_BUFFER,Some(vbo));

            state.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER,&(self.vertex_buffer.data_of_list_of_attrib_vertex_one_pass)[..],self.buffer_data_usage as _);

            let mut offset = 0u32;
            for attrib_vertex_desc in &self.vertex_buffer.layout_of_attrib_vertex_one_pass{
                // how to read
                state.gl.vertex_attrib_pointer_f32(
                    attrib_vertex_desc.shader_location,
                    attrib_vertex_desc.num_of_demision,
                    attrib_vertex_desc.data_type as _,
                    attrib_vertex_desc.normalized,
                    self.vertex_buffer.one_pass_size_in_byte().try_into().unwrap(),
                    offset.try_into().unwrap(),
                    );
                state.gl.enable_vertex_attrib_array(attrib_vertex_desc.shader_location);
                offset += attrib_vertex_desc.size_in_byte();
            }
            state.gl.bind_buffer(glow::ARRAY_BUFFER,None);
            return NativeBuffer{
                vbo:vbo,
            };
        }
    }

}


pub struct PipelineState{
    pub gl: glow::Context,
}
