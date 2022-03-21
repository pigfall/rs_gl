pub mod gl_45;

use rustsdk::string;

#[derive(Debug)]
pub enum ErrGL{
    GL_INVALID_ENUM,
    GL_INVALID_VALUE,
}

pub enum DrawArrayMode{
    GL_TRIANGLES,
}


#[derive(Debug, PartialEq,Clone,Copy)]
pub enum ClearColorTarget{
    COLOR_BUFFER_BIT,
    DEPTH_BUFFER_BIT,
}

pub enum ShaderIvType{
    TYPE ,
    DELETE_STATUS ,
    COMPILE_STATUS ,
    INFO_LOG_LENGTH,
    SHADER_SOURCE_LENGTH ,
}

pub enum ShaderType{
    VERTEX_SHADER ,
    FRAGMENT_SHADER, 
}

pub trait GLFns{
    fn draw_array(&self,mode: DrawArrayMode,starting_index: i32,count: i32)->Result<(),ErrGL>;

    fn get_error(&self)->Result<(),ErrGL>;

    fn clear_color(&self,r: f32,g:f32,b:f32,a:f32)->Result<(),ErrGL>;

    fn clear(&self,targets: &[ClearColorTarget])->Result<(),ErrGL>;

    fn max_texture_size()->u32;

    fn raw_compile_shader(&self,shader_id: u32);

    fn get_shader_iv(&self,shader_id: u32,iv_type: ShaderIvType)->i32;


    fn get_shader_iv_log_len(&self,shader_id: u32)-> u32;

    fn get_shader_iv_log(&self ,shader_id: u32)->String{
        let mut log_len = self.get_shader_iv_log_len(shader_id) as _ ;
        if log_len == 0{
            return String::from("");
        }
        let log_str = string::with_capacity_and_fill_in(log_len as _,'\0');

        self.raw_get_shader_iv_log(shader_id,log_len,&mut log_len,(&log_str[..]).as_ptr() as _);
        return log_str;
    }

    fn raw_get_shader_iv_log(&self,shader_id: u32,buf_size: isize,length:* mut isize,log_info : *mut i8);

    fn compile_shader(&self,shader_id: u32)->Result<(),String>{
        self.raw_compile_shader(shader_id);
        if self.get_shader_iv(shader_id,ShaderIvType::COMPILE_STATUS) != 1{
            return Err(self.get_shader_iv_log(shader_id));
        }
        return Ok(())
    }
    fn create_shader(&self,shader_type: ShaderType)->Result<u32,ErrGL>;
}
