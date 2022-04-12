use crate::PhantomData;
use crate::{
    PipelineState,
};
pub struct GpuTexture {
    state: *mut PipelineState,
    texture: glow::Texture,
    kind: GpuTextureKind,
    min_filter: MinificationFilter,
    mag_filter: MagnificationFilter,
    s_wrap_mode: WrapMode,
    t_wrap_mode: WrapMode,
    r_wrap_mode: WrapMode,
    anisotropy: f32,
    pixel_kind: PixelKind,
    // Force compiler to not implement Send and Sync, because OpenGL is not thread-safe.
    thread_mark: PhantomData<*const u8>,
}

impl GpuTexture{
    pub fn bind(&self, state: &mut PipelineState, sampler_index: u32) {
        state.set_texture(
            sampler_index,
            self.kind.gl_texture_target(),
            Some(self.texture),
        );
    }
}


#[derive(Copy, Clone)]
pub enum GpuTextureKind {
    Line {
        length: usize,
    },
    Rectangle {
        width: usize,
        height: usize,
    },
    Cube {
        width: usize,
        height: usize,
    },
    Volume {
        width: usize,
        height: usize,
        depth: usize,
    },
}

impl GpuTextureKind {
    fn gl_texture_target(&self) -> u32 {
        match self {
            Self::Line { .. } => glow::TEXTURE_1D,
            Self::Rectangle { .. } => glow::TEXTURE_2D,
            Self::Cube { .. } => glow::TEXTURE_CUBE_MAP,
            Self::Volume { .. } => glow::TEXTURE_3D,
        }
    }
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MinificationFilter {
    Nearest = glow::NEAREST,
    NearestMipMapNearest = glow::NEAREST_MIPMAP_NEAREST,
    NearestMipMapLinear = glow::NEAREST_MIPMAP_LINEAR,
    Linear = glow::LINEAR,
    LinearMipMapNearest = glow::LINEAR_MIPMAP_NEAREST,
    LinearMipMapLinear = glow::LINEAR_MIPMAP_LINEAR,
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MagnificationFilter {
    Nearest,
    Linear,
}


#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum WrapMode {
    Repeat = glow::REPEAT,
    ClampToEdge = glow::CLAMP_TO_EDGE,
    ClampToBorder = glow::CLAMP_TO_BORDER,
    MirroredRepeat = glow::MIRRORED_REPEAT,
    MirrorClampToEdge = glow::MIRROR_CLAMP_TO_EDGE,
}

impl WrapMode {
    pub fn into_gl_value(self) -> i32 {
        self as i32
    }
}


#[derive(Copy, Clone, Debug)]
pub enum PixelKind {
    F32,
    F16,
    D32F,
    D16,
    D24S8,
    RGBA8,
    SRGBA8,
    RGB8,
    SRGB8,
    BGRA8,
    BGR8,
    RG8,
    RG16,
    R8,
    R8UI,
    R16,
    RGB16,
    RGBA16,
    DXT1RGB,
    DXT1RGBA,
    DXT3RGBA,
    DXT5RGBA,
    RGB32F,
    RGBA32F,
    RGBA16F,
    R8RGTC,
    RG8RGTC,
    R11G11B10F,
    RGB10A2,
}


