pub mod gl_45;

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

pub trait GLFns{
    fn draw_array(&self,mode: DrawArrayMode,starting_index: i32,count: i32)->Result<(),ErrGL>;

    fn get_error(&self)->Result<(),ErrGL>;

    fn clear_color(&self,r: f32,g:f32,b:f32,a:f32)->Result<(),ErrGL>;

    fn clear(&self,targets: &[ClearColorTarget])->Result<(),ErrGL>;
}
