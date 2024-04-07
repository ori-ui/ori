#version 300 es

precision highp float;

uniform vec2 resolution;

layout(location = 0) in vec2 m_position;
layout(location = 1) in vec2 m_tex_coord;
layout(location = 2) in vec4 m_color;

out vec2 v_tex_coord;
out vec4 v_color;

vec2 screenToClip(vec2 position) {
    return position / resolution * vec2(2.0, -2.0) - vec2(1.0, -1.0);
}

void main() {
    gl_Position = vec4(screenToClip(m_position), 0.0, 1.0);
    v_tex_coord = m_tex_coord;
    v_color = m_color;
}
