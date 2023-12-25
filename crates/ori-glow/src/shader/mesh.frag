#version 330

uniform sampler2D image;

in vec2 v_tex_coord;
in vec4 v_color;

layout(location = 0) out vec4 o_color;

void main() {
    o_color = v_color * texture(image, v_tex_coord);
}
