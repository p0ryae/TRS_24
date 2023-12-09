precision mediump float;

varying vec4 v_color;
varying vec2 tex_coord;

uniform sampler2D tex0;

void main() {
    vec4 sampled = texture2D(tex0, tex_coord);
    gl_FragColor = v_color * sampled;
}