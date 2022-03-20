use crate::*;
pub use std::os::raw;

mod gl_gen{
    include!(concat!(env!("OUT_DIR"),"/gl_bindings.rs"));
}

pub struct GLFnsWrapper{
    raw:gl_gen::Gl,
}

impl GLFnsWrapper{
    pub fn new<F>(f: F)->GLFnsWrapper
    where
        F:FnMut(&'static str) -> *const raw::c_void
    {
        let raw =gl_gen::Gl::load_with(f);
        return GLFnsWrapper{
            raw:raw,
        };
    }

}


impl GLFns for GLFnsWrapper{
    fn get_error(&self)->Result<(),ErrGL>{
        unsafe{
            let gl = &self.raw;
            let raw_err  = gl.GetError();
            match raw_err {
                gl_gen::INVALID_ENUM =>Err(ErrGL::GL_INVALID_ENUM),
                gl_gen::INVALID_VALUE => Err(ErrGL::GL_INVALID_VALUE),
                0=>Ok(()),
                _=>panic!("error code {:?}",raw_err),
            }
        }
    }
    fn draw_array(&self,mode: DrawArrayMode,starting_index: i32,count: i32)->Result<(),ErrGL>{
        self.get_error().unwrap();

        let gl = &self.raw;

        unsafe{
            gl.DrawArrays(mode as _ ,starting_index,count);
        };

        return self.get_error();
    }

    fn clear_color(&self,r: f32,g:f32,b:f32,a:f32)->Result<(),ErrGL>{
        let gl = &self.raw;
        unsafe{
            gl.ClearColor(r,g,b,a);
        };
        return self.get_error();
    }

    fn clear(&self,targets: &[ClearColorTarget])->Result<(),ErrGL>{
        let gl = &self.raw;
        let target = targets.iter().map(|e|{
            match e{
                &ClearColorTarget::COLOR_BUFFER_BIT => gl_gen::COLOR_BUFFER_BIT,
                &ClearColorTarget::DEPTH_BUFFER_BIT=> gl_gen::DEPTH_BUFFER_BIT,
            }
        }).fold(0,|acc,x| acc|x );
        unsafe{
            gl.Clear(target);
        };
        return self.get_error();

    }
}

