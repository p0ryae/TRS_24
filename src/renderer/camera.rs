use crate::renderer::gl;

pub enum ProjectionType {
    Perspective,
    Orthographic,
}

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
            speed: 0.25,
            sensitivity: 0.01,
            first_click: true,
        }
    }

    pub fn adjust(
        &self,
        gl: gl::Gl,
        program: gl::types::GLuint,
        projection_type: ProjectionType,
        fov_deg: f32,
        near_plane: f32,
        far_plane: f32,
    ) {
        unsafe {
            gl.UseProgram(program);

            let view: nalgebra_glm::Mat4 = nalgebra_glm::look_at(
                &self.position,
                &(self.position + self.orientation),
                &self.up,
            );

            let matrix = match projection_type {
                ProjectionType::Perspective => {
                    let projection: nalgebra_glm::Mat4 = nalgebra_glm::perspective_fov(
                        fov_deg,
                        self.width,
                        self.height,
                        near_plane,
                        far_plane,
                    );
                    projection * view
                }
                ProjectionType::Orthographic => {
                    let projection: nalgebra_glm::Mat4 =
                        nalgebra_glm::ortho(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0);
                    projection
                }
            };

            gl.UniformMatrix4fv(
                gl.GetUniformLocation(program, b"cam_matrix\0".as_ptr() as *const _),
                1,
                gl::FALSE,
                matrix.as_slice().as_ptr(),
            );
        }
    }
}
