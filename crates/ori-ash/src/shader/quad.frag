#version 450

layout(location = 0) in vec2 v_top_left;
layout(location = 1) in vec2 v_bottom_right;
layout(location = 2) in vec4 v_color;
layout(location = 3) in vec4 v_border_color;
layout(location = 4) in vec4 v_border_radius;
layout(location = 5) in float v_border_width;

layout(location = 0) out vec4 f_color;

layout(binding = 0) uniform Uniforms {
	vec2 resolution;
} uniforms;

float quad_distance(vec2 p, vec2 tl, vec2 br, float radius) {
	vec2 tl_dist = tl - p + vec2(radius);
	vec2 br_dist = p - br + vec2(radius);

	vec2 dist = max(max(tl_dist, br_dist), vec2(0.0));

	return length(dist);
}

float select_border_radius(vec2 p, vec2 tl, vec2 br, vec4 radi) {
	vec2 center = (tl + br) / 2.0;

	float rx = p.x > center.x ? radi.x : radi.z;
	float ry = p.x > center.x ? radi.y : radi.w;

	return p.y > center.y ? rx : ry;
}

void main() {
	vec4 color = v_color;

	float border_radius = select_border_radius(gl_FragCoord.xy, v_top_left, v_bottom_right, v_border_radius);

	if (v_border_width > 0.0) {
		float internal_border = max(border_radius - v_border_width, 0.0);

		float interal_dist = quad_distance(gl_FragCoord.xy, v_top_left, v_bottom_right, internal_border);

		float border_mix = smoothstep(max(internal_border - 0.5, 0.0), internal_border + 0.5, interal_dist);

		color = mix(color, v_border_color, border_mix);
	}

	float dist = quad_distance(gl_FragCoord.xy, v_top_left, v_bottom_right, border_radius);
	float radius_alpha = smoothstep(max(border_radius - 0.5, 0.0), border_radius + 0.5, dist);

	f_color = vec4(color.rgb, color.a * radius_alpha);
}
