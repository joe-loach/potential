use crate::mq;
use mq::{Context, Shader, ShaderError, ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

const VERTEX: &str = include_str!("vertex.glsl");
const FRAGMENT: &str = include_str!("frag.glsl");

pub fn shader(ctx: &mut Context) -> Result<Shader, ShaderError> {
    let meta = ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![UniformDesc::new("transform".into(), UniformType::Mat4)],
        },
    };
    Shader::new(ctx, VERTEX, FRAGMENT, meta)
}
