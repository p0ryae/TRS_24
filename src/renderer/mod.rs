mod renderer;
mod model;
mod camera;
mod mesh;
mod shader;
mod texture;

pub use model::Model;
pub use renderer::Renderer;

mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

    pub use Gles2 as Gl;
}
