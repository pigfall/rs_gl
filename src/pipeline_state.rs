use glow::HasContext;
use std::fmt::Display;
pub struct PipelineState {
    pub gl: glow::Context,
    vao: Option<glow::VertexArray>,
    frame_statistics: PipelineStatistics,
    vbo: Option<glow::Buffer>,

}
use std::fmt::Formatter;

impl PipelineState{
    pub fn set_vertex_array_object(&mut self, vao: Option<glow::VertexArray>) {
        if self.vao != vao {
            self.vao = vao;

            self.frame_statistics.vao_binding_changes += 1;

            unsafe {
                self.gl.bind_vertex_array(self.vao);
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
