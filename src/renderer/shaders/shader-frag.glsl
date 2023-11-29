precision mediump float;

varying vec3 v_normal;
varying vec3 v_color;
varying vec2 tex_coord;

uniform sampler2D tex0;

void main() {
    vec3 lightDirection = normalize(vec3(1.0, 1.0, 1.0));
    float diffuse = max(dot(normalize(v_normal), lightDirection), 0.0);
    vec3 shadedColor = v_color * diffuse;

    vec4 textureColor = texture2D(tex0, tex_coord);
    
    vec3 finalColor = shadedColor * textureColor.rgb;
    
    gl_FragColor = vec4(finalColor, 1.0);
}
