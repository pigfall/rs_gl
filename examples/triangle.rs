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
        gl_FragColor = vec4(0.1,0.2,0.3,0.0);
    }

    ";

    let data = [
        0.0f32,0.5,0.0,
        -0.5,-0.5,0.0,
        0.5,-0.5,0.0,
    ];

    let buffer_id = gl.gen_buffer().unwrap();
    gl.bind_buffer(TargetBindBuffer::ArrayBuffer(),Some(buffer_id));
    gl.buffer_data_u8_slice(
        TargetBindBuffer::ArrayBuffer(),
        &(data.iter().flat_map(|e|e.to_le_bytes()).collect::<Vec<u8>>())[..],
        BufferDataUsage::StaticDraw(),
        );

    let pg = gl.make_program(vertex_shader,frag_shader).unwrap();

    let attirb_pos_loc ={
        gl.get_attrib_location(pg.pg_id(),"attrib_pos").unwrap()
    };
    gl.vertex_attrib_pointer(
        attirb_pos_loc,
        3,
        VertexComponentDataType::float32(),
        VertexAttribPointerShouleBeNormalized::false_value(),
        0,
        0,
        );

    gl.enable_vertex_attrib_array(attirb_pos_loc);

    gl.get_error().unwrap();

    ev.run(move |_,_,_|{
        gl.draw_arrays(DrawArrayMode::triangle(),0,3);
        ctx.swap_buffers().unwrap();
    });
}

