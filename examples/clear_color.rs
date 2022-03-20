use winit::window::{WindowBuilder};
use winit::event_loop::{EventLoop};
use glutin::ContextBuilder;
use glutin::platform::windows::RawContextExt;
use winit::platform::windows::WindowExtWindows;

extern crate rs_gl;

use rs_gl::gl_45::gl::GLFnsWrapper;
use rs_gl::{ClearColorTarget,GLFns};

fn main(){
    let ev_lp = EventLoop::new();
    let win = WindowBuilder::new().build(&ev_lp).unwrap();

    let gl_ctx = unsafe{
        ContextBuilder::new().build_raw_context(win.hwnd()).unwrap().make_current().unwrap()
    };

    let gl = GLFnsWrapper::new(|name|gl_ctx.get_proc_address(name));


    ev_lp.run(move |_,_,_,|{
        gl.clear_color(0.1,0.2,0.3,1.0).unwrap();
        gl.clear(&[ClearColorTarget::COLOR_BUFFER_BIT]).unwrap();
        gl_ctx.swap_buffers().unwrap();
    })
}
