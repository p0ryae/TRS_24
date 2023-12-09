use crate::renderer::gl;
use crate::renderer::texture;

pub struct Mesh {
    gl: gl::Gl,
    indices: Vec<u32>,
    texture: texture::Texture,
    vbo: gl::types::GLuint,
    ebo: gl::types::GLuint,
}

impl Mesh {
    pub fn new(
        gl: gl::Gl,
        vertices: Vec<f32>,
        indices: Vec<u32>,
        texture: texture::Texture,
    ) -> Self {
        unsafe {
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
                texture,
                ebo,
                vbo,
            }
        }
    }
    pub fn draw(
        &self,
        program: gl::types::GLuint,
        position: nalgebra_glm::Vec3,
        scale: nalgebra_glm::Vec3,
        rotation: nalgebra_glm::Quat,
    ) {
        unsafe {
            self.gl.UseProgram(program);

            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);

            let translation_matrix =
                nalgebra_glm::translate(&nalgebra_glm::Mat4::identity(), &position);
            let rotation_matrix = nalgebra_glm::quat_to_mat4(&rotation);
            let scale_matrix = nalgebra_glm::scale(&nalgebra_glm::Mat4::identity(), &scale);

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
                11 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            self.gl
                .EnableVertexAttribArray(pos_attrib as gl::types::GLuint);

            let normal_attrib = self
                .gl
                .GetAttribLocation(program, b"normal\0".as_ptr() as *const _);
            self.gl.VertexAttribPointer(
                normal_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                gl::FALSE,
                11 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            self.gl
                .EnableVertexAttribArray(normal_attrib as gl::types::GLuint);

            let color_attrib = self
                .gl
                .GetAttribLocation(program, b"color\0".as_ptr() as *const _);
            self.gl.VertexAttribPointer(
                color_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                gl::FALSE,
                11 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (6 * std::mem::size_of::<f32>()) as *const () as *const _,
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
                11 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (9 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            self.gl
                .EnableVertexAttribArray(tex_attrib as gl::types::GLuint);

            self.gl.Uniform1i(
                self.gl
                    .GetUniformLocation(program, b"tex0\0".as_ptr() as *const _),
                0,
            );
            self.texture.bind();

            self.gl.DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            self.texture.unbind();
        }
    }
}
