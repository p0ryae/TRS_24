use crate::renderer::gl;

const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("./shaders/shader-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("./shaders/shader-frag.glsl");

pub fn create_init_shader(gl: gl::Gl) -> gl::types::GLuint {
    unsafe {
        let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
        let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

        let program = gl.CreateProgram();

        gl.AttachShader(program, vertex_shader);
        gl.AttachShader(program, fragment_shader);

        gl.LinkProgram(program);

        gl.DeleteShader(vertex_shader);
        gl.DeleteShader(fragment_shader);

        return program;
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
