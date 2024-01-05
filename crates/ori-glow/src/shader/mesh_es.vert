uniform vec2 resolution;

layout(location = 0) attribute vec2 m_position;
layout(location = 1) attribute vec2 m_tex_coord;
layout(location = 2) attribute vec4 m_color;

varying vec2 v_tex_coord;
varying vec4 v_color;

vec2 screenToClip(vec2 position) {
    return position / resolution * vec2(2.0, -2.0) - vec2(1.0, -1.0);
}

void main() {
    gl_Position = vec4(screenToClip(m_position), 0.0, 1.0);
    v_tex_coord = m_tex_coord;
    v_color = m_color;
}
