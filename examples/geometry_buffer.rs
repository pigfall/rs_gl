extern crate rs_gl;
use rs_gl::Gl;
use rs_gl::glow;
use rs_gl::types::*;
use rs_gl::glow::HasContext;
use glutin::platform::windows::RawContextExt;
use winit::window::{WindowBuilder};
use glutin::ContextBuilder;
use glutin::platform::windows::WindowExtWindows;
use winit::event_loop::EventLoop;

use rs_gl::{
    core::{
        algebra::{Vector2, Vector3, Vector4},
        arrayvec::ArrayVec,
        byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt},
        futures::io::Error,
        visitor::prelude::*,
    },
};

fn main(){
    let ev = EventLoop::new();
    let win = WindowBuilder::new().build(&ev).unwrap();

    let ctx = unsafe{
        ContextBuilder::new().build_raw_context(win.hwnd()).unwrap().make_current().unwrap()
    };
    let gl = Gl::from_load_fn(|fn_name|ctx.get_proc_address(fn_name));

    let surface_data = SurfaceData::make_cube();
    let geometry_buffer = GeometryBuffer::from_surface_data(&pipelie_state,&surface_data);

    draw(pipelie_state,geometry_buffer);

}

