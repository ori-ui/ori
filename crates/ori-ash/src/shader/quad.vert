#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 top_left;
layout(location = 2) in vec2 bottom_right;
layout(location = 3) in vec4 color;
layout(location = 4) in vec4 border_color;
layout(location = 5) in vec4 border_radius;
layout(location = 6) in float border_width;

layout(location = 0) out vec2 v_top_left;
layout(location = 1) out vec2 v_bottom_right;
layout(location = 2) out vec4 v_color;
layout(location = 3) out vec4 v_border_color;
layout(location = 4) out vec4 v_border_radius;
layout(location = 5) out float v_border_width;

layout(binding = 0) uniform Uniforms {
	vec2 resolution;
} uniforms;

void main() {
	vec2 p = position / uniforms.resolution * 2.0 - 1.0;
	gl_Position = vec4(p, 0.0, 1.0);
	v_top_left = top_left;
	v_bottom_right = bottom_right;
	v_color = color;
	v_border_color = border_color;
	v_border_radius = border_radius;
	v_border_width = border_width;
}

