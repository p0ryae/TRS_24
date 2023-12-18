mod camera;
mod mesh;
mod model;
mod renderer;
mod shader;
mod texture;

pub use camera::Camera;
pub use model::Model;
pub use renderer::Renderer;

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

    pub use Gles2 as Gl;

    impl std::fmt::Debug for Gles2 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Gles2")
        }
    }
}
