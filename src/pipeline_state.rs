use glow::HasContext;
use crate::core::color::Color;
use std::fmt::Display;
pub struct PipelineState {
    pub gl: glow::Context,
    vao: Option<glow::VertexArray>,
    frame_statistics: PipelineStatistics,
    vbo: Option<glow::Buffer>,
    program: Option<glow::Program>,
    texture_units: [TextureUnit; 32],
    clear_color: Color,

}
use std::fmt::Formatter;

impl PipelineState{
    pub fn set_texture(&mut self, sampler_index: u32, target: u32, texture: Option<glow::Texture>) {
        let unit = self.texture_units.get_mut(sampler_index as usize).unwrap();

        if unit.target != target || unit.texture != texture {
            unit.texture = texture;
            unit.target = target;

            self.frame_statistics.texture_binding_changes += 1;

            unsafe {
                self.gl.active_texture(glow::TEXTURE0 + sampler_index);
                self.gl.bind_texture(target, unit.texture);
            }
        }
    }

    pub fn new(context: glow::Context) -> Self {

        Self {
            gl: context,
            vao: Default::default(),
            vbo: Default::default(),
            frame_statistics: Default::default(),
            program: Default::default(),
            texture_units: [Default::default(); 32],
            clear_color: Color::from_rgba(0, 0, 0, 0),
        }
    }
    pub fn set_vertex_array_object(&mut self, vao: Option<glow::VertexArray>) {
        if self.vao != vao {
            self.vao = vao;

            self.frame_statistics.vao_binding_changes += 1;

            unsafe {
                self.gl.bind_vertex_array(self.vao);
            }
        }
    }

    pub fn set_program(&mut self, program: Option<glow::Program>) {
        if self.program != program {
            self.program = program;

            self.frame_statistics.program_binding_changes += 1;

            unsafe {
                self.gl.use_program(self.program);
            }
        }
    }

    pub fn set_vertex_buffer_object(&mut self, vbo: Option<glow::Buffer>) {
        if self.vbo != vbo {
            self.vbo = vbo;

            self.frame_statistics.vbo_binding_changes += 1;

            unsafe {
                self.gl.bind_buffer(glow::ARRAY_BUFFER, self.vbo);
            }
        }
    }

    pub fn set_clear_color(&mut self, color: Color) {
        if self.clear_color != color {
            self.clear_color = color;

            let rgba = color.as_frgba();
            unsafe {
                self.gl.clear_color(rgba.x, rgba.y, rgba.z, rgba.w);
            }
        }
    }
}


#[derive(Debug, Default, Copy, Clone)]
pub struct PipelineStatistics {
    pub texture_binding_changes: usize,
    pub vbo_binding_changes: usize,
    pub vao_binding_changes: usize,
    pub blend_state_changes: usize,
    pub framebuffer_binding_changes: usize,
    pub program_binding_changes: usize,
}


impl Display for PipelineStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pipeline state changes:\n\
            \tTextures: {},\n\
            \tVBO: {},\n\
            \tVAO: {},\n\
            \tFBO: {},\n\
            \tShaders: {},\n\
            \tBlend: {}",
            self.texture_binding_changes,
            self.vbo_binding_changes,
            self.vao_binding_changes,
            self.framebuffer_binding_changes,
            self.program_binding_changes,
            self.blend_state_changes
        )
    }
}


#[derive(Copy, Clone)]
struct TextureUnit {
    target: u32,
    texture: Option<glow::Texture>,
}

impl Default for TextureUnit {
    fn default() -> Self {
        Self {
            target: glow::TEXTURE_2D,
            texture: Default::default(),
        }
    }
}
