precision mediump float;

attribute vec2 position;
attribute vec4 color;
attribute vec2 tex;

varying vec4 v_color;
varying vec3 v_normal;
varying vec2 tex_coord;

uniform mat4 cam_matrix;
uniform mat4 matrix;

void main() {
    gl_Position = cam_matrix * matrix * vec4(position, 0.0, 1.0);
    v_color = color;
    tex_coord = tex;
}
