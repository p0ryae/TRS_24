use crate::renderer::camera;
use crate::renderer::get_gl_string;
use crate::renderer::gl;
use crate::renderer::model;
use crate::renderer::shader;

use glutin::display::Display;
use glutin::prelude::*;
use std::ffi::CString;

pub struct Renderer {
    program: gl::types::GLuint,
    gl: gl::Gl,
    models: Vec<model::Model>,
}

impl Renderer {
    pub fn new(gl_display: &Display) -> Self {
        unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
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

            /*
            let mut x1 = model::Model::new(
                gl.clone(),
                program,
                include_bytes!("../../assets/models/scene/scene.gltf"),
                include_bytes!("../../assets/models/scene/scene.bin")
            );
            x1.set_position(nalgebra_glm::vec3(1.0, 0.0, 0.0));
            x1.set_scale(nalgebra_glm::vec3(0.02, 0.02, 0.02));
            x1.set_rotation(nalgebra_glm::quat_angle_axis(80.0, &nalgebra_glm::vec3(0.0, 1.0, 0.0)));
            models.push(x1);
            */

            let mut x2 = model::Model::new(
                gl.clone(),
                program,
                include_bytes!("../../assets/models/scene2/scene.gltf"),
                include_bytes!("../../assets/models/scene2/scene.bin"),
                include_bytes!("../../assets/models/scene2/Material.001_baseColor.png")
            );
            x2.set_position(nalgebra_glm::vec3(-0.5, 0.0, 0.0));
            x2.set_scale(nalgebra_glm::vec3(0.09, 0.09, 0.09));
            models.push(x2);

            let mut x3 = model::Model::new(
                gl.clone(),
                program,
                include_bytes!("../../assets/models/scene3/scene.gltf"),
                include_bytes!("../../assets/models/scene3/scene.bin"),
                include_bytes!("../../assets/models/scene3/texture.png")
            );
            x3.set_position(nalgebra_glm::vec3(0.7, -0.5, 0.0));
            x3.set_scale(nalgebra_glm::vec3(0.007, 0.007, 0.007));
            models.push(x3);

            Self {
                program,
                gl,
                models,
            }
        }
    }

    pub fn draw(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl.ClearColor(0.4, 0.0, 1.0, 1.0);
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

    // Not needed, since the engine prefers landscape mode by default
    #[allow(dead_code)]
    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}
