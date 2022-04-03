extern crate rs_gl;
use rs_gl::Gl;
use rs_gl::glow;
use rs_gl::types::*;
use rs_gl::glow::HasContext;
use rs_gl::mesh::Mesh;
use glutin::platform::windows::RawContextExt;
use winit::window::{WindowBuilder};
use glutin::ContextBuilder;
use glutin::platform::windows::WindowExtWindows;
use winit::event_loop::EventLoop;
use nalgebra_glm as glm;

fn main(){
    let ev = EventLoop::new();
    let win = WindowBuilder::new().build(&ev).unwrap();

    let ctx = unsafe{
        ContextBuilder::new().build_raw_context(win.hwnd()).unwrap().make_current().unwrap()
    };
    let gl = Gl::from_load_fn(|fn_name|ctx.get_proc_address(fn_name));

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

    let mesh  = [
        Mesh::new(&gl,vec![
                  glm::vec3(0.0f32,0.5,0.0),
                  glm::vec3(-0.5f32,-0.5f32,0.0),
                  glm::vec3(0.5,-0.5,0.0),

        ]),
        Mesh::new(
            &gl,
            vec![
                  glm::vec3(-0.532,0.5,0.0),
                  glm::vec3(0.0f32,0.5,0.0),
                  glm::vec3(-0.5f32,-0.5f32,0.0),
            ],
            ),

    ];



    ev.run(move |_,_,_|{
        gl.draw_mesh(&mesh);
        ctx.swap_buffers().unwrap();
    });
}

