precision mediump float;

attribute vec3 position;
attribute vec3 normal;
attribute vec3 color;
attribute vec2 tex;

varying vec3 v_color;
varying vec3 v_normal;
varying vec2 tex_coord;

uniform mat4 cam_matrix;
uniform mat4 matrix;

void main() {
    gl_Position = cam_matrix * matrix * vec4(position, 1.0);
    v_color = color;
    v_normal = normal;
    tex_coord = tex;
}
