use crate::renderer::gl;
use std::io::{Cursor, Read};

#[allow(dead_code)]
pub struct Texture {
    gl: gl::Gl,
    texture: gl::types::GLuint,
}

#[allow(dead_code)]
impl Texture {
    pub fn new(
        gl: gl::Gl,
        img: &[u8],
    ) -> Texture {
        unsafe {
            let mut cursor = Cursor::new(img);
            let mut contents = vec![];
            let _ = cursor.read_to_end(&mut contents);

            let mut image_width = 0;
            let mut image_height = 0;
            let mut num_color_channels = 0;

            let bytes = stb_image_rust::stbi_load_from_memory(
                contents.as_mut_ptr(),
                contents.len() as i32,
                &mut image_width,
                &mut image_height,
                &mut num_color_channels,
                stb_image_rust::STBI_rgb_alpha,
            );

            let mut texture: gl::types::GLuint = std::mem::zeroed();
            gl.GenTextures(1, &mut texture);

            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, texture);

            gl.TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_LINEAR.try_into().unwrap(),
            );
            gl.TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST.try_into().unwrap(),
            );
            gl.TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::REPEAT.try_into().unwrap(),
            );
            gl.TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::REPEAT.try_into().unwrap(),
            );

            if num_color_channels == 4 {
                gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    image_width as i32,
                    image_height as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    bytes as *const std::ffi::c_void,
                );
            } else if num_color_channels == 3 {
                gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    image_width as i32,
                    image_height as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    bytes as *const std::ffi::c_void,
                );
            } else if num_color_channels == 1 {
                gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    image_width as i32,
                    image_height as i32,
                    0,
                    gl::RED_BITS,
                    gl::UNSIGNED_BYTE,
                    bytes as *const std::ffi::c_void,
                );
            }

            gl.GenerateMipmap(gl::TEXTURE_2D);

            stb_image_rust::stbi_image_free(bytes);
            gl.BindTexture(gl::TEXTURE_2D, 0);

            Self {
                gl,
                texture
            }
        }
    }
    pub fn bind(&self) {
        unsafe {
            self.gl.ActiveTexture(gl::TEXTURE0);
            self.gl.BindTexture(gl::TEXTURE_2D, self.texture)
        }
    }
    pub fn unbind(&self) {
        unsafe {
            self.gl.BindTexture(gl::TEXTURE_2D, 0)
        }
    }
}
