precision mediump float;

varying vec4 v_color;
varying vec2 tex_coord;

uniform sampler2D tex0;

void main() {
    gl_FragColor = vec4(v_color);
}
