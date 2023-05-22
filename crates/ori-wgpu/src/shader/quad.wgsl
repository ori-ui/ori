struct Uniforms {
	resolution: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
	@location(0) position: vec2<f32>,
	@location(1) top_left: vec2<f32>,
	@location(2) bottom_right: vec2<f32>,
	@location(3) color: vec4<f32>,
	@location(4) border_color: vec4<f32>,
	@location(5) border_radius: vec4<f32>,
	@location(6) border_width: f32,
}

struct VertexOutput {
	@builtin(position) clip: vec4<f32>,
	@location(0) top_left: vec2<f32>,
	@location(1) bottom_right: vec2<f32>,
	@location(2) color: vec4<f32>,
	@location(3) border_color: vec4<f32>,
	@location(4) border_radius: vec4<f32>,
	@location(5) border_width: f32,
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	let position = in.position / uniforms.resolution * vec2<f32>(2.0, -2.0) - vec2<f32>(1.0, -1.0);
	out.clip = vec4<f32>(position, 0.0, 1.0);
	out.top_left = in.top_left;
	out.bottom_right = in.bottom_right;
	out.color = in.color;
	out.border_color = in.border_color;
	out.border_radius = in.border_radius;
	out.border_width = in.border_width;

	return out;
}

fn quad_distance(
	position: vec2<f32>,
	top_left: vec2<f32>,
	bottom_right: vec2<f32>,
	radius: f32,
) -> f32 {
	let top_left_distance = top_left - position + radius;
	let bottom_right_distance = position - bottom_right + radius;

	let dist = vec2<f32>(
		max(max(top_left_distance.x, bottom_right_distance.x), 0.0),
		max(max(top_left_distance.y, bottom_right_distance.y), 0.0),
	);

	return length(dist);
}

fn select_border_radius(
	position: vec2<f32>, 
	top_left: vec2<f32>, 
	bottom_right: vec2<f32>,
	radi: vec4<f32>,
) -> f32 {
	let center = (top_left + bottom_right) / 2.0;

	let rx = select(radi.x, radi.y, position.x > center.x);
	let ry = select(radi.w, radi.z, position.x > center.x);
	return select(rx, ry, position.y > center.y);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	var color = in.color;

	let border_radius = select_border_radius(
		in.clip.xy,
		in.top_left,
		in.bottom_right,
		in.border_radius,
	);

	if in.border_width > 0.0 {
		let internal_border = max(border_radius - in.border_width, 0.0);

		let internal_dist = quad_distance(
			in.clip.xy,
			in.top_left + vec2<f32>(in.border_width),
			in.bottom_right - vec2<f32>(in.border_width),
			internal_border,
		);

		let border_mix = smoothstep(
			max(internal_border - 0.5, 0.0),
			internal_border + 0.5,
			internal_dist,
		);

		color = mix(color, in.border_color, border_mix);
	}

	let dist = quad_distance(
		in.clip.xy,
		in.top_left,
		in.bottom_right,
		border_radius,
	);

	let radius_alpha = 1.0 - smoothstep(
		max(border_radius - 0.5, 0.0),
		border_radius + 0.5,
		dist,
	);

	return vec4<f32>(
		color.x,
		color.y,
		color.z,
		color.w * radius_alpha,
	);
}
