use crate::overture;
use crate::renderer::gl;
use crate::renderer::model;
use crate::renderer::shader;
use crate::renderer::Camera;
use crate::types;
use crate::ui;

use glutin::display::Display;
use glutin::prelude::*;

pub struct Renderer {
    program: gl::types::GLuint,
    program_ui: gl::types::GLuint,
    gl: gl::Gl,
    models: Vec<model::ReadyModel>,
    ui: Vec<types::ElementType>,
}

impl Renderer {
    pub fn new(
        gl_display: &Display,
        not_ready_models: &Vec<model::Model>,
        not_ready_ui: &Vec<ui::Element>,
    ) -> Self {
        unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            unsafe fn get_gl_string(
                gl: &gl::Gl,
                variant: gl::types::GLenum,
            ) -> Option<&'static std::ffi::CStr> {
                let s = gl.GetString(variant);
                (!s.is_null()).then(|| std::ffi::CStr::from_ptr(s.cast()))
            }

            if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(&gl, gl::VERSION) {
                println!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            let program = shader::create_init_shader(
                gl.clone(),
                include_bytes!("./shaders/shader-vert.glsl"),
                include_bytes!("./shaders/shader-frag.glsl"),
            );
            let program_ui = shader::create_init_shader(
                gl.clone(),
                include_bytes!("../ui/shaders/shader-vert.glsl"),
                include_bytes!("../ui/shaders/shader-frag.glsl"),
            );

            gl.Enable(gl::DEPTH_TEST);
            gl.Enable(gl::BLEND);
            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            let mut models = Vec::new();

            for model in not_ready_models {
                let mut x = model::ReadyModel::new(
                    gl.clone(),
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
                match model.rotation.1 {
                    types::RotAxis::Pitch => {
                        x.set_rotation(nalgebra_glm::quat_angle_axis(
                            model.rotation.0,
                            &nalgebra_glm::vec3(1.0, 0.0, 0.0),
                        ));
                    }
                    types::RotAxis::Roll => {
                        x.set_rotation(nalgebra_glm::quat_angle_axis(
                            model.rotation.0,
                            &nalgebra_glm::vec3(0.0, 1.0, 0.0),
                        ));
                    }
                    types::RotAxis::Yaw => {
                        x.set_rotation(nalgebra_glm::quat_angle_axis(
                            model.rotation.0,
                            &nalgebra_glm::vec3(0.0, 0.0, 1.0),
                        ));
                    }
                }

                models.push(x);
            }

            let mut ui: Vec<types::ElementType> = Vec::new();

            for element in not_ready_ui {
                match &element.el_type {
                    types::ElementType::Shape(shape_builder) => {
                        let x = types::ElementType::Shape(ui::ShapeBuilder::new_instance(
                            gl.clone(),
                            shape_builder,
                            &element.color,
                            nalgebra_glm::vec3(
                                element.position.x,
                                element.position.y,
                                element.position.z,
                            ),
                            nalgebra_glm::vec3(element.scale.x, element.scale.y, element.scale.z),
                            nalgebra_glm::quat_angle_axis(
                                element.rotation,
                                &nalgebra_glm::vec3(0.0, 0.0, 1.0),
                            ),
                        ));
                        ui.push(x);
                    }
                    types::ElementType::Text(text_builder) => {
                        let x = types::ElementType::Text(ui::TextBuilder::new_instance(
                            gl.clone(),
                            text_builder,
                            element.color.clone(),
                            nalgebra_glm::vec3(
                                element.position.x,
                                element.position.y,
                                element.position.z,
                            ),
                            nalgebra_glm::vec3(element.scale.x, element.scale.y, element.scale.z),
                            nalgebra_glm::quat_angle_axis(
                                element.rotation,
                                &nalgebra_glm::vec3(0.0, 0.0, 1.0),
                            ),
                        ));
                        ui.push(x);
                    }
                }
            }

            Self {
                program,
                program_ui,
                gl,
                models,
                ui,
            }
        }
    }

    pub fn draw(&mut self, world_color: &overture::RGBA, camera: &Camera) {
        unsafe {
            self.gl
                .ClearColor(world_color.r, world_color.g, world_color.b, world_color.a);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            camera.adjust(self.gl.clone(), self.program, 45.0, 0.1, 100.0);

            for model in &self.models {
                model.draw(self.program);
            }

            self.gl.Clear(gl::DEPTH_BUFFER_BIT);

            camera.adjust(self.gl.clone(), self.program_ui, 45.0, 0.1, 100.0);

            for element in &self.ui {
                match element {
                    types::ElementType::Shape(shape_instance) => {
                        shape_instance.draw(self.program_ui);
                    }
                    types::ElementType::Text(text_instance) => {
                        text_instance.draw(self.program_ui);
                    }
                }
            }
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}
