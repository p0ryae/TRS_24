use crate::renderer::gl;

pub fn adjust(
    gl: gl::Gl,
    program: gl::types::GLuint,
    width: f32,
    height: f32,
    fov_deg: f32,
    near_plane: f32,
    far_plane: f32,
) {
    unsafe {
        let position: nalgebra_glm::Vec3 = nalgebra_glm::vec3(0.0, 0.0, 2.0);
        let orientation: nalgebra_glm::Vec3 = nalgebra_glm::vec3(0.0, 0.0, -1.0);
        let up: nalgebra_glm::Vec3 = nalgebra_glm::vec3(0.0, 1.0, 0.0);

        let view: nalgebra_glm::Mat4 =
            nalgebra_glm::look_at(&position, &(position + orientation), &up);
        let projection: nalgebra_glm::Mat4 =
            nalgebra_glm::perspective_fov(fov_deg, width, height, near_plane, far_plane);

        gl.UniformMatrix4fv(
            gl.GetUniformLocation(program, b"cam_matrix\0".as_ptr() as *const _),
            1,
            gl::FALSE,
            (&projection * &view).as_slice().as_ptr(),
        );
    }
}
