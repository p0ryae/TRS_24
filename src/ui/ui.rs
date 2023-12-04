use crate::overture::{self, RGBA};
use crate::renderer::gl;
use crate::types;

pub struct Element {
    pub shape: types::Shape,
    pub image: Option<&'static [u8]>,
    pub color: RGBA,
    pub position: overture::Vec3,
    pub scale: overture::Vec3,
    pub rotation: f32
}

impl Element {
    pub fn new(
        shape: types::Shape,
    ) -> Self {
        Self {
            shape,
            image: None,
            color: RGBA::new(0.1, 0.1, 0.1, 1.0),
            position: overture::Vec3::new(0.0, 0.0, 0.0),
            scale: overture::Vec3::new(1.0, 1.0, 1.0),
            rotation: 0.0
        }
    }

    pub fn set_position(mut self, position: overture::Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn set_scale(mut self, scale: overture::Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn set_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    // pub fn set_image(mut self, image: &'static [u8]) -> Self {
    //    self.image = Some(image);
    //    self
    //}

    pub fn set_color(mut self, rgba: RGBA) -> Self {
        self.color = rgba;
        self
    }
}

pub struct ReadyElement {
    gl: gl::Gl,
    indices: Vec<i32>,
    position: nalgebra_glm::Vec3,
    scale: nalgebra_glm::Vec3,
    rotation: nalgebra_glm::Quat,
    vbo: gl::types::GLuint,
    ebo: gl::types::GLuint,
}

impl ReadyElement {
    pub fn new(
        gl: gl::Gl,
        shape: &types::Shape,
        //image: Option<&'static [u8]>,
        rgba: &RGBA,
        position: nalgebra_glm::Vec3,
        scale: nalgebra_glm::Vec3,
        rotation: nalgebra_glm::Quat,
    ) -> Self {
        unsafe {
            let vertices: Vec<f32>;
            let indices: Vec<i32>;

            match shape {
                #[rustfmt::skip]
                types::Shape::Square => {
                    vertices = [
                        -0.5, -0.5, 0.0,   rgba.r, rgba.g, rgba.b, rgba.a,   0.0, 0.0, 
                        -0.5,  0.5, 0.0,   rgba.r, rgba.g, rgba.b, rgba.a,   0.0, 1.0, 
                         0.5,  0.5, 0.0,   rgba.r, rgba.g, rgba.b, rgba.a,   1.0, 1.0, 
                         0.5, -0.5, 0.0,   rgba.r, rgba.g, rgba.b, rgba.a,   1.0, 0.0,
                    ]
                    .to_vec();

                    indices = [
                        0, 2, 1, 
                        0, 3, 2
                    ]
                    .to_vec();
                }
                #[rustfmt::skip]
                types::Shape::Triangle => {
                    vertices = [
                         0.0,  0.5, 0.0,   rgba.r, rgba.g, rgba.b, rgba.a,   0.5, 1.0,
                        -0.5, -0.5, 0.0,   rgba.r, rgba.g, rgba.b, rgba.a,   0.0, 0.0, 
                         0.5, -0.5, 0.0,   rgba.r, rgba.g, rgba.b, rgba.a,   1.0, 0.0,
                    ]
                    .to_vec();

                    indices = [
                        0, 1, 2
                    ].to_vec();
                }
            }

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let mut ebo = std::mem::zeroed();
            gl.GenBuffers(1, &mut ebo);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            Self {
                gl,
                indices,
                position,
                scale,
                rotation,
                vbo,
                ebo,
            }
        }
    }

    pub fn draw(&self, program: gl::types::GLuint) {
        unsafe {
            self.gl.UseProgram(program);

            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);

            let translation_matrix =
                nalgebra_glm::translate(&nalgebra_glm::Mat4::identity(), &self.position);
            let rotation_matrix = nalgebra_glm::quat_to_mat4(&self.rotation);
            let scale_matrix = nalgebra_glm::scale(&nalgebra_glm::Mat4::identity(), &self.scale);

            let model_matrix = translation_matrix * rotation_matrix * scale_matrix;

            self.gl.UniformMatrix4fv(
                self.gl
                    .GetUniformLocation(program, b"matrix\0".as_ptr() as *const _),
                1,
                gl::FALSE,
                model_matrix.as_slice().as_ptr() as *const f32,
            );

            let pos_attrib = self
                .gl
                .GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            self.gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                gl::FALSE,
                9 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            self.gl
                .EnableVertexAttribArray(pos_attrib as gl::types::GLuint);

            let color_attrib = self
                .gl
                .GetAttribLocation(program, b"color\0".as_ptr() as *const _);
            self.gl.VertexAttribPointer(
                color_attrib as gl::types::GLuint,
                4,
                gl::FLOAT,
                gl::FALSE,
                9 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            self.gl
                .EnableVertexAttribArray(color_attrib as gl::types::GLuint);

            let tex_attrib = self
                .gl
                .GetAttribLocation(program, b"tex\0".as_ptr() as *const _);
            self.gl.VertexAttribPointer(
                tex_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                gl::FALSE,
                9 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (6 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            self.gl
                .EnableVertexAttribArray(tex_attrib as gl::types::GLuint);

            self.gl.DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }
}
