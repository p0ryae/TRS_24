use crate::overture;
use crate::renderer::camera;
use crate::renderer::get_gl_string;
use crate::renderer::gl;
use crate::renderer::model;
use crate::renderer::shader;

use glutin::display::Display;
use glutin::prelude::*;

pub struct Model {
    gltf_file: &'static [u8],
    bin_file: &'static [u8],
    texture_file: &'static [u8],
    position: overture::Vec3,
    scale: overture::Vec3,
    rotation: f32,
}

pub struct Renderer {
    program: gl::types::GLuint,
    gl: gl::Gl,
    models: Vec<model::model::Model>,
}

impl Renderer {
    pub fn new(gl_display: &Display, not_ready_models: &Vec<Model>) -> Self {
        unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(&gl, gl::VERSION) {
                println!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            let program = shader::create_init_shader(gl.clone());

            gl.Enable(gl::DEPTH_TEST);

            let mut models = Vec::new();

            for model in not_ready_models {
                let mut x = model::model::Model::new(
                    gl.clone(),
                    program,
                    model.gltf_file,
                    model.bin_file,
                    model.texture_file,
                );
                x.set_position(nalgebra_glm::vec3(
                    model.position.x,
                    model.position.y,
                    model.position.z,
                ));
                x.set_scale(nalgebra_glm::vec3(
                    model.scale.x,
                    model.scale.y,
                    model.scale.z,
                ));
                x.set_rotation(nalgebra_glm::quat_angle_axis(
                    model.rotation,
                    &nalgebra_glm::vec3(0.0, 1.0, 0.0),
                ));

                models.push(x);
            }

            Self {
                program,
                gl,
                models,
            }
        }
    }

    pub fn draw(&mut self, width: i32, height: i32, world_color: &overture::RGBA) {
        unsafe {
            self.gl
                .ClearColor(world_color.r, world_color.g, world_color.b, world_color.a);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            camera::adjust(
                self.gl.clone(),
                self.program,
                width as f32,
                height as f32,
                45.0,
                0.1,
                100.0,
            );

            for model in &self.models {
                model.draw();
            }
        }
    }

    pub fn new_model(
        gltf_file: &'static [u8],
        bin_file: &'static [u8],
        texture_file: &'static [u8],
        position: overture::Vec3,
        scale: overture::Vec3,
        rotation: f32,
    ) -> Model {
        return Model {
            gltf_file,
            bin_file,
            texture_file,
            position,
            scale,
            rotation,
        };
    }

    // Not needed, since the engine prefers landscape mode by default
    #[allow(dead_code)]
    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}
