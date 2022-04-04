pub use glow;
pub use glow::HasContext;
use glow::Shader;
use std::ffi::CStr;
use std::ops::Deref;
use std::os::raw::c_void;
use crate::types::{
    ShaderType,VertexComponentDataType, VertexAttribPointerShouleBeNormalized,TargetBindBuffer,BufferDataUsage,GlErr,DrawArrayMode};
use crate::shader_program::ShaderProgram;
use crate::mesh::Mesh;
pub struct Gl{
    raw:glow::Context,
}


impl Gl{
    pub fn from_load_fn<F>(f:F)->Self
        where 
        F: FnMut(&str)->*const std::os::raw::c_void,
        {
            let raw = unsafe{
                glow::Context::from_loader_function(f)
            };
            return Gl{
                raw:raw,
            };
        }

    pub fn get_version(&self)->String{
         unsafe{
            let c_str = self.raw.raw.GetString(glow::VERSION);
            CStr::from_ptr(c_str as _).to_str().unwrap().to_string()
        }
    }

    pub fn create_shader(&self,shader_type: ShaderType)->Result<glow::Shader,String>{
        unsafe{
            return self.raw.create_shader(shader_type.0);
        }
    }

    pub fn create_shader_and_compile(&self,shader_type:ShaderType,shader_src:&str)->Result<glow::Shader,String>{
        let shader = self.create_shader(shader_type)?;
        unsafe{
            self.raw.shader_source(shader,shader_src);
            self.raw.compile_shader(shader);
            let compile_suc = self.raw.get_shader_compile_status(shader);
            if !compile_suc{
                return Err(self.get_shader_info_log(shader))
            }
            return Ok(shader)
        };
    }

    pub fn make_program(&self,vertex_shader_src: &str, frag_shader_src: &str)->Result<ShaderProgram,String>{
        let gl = &self.raw;
        unsafe{
            let pg = gl.create_program()?;
            let vertex_shader = self.create_shader_and_compile(ShaderType::VertexShader(),vertex_shader_src)?;
            self.raw.attach_shader(pg,vertex_shader);
            let frag_shader = self.create_shader_and_compile(ShaderType::FragShader(),frag_shader_src)?;
            self.raw.attach_shader(pg,frag_shader);

            self.raw.link_program(pg);
            let link_pg_ok = self.raw.get_program_link_status(pg);
            if !link_pg_ok{
                return Err(self.raw.get_program_info_log(pg));
            }
            return Ok(ShaderProgram::new(pg,vertex_shader,frag_shader));
        };
    }

    pub fn use_program(&self,pg: Option<glow::Program>){
        unsafe{
            self.raw.use_program(pg)
        }

    }

    pub fn vertex_attrib_pointer(
        &self,
        attrib_location: u32,
        num_of_vertex_component: i32,
        vertex_component_data_type:VertexComponentDataType,
        should_normalize: VertexAttribPointerShouleBeNormalized,
        stride_in_byte:i32,
        offset_in_byte:i32,
        ){
        unsafe{
            self.raw.raw.VertexAttribPointer(
                attrib_location,
                num_of_vertex_component,
                vertex_component_data_type.0,
                should_normalize.0,
                stride_in_byte,
                offset_in_byte as *const c_void,
                                            )
        }
    }

    pub fn get_attrib_location(&self,pg: glow::Program,name: &str)->Option<u32>{
        unsafe{
            self.raw.get_attrib_location(pg,name)
        }
    }

    pub fn enable_vertex_attrib_array(&self,attrib_location: u32){
        unsafe{
            self.raw.enable_vertex_attrib_array(attrib_location)
        }
    }

    pub fn gen_buffer(&self)->Result<glow::Buffer,String>{
        unsafe{
            self.raw.create_buffer()
        }
    }
    pub fn gen_vertex_array(&self)->Result<glow::VertexArray,String>{
        unsafe{
            return self.raw.create_vertex_array();
        }
    }

    pub fn bind_vertex_array(&self,buffer: Option<glow::VertexArray>){
        unsafe{
            return self.raw.bind_vertex_array(buffer)
        }
    }

    pub fn bind_buffer(&self,target: TargetBindBuffer,buffer :Option<glow::Buffer>){
        unsafe{
            self.raw.bind_buffer(target.0,buffer)
        }
    }

    pub fn buffer_data_u8_slice(&self,target: TargetBindBuffer,data: &[u8],usage: BufferDataUsage){
        unsafe{
            self.raw.buffer_data_u8_slice(target.0,data,usage.0)
        }
    }

    pub fn get_error(&self)->Result<(),GlErr>{
        unsafe{
            let err = self.raw.get_error();
            match err{
                0 =>{return Ok(())},
                _ => {panic!("{:?}",format!("undefined gl error {:?}",err))}
            }
        }

    }

    pub fn draw_arrays(&self,mode: DrawArrayMode,first:  i32,number_of_vertex_to_draw: i32){
        unsafe{
            self.raw.draw_arrays(mode.0,first,number_of_vertex_to_draw);
        };
    }

    pub fn draw_mesh(&self, meshes: &[Mesh]){
        let target = TargetBindBuffer::array_buffer();
        for mesh in meshes{
            self.bind_vertex_array(Some(mesh.vao));
            self.draw_arrays(DrawArrayMode::triangle(),0,mesh.vertices.len().try_into().unwrap())
        }
    }
}

impl Deref for Gl{
    type Target = glow::Context;
    fn deref(&self)->&glow::Context{
        return &self.raw;
    }
}
