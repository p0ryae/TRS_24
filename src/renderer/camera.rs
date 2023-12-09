use crate::renderer::gl;

pub struct Camera {
    pub width: f32,
    pub height: f32,
    pub position: nalgebra_glm::Vec3,
    pub orientation: nalgebra_glm::Vec3,
    pub up: nalgebra_glm::Vec3,
    pub speed: f32,
    pub sensitivity: f32,
    pub first_click: bool,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            position: nalgebra_glm::vec3(0.0, 0.0, 3.0),
            orientation: nalgebra_glm::vec3(0.0, 0.0, -1.0),
            up: nalgebra_glm::vec3(0.0, 1.0, 0.0),
            speed: 0.3,
            sensitivity: 0.01,
            first_click: true,
        }
    }

    pub fn adjust(
        &self,
        gl: gl::Gl,
        program: gl::types::GLuint,
        fov_deg: f32,
        near_plane: f32,
        far_plane: f32,
    ) {
        unsafe {
            let view = nalgebra_glm::look_at(
                &self.position,
                &(self.position + self.orientation),
                &self.up,
            );

            let projection: nalgebra_glm::Mat4 = nalgebra_glm::perspective_fov(
                fov_deg,
                self.width,
                self.height,
                near_plane,
                far_plane,
            );

            gl.UniformMatrix4fv(
                gl.GetUniformLocation(program, b"cam_matrix\0".as_ptr() as *const _),
                1,
                gl::FALSE,
                (&projection * &view).as_slice().as_ptr(),
            );
        }
    }
}
