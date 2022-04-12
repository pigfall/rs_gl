extern crate rs_gl;
use rs_gl::Gl;
use rs_gl::core::color::Color;
use rs_gl::glow;
use rs_gl::surface_data::SurfaceData;
use rs_gl::geometry_buffer::GeometryBuffer;
use rs_gl::native_buffer::GeometryBufferKind;
use rs_gl::types::*;
use rs_gl::pipeline_state::{PipelineState};
use rs_gl::glow::HasContext;
use rs_gl::GpuProgram;
use glutin::platform::windows::RawContextExt;
use winit::window::{WindowBuilder};
use glutin::ContextBuilder;
use glutin::platform::windows::WindowExtWindows;
use winit::event_loop::EventLoop;

use rs_gl::{
    shader::ShaderDefinition,
    core::{
        algebra::{Vector2, Vector3, Vector4,Matrix4},
        arrayvec::ArrayVec,
        byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt},
        futures::io::Error,
        visitor::prelude::*,
    },
};

fn main(){
    let shader_def = ShaderDefinition::from_str(fyrox::material::shader::STANDARD_SHADER_SRC).unwrap();
    let render_pass = shader_def.passes.iter().find(|&e|e.name == "GBuffer").unwrap();

    
    let ev = EventLoop::new();
    let win = WindowBuilder::new().build(&ev).unwrap();

    let ctx = unsafe{
        ContextBuilder::new().build_raw_context(win.hwnd()).unwrap().make_current().unwrap()
    };
    let gl =unsafe{
        glow::Context::from_loader_function(|fn_name|ctx.get_proc_address(fn_name))

    };

    let mut pipeline_state = PipelineState::new(gl);
    pipeline_state.set_clear_color(Color::from_rgba(255, 0, 0, 0));

    let pg = GpuProgram::from_source(&mut pipeline_state,"todo",render_pass.vertex_shader.as_str(),render_pass.fragment_shader.as_str()).unwrap();
    pg.bind(&mut pipeline_state);
    

    //let surface_data = SurfaceData::make_cube(
    //    Matrix4::new_nonuniform_scaling(&Vector3::new(
    //            0.25f32, 0.25, 0.25,
    //            ))
    //    );
    let surface_data = SurfaceData::make_unit_xy_quad();
    let geometry_buffer = GeometryBuffer::from_surface_data(&surface_data, GeometryBufferKind::StaticDraw,&mut pipeline_state,);

    ev.run(move|_,_,_|{
        unsafe{
            pipeline_state.gl.clear(glow::COLOR_BUFFER_BIT);
        };
        geometry_buffer.bind(&mut pipeline_state).draw();
        ctx.swap_buffers().unwrap();
    })

}

