use crate::overture::*;
use crate::renderer::gl;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TextBuilder {
    text: String,
    font: &'static [u8],
    size: u32,
    gl: gl::Gl,
    position: nalgebra_glm::Vec3,
    scale: nalgebra_glm::Vec3,
    rotation: nalgebra_glm::Quat,
    rgba: RGBA,
    characters: HashMap<char, Character>,
    vbo: gl::types::GLuint,
}

#[derive(Clone)]
struct Character {
    texture: u32,
    size: (i32, i32),
    bearing: (i32, i32),
    advance: i32,
}

impl TextBuilder {
    pub fn new(text: String, font_data: &'static [u8], size: u32) -> Self {
        unsafe {
            Self {
                text,
                font: font_data,
                size,
                gl: std::mem::zeroed(),
                position: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                scale: nalgebra_glm::vec3(1.0, 1.0, 1.0),
                rotation: nalgebra_glm::quat_angle_axis(0.0, &nalgebra_glm::vec3(0.0, 0.0, 0.0)),
                rgba: RGBA::new(0.1, 0.1, 0.1, 1.0),
                characters: HashMap::new(),
                vbo: std::mem::zeroed(),
            }
        }
    }

    pub fn new_instance(
        gl: gl::Gl,
        text_builder: &TextBuilder,
        rgba: RGBA,
        position: nalgebra_glm::Vec3,
        scale: nalgebra_glm::Vec3,
        rotation: nalgebra_glm::Quat,
    ) -> Self {
        unsafe {
            let mut characters: HashMap<char, Character> = HashMap::new();

            let ft = freetype::Library::init().unwrap();
            let face = ft.new_memory_face(text_builder.font.to_vec(), 0).unwrap();
            face.set_pixel_sizes(0, text_builder.size).unwrap();

            gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            
            for c in 0..128u8 {
                face.load_char(c as usize, freetype::face::LoadFlag::RENDER)
                    .unwrap();

                let mut texture = 0;
                gl.GenTextures(1, &mut texture);
                gl.BindTexture(gl::TEXTURE_2D, texture);
                gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::LUMINANCE as gl::types::GLint,
                    face.glyph().bitmap().width(),
                    face.glyph().bitmap().rows(),
                    0,
                    gl::LUMINANCE,
                    gl::UNSIGNED_BYTE,
                    face.glyph().bitmap().buffer().as_ptr() as *const std::ffi::c_void,
                );

                gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

                let character = Character {
                    texture: texture as u32,
                    size: (face.glyph().bitmap().width(), face.glyph().bitmap().rows()),
                    bearing: (face.glyph().bitmap_left(), face.glyph().bitmap_top()),
                    advance: face.glyph().advance().x as i32,
                };

                characters.insert(c as char, character);
            }

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (48 * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            
            gl.BindTexture(gl::TEXTURE_2D, 0);
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);

            let cloned = text_builder.clone();

            Self {
                text: cloned.text,
                font: cloned.font,
                size: cloned.size,
                gl,
                position,
                scale,
                rotation,
                rgba,
                characters,
                vbo,
            }
        }
    }

    pub fn draw(&self, program: gl::types::GLuint) {
        unsafe {
            self.gl.UseProgram(program);

            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            self.gl.ActiveTexture(gl::TEXTURE0);

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

            self.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            
            let mut x = self.position.x;
            let y = self.position.y;

            for c in self.text.chars() {
                let ch = self.characters.get(&c).unwrap();

                let xpos = x as f32 + ch.bearing.0 as f32;
                let ypos = y as f32 - (ch.size.1 - ch.bearing.1) as f32;
        
                let w = ch.size.0 as f32;
                let h = ch.size.1 as f32;

                #[rustfmt::skip]
                let vertices: [f32; 48] = [
                    xpos,     ypos + h,    self.rgba.r, self.rgba.g, self.rgba.b, self.rgba.a,    0.0,  0.0,
                    xpos,     ypos,        self.rgba.r, self.rgba.g, self.rgba.b, self.rgba.a,    0.0,  1.0,
                    xpos + w, ypos,        self.rgba.r, self.rgba.g, self.rgba.b, self.rgba.a,    1.0,  1.0,
        
                    xpos,     ypos + h,    self.rgba.r, self.rgba.g, self.rgba.b, self.rgba.a,    0.0,  0.0,
                    xpos + w, ypos,        self.rgba.r, self.rgba.g, self.rgba.b, self.rgba.a,    1.0,  1.0,
                    xpos + w, ypos + h,    self.rgba.r, self.rgba.g, self.rgba.b, self.rgba.a,    1.0,  0.0  
                ];

                self.gl.BindTexture(gl::TEXTURE_2D, ch.texture);

                self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                self.gl.BufferSubData(
                    gl::ARRAY_BUFFER, 
                    0, 
                    (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, 
                    vertices.as_ptr() as *const _
                );
                self.gl.BindBuffer(gl::ARRAY_BUFFER, 0);

                self.gl.DrawArrays(gl::TRIANGLES, 0, 6);

                x += (ch.advance >> 6) as f32;
            }
            
            self.gl.BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}