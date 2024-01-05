uniform sampler2D image;

varying vec2 v_tex_coord;
varying vec4 v_color;

void main() {
    gl_FragColor = v_color * texture2D(image, v_tex_coord);
}
