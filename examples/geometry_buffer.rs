extern crate rs_gl;
use rs_gl::Gl;
use rs_gl::glow;
use rs_gl::surface_data::SurfaceData;
use rs_gl::geometry_buffer::GeometryBuffer;
use rs_gl::native_buffer::GeometryBufferKind;
use rs_gl::types::*;
use rs_gl::pipeline_state::{PipelineState};
use rs_gl::glow::HasContext;
use glutin::platform::windows::RawContextExt;
use winit::window::{WindowBuilder};
use glutin::ContextBuilder;
use glutin::platform::windows::WindowExtWindows;
use winit::event_loop::EventLoop;

use rs_gl::{
    core::{
        algebra::{Vector2, Vector3, Vector4,Matrix4},
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
    let gl =unsafe{
        glow::Context::from_loader_function(|fn_name|ctx.get_proc_address(fn_name))

    };

    let mut pipeline_state = PipelineState::new(gl);
    

    let surface_data = SurfaceData::make_cube(
        Matrix4::new_nonuniform_scaling(&Vector3::new(
                25.0f32, 0.25, 25.0,
                ))
        );
    let geometry_buffer = GeometryBuffer::from_surface_data(&surface_data, GeometryBufferKind::StaticDraw,&mut pipeline_state,);

    ev.run(move|_,_,_|{
        geometry_buffer.bind(&mut pipeline_state).draw();
        ctx.swap_buffers().unwrap();
    })

}

