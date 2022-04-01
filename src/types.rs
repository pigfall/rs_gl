

pub struct ShaderType(pub u32);

impl ShaderType{
    pub fn VertexShader()->Self{
        return ShaderType(glow::VERTEX_SHADER)
    }
    pub fn FragShader()->Self{
        return ShaderType(glow::FRAGMENT_SHADER)
    }
}

pub struct VertexComponentDataType(pub u32);

impl VertexComponentDataType{
    pub fn float32()->Self{
        Self(glow::FLOAT)
    }
}

pub struct VertexAttribPointerShouleBeNormalized(pub u8);

impl VertexAttribPointerShouleBeNormalized{
    pub fn false_value()->Self{
        return Self(0)
    }

    pub fn true_value()->Self{
        return Self(1)
    }
}
