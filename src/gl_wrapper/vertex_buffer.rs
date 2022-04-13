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

    pub fn unit_triangle()->Self{
        let pos_data = [
            0.0f32,0.5,0.0,
            -0.5,-0.5,0.0,
            0.5,-0.5,0.0,
        ];
        return Self{
            data_of_list_of_attrib_vertex_one_pass:pos_data.iter().flat_map(|e|e.to_le_bytes()).collect::<Vec<u8>>(),
            layout_of_attrib_vertex_one_pass:vec![VertexAttributeDescriptor{
                shader_location:0,
                num_of_demision:3,
                data_type: VertexAttributeDataType::F32,
                normalized:false,
            }]
        }
    }

    pub fn unit_rectangle()->Self{
        let pos_data = [
            -0.5f32,0.5,0.0,
            0.5,0.5,0.0,
            -0.5,-0.5,0.0,

            -0.5,-0.5,0.0,
            0.5,-0.5,0.0,
            0.5,0.5,0.0,
        ];
        return Self{
            data_of_list_of_attrib_vertex_one_pass:pos_data.iter().flat_map(|e|e.to_le_bytes()).collect::<Vec<u8>>(),
            layout_of_attrib_vertex_one_pass:vec![VertexAttributeDescriptor{
                shader_location:0,
                num_of_demision:3,
                data_type: VertexAttributeDataType::F32,
                normalized:false,
            }]
        }
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

pub struct GeometryBuffer{
    vao:glow::VertexArray,
    draw_count: i32,
}

impl GeometryBuffer{
    pub fn from_native_buffer(buffer_builder:NativeBufferBuilder,state:&mut PipelineState,draw_count:i32)->GeometryBuffer{
        return GeometryBufferBuilder::with_native_buffer(buffer_builder).build(state,draw_count);
    }

    pub fn bind(&self,state:&mut PipelineState)->GeometryBufferBinding{
        unsafe{
            state.gl.bind_vertex_array(Some(self.vao));
            return GeometryBufferBinding{
                buffer:self,
            }
        }
    }

    pub fn unit_triangle(state:&mut PipelineState)->GeometryBuffer{
        return Self::from_native_buffer(
            NativeBufferBuilder::from_vertex_buffer(&VertexBuffer::unit_triangle(), NativeBufferBufferDataUsage::STATIC_DRAW),
            state,
            3,
            )
    }

    pub fn unit_rectangle(state:&mut PipelineState)->GeometryBuffer{
        return Self::from_native_buffer(
            NativeBufferBuilder::from_vertex_buffer(&VertexBuffer::unit_rectangle(), NativeBufferBufferDataUsage::STATIC_DRAW),
            state,
            6,
            )
    }
}

pub struct GeometryBufferBuilder<'a>{
    buffer_builder: NativeBufferBuilder<'a>,
}

impl <'a>GeometryBufferBuilder<'a>{
    pub fn with_native_buffer(buffer: NativeBufferBuilder<'a>)->Self{
        return Self{
            buffer_builder:buffer,
        }
    }

    pub fn build(&self,state:&mut PipelineState,draw_count:i32)->GeometryBuffer{
        unsafe{
            let vao = state.gl.create_vertex_array().unwrap();
            state.gl.bind_vertex_array(Some(vao));
            self.buffer_builder.build(state);

            return GeometryBuffer{
                vao:vao,
                draw_count:draw_count,
            }
        }
    }

}


pub struct GeometryBufferBinding<'a>{
    buffer: &'a GeometryBuffer,
}

impl <'a>GeometryBufferBinding<'a>{
    pub fn draw(&self,state:&mut PipelineState){
        unsafe{
            state.gl.draw_arrays(glow::TRIANGLES,0,self.buffer.draw_count)
        }
    }

}

