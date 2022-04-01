

pub struct ShaderProgram{
    pg: glow::Program,
     vertex_shader: glow::Shader,
    frag_shader: glow::Shader,
}

impl ShaderProgram{
    pub fn new(pg: glow::Program,vertex_shader: glow::Shader,frag_shader: glow::Shader)->Self{
        return ShaderProgram { pg: pg, vertex_shader: vertex_shader, frag_shader: frag_shader };
    }

    pub fn pg_id(&self)->glow::Program{
        return self.pg;
    }
}
