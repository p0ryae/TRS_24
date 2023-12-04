use crate::renderer::gl;

pub fn create_init_shader(
    gl: gl::Gl,
    vertex_shader_source: &[u8],
    fragment_shader_source: &[u8],
) -> gl::types::GLuint {
    unsafe {
        let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, vertex_shader_source);
        let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, fragment_shader_source);

        let program = gl.CreateProgram();

        gl.AttachShader(program, vertex_shader);
        gl.AttachShader(program, fragment_shader);

        gl.LinkProgram(program);

        gl.DeleteShader(vertex_shader);
        gl.DeleteShader(fragment_shader);

        program
    }
}

unsafe fn create_shader(
    gl: &gl::Gl,
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    let shader = gl.CreateShader(shader);
    let len = source.len() as gl::types::GLint;
    gl.ShaderSource(shader, 1, [source.as_ptr().cast()].as_ptr(), &len);
    gl.CompileShader(shader);
    shader
}
