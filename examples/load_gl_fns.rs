extern crate rs_gl;
use rs_gl::Gl;
use glutin::platform::windows::RawContextExt;
use winit::window::{WindowBuilder};
use glutin::ContextBuilder;
use glutin::platform::windows::WindowExtWindows;
use winit::event_loop::EventLoop;

fn main(){
    let ev = EventLoop::new();
    let win = WindowBuilder::new().build(&ev).unwrap();

    let ctx = unsafe{
         ContextBuilder::new().build_raw_context(win.hwnd()).unwrap().make_current().unwrap()
    };
    let gl = Gl::from_load_fn(|fn_name|ctx.get_proc_address(fn_name));

    println!("gl version {:?}",gl.get_version());
}
