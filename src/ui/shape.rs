use crate::renderer::gl;
use crate::types;

#[derive(Clone)]
pub struct ShapeBuilder {
    pub shape: types::Shape,
    pub is_hud: bool,
    gl: gl::Gl,
    indices: Vec<i32>,
    position: nalgebra_glm::Vec3,
    scale: nalgebra_glm::Vec3,
    rotation: nalgebra_glm::Quat,
    vbo: gl::types::GLuint,
    ebo: gl::types::GLuint,
    texture: gl::types::GLuint,
}

impl ShapeBuilder {
    pub fn new(shape: types::Shape) -> Self {
        unsafe {
            Self { 
                shape,
                is_hud: false,
                gl: std::mem::zeroed(),
                indices: Vec::new(),
                position: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
                rotation: nalgebra_glm::quat_angle_axis(
                    0.0,
                    &nalgebra_glm::vec3(0.0, 0.0, 0.0),
                ),
                vbo: std::mem::zeroed(),
                ebo: std::mem::zeroed(),
                texture: std::mem::zeroed()
            }
        }
    }

    pub fn new_instance(
        gl: gl::Gl,
        shape_builder: &ShapeBuilder,
        is_hud: bool, 
        rgba: &types::RGBA,
        position: nalgebra_glm::Vec3,
        scale: nalgebra_glm::Vec3,
        rotation: nalgebra_glm::Quat,
    ) -> Self {
        unsafe {
            let vertices: Vec<f32>;
            let indices: Vec<i32>;

            match shape_builder.shape {
                #[rustfmt::skip]
                types::Shape::Square => {
                    vertices = [
                        -0.5, -0.5,  rgba.r, rgba.g, rgba.b, rgba.a,   0.0, 0.0, 
                        -0.5,  0.5,  rgba.r, rgba.g, rgba.b, rgba.a,   0.0, 1.0, 
                         0.5,  0.5,  rgba.r, rgba.g, rgba.b, rgba.a,   1.0, 1.0, 
                         0.5, -0.5,  rgba.r, rgba.g, rgba.b, rgba.a,   1.0, 0.0,
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
                         0.0,  0.5,  rgba.r, rgba.g, rgba.b, rgba.a,   0.5, 1.0,
                        -0.5, -0.5,  rgba.r, rgba.g, rgba.b, rgba.a,   0.0, 0.0, 
                         0.5, -0.5,  rgba.r, rgba.g, rgba.b, rgba.a,   1.0, 0.0,
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

            let mut texture = std::mem::zeroed();
            gl.GenTextures(1, &mut texture);
            gl.BindTexture(gl::TEXTURE_2D, texture);
            let data: [u8; 1] = [255];
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::LUMINANCE as i32,
                1,
                1,
                0,
                gl::LUMINANCE,
                gl::UNSIGNED_BYTE,
                &data[0] as *const u8 as *const std::ffi::c_void,
            );

            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl.BindTexture(gl::TEXTURE_2D, 0);
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            Self {
                shape: shape_builder.clone().shape,
                is_hud,
                gl,
                indices,
                position,
                scale,
                rotation,
                vbo,
                ebo,
                texture,
            }
        }
    }

    pub fn draw(&self, program: gl::types::GLuint) {
        unsafe {
            self.gl.UseProgram(program);

            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);

            self.gl.ActiveTexture(gl::TEXTURE0);
            self.gl.BindTexture(gl::TEXTURE_2D, self.texture);

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
                2,
                gl::FLOAT,
                gl::FALSE,
                8 * std::mem::size_of::<f32>() as gl::types::GLsizei,
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
                8 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
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
                8 * std::mem::size_of::<f32>() as gl::types::GLsizei,
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

            self.gl.BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}
