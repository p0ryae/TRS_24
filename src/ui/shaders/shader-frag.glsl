precision mediump float;

varying vec4 v_color;
varying vec2 tex_coord;

uniform sampler2D tex0;

void main() {
    vec4 sampled = vec4(1.0, 1.0, 1.0, texture2D(tex0, tex_coord).r);
    gl_FragColor = v_color * sampled;
}