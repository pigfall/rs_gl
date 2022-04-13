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

use rs_gl::gl_wrapper::vertex_buffer::*;

fn main(){
    let ev = EventLoop::new();
    let win = WindowBuilder::new().build(&ev).unwrap();

    let ctx = unsafe{
        ContextBuilder::new().build_raw_context(win.hwnd()).unwrap().make_current().unwrap()
    };
    let gl = Gl::from_load_fn(|fn_name|ctx.get_proc_address(fn_name));

    let gl_glow = unsafe{glow::Context::from_loader_function(|fn_name|ctx.get_proc_address(fn_name))};

    const vertex_shader:&str="#version 330 core
    attribute vec3 attrib_pos;
    void main(){
        gl_Position = vec4(attrib_pos,1.0);
    }
    ";
    const frag_shader:&str ="#version 330 core
    void main(){
        gl_FragColor = vec4(0.1,0.2,0.3,0.5);
    }

    ";

    let data = [
        0.0f32,0.5,0.0,
        -0.5,-0.5,0.0,
        0.5,-0.5,0.0,
    ];

    let pg = gl.make_program(vertex_shader,frag_shader).unwrap();
    gl.use_program(Some(pg.pg_id()));

    let mut state = PipelineState{gl:gl_glow};
    
    
    let data_bytes = data.iter().flat_map(|e|e.to_le_bytes()).collect::<Vec<u8>>();


    let gbuffer = GeometryBuffer::unit_rectangle(&mut state);



    ev.run(move |_,_,_|{
        //gl.draw_arrays(DrawArrayMode::triangle(),0,3);
        gbuffer.bind(&mut state).draw(&mut state);
        ctx.swap_buffers().unwrap();
    });
}

