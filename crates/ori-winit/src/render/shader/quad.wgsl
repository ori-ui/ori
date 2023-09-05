struct Uniforms {
	resolution: vec2<f32>,
	translation: vec2<f32>,
	matrix: mat2x2<f32>,
	rect_min: vec2<f32>,
	rect_max: vec2<f32>,
	color: vec4<f32>,
    border_radius: vec4<f32>,
    border_width: vec4<f32>,
	border_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
	@location(0) position: vec2<f32>,
	@location(1) uv: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) clip: vec4<f32>,
	@location(0) position: vec2<f32>,
	@location(1) uv: vec2<f32>,
}

fn screen_to_clip(position: vec2<f32>) -> vec2<f32> {
	return position / uniforms.resolution * vec2<f32>(2.0, -2.0) - vec2<f32>(1.0, -1.0);
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	var position = uniforms.matrix * in.position + uniforms.translation;
	out.clip = vec4<f32>(screen_to_clip(position), 0.0, 1.0);
	out.position = in.position;
	out.uv = in.uv;

	return out;
}

fn quad_distance(
	position: vec2<f32>,
	rect_min: vec2<f32>,
	rect_max: vec2<f32>,
	radius: f32,
) -> f32 {
	let min_distance = rect_min - position + radius;
	let max_distance = position - rect_max + radius;

	let dist = vec2<f32>(
		max(max(min_distance.x, max_distance.x), 0.0),
		max(max(min_distance.y, max_distance.y), 0.0),
	);

	return length(dist);
}

fn select_border_radius(
	position: vec2<f32>, 
	rect_min: vec2<f32>, 
	rect_max: vec2<f32>,
	radi: vec4<f32>,
) -> f32 {
	let center = (rect_min + rect_max) / 2.0;

	let rx = select(radi.x, radi.y, position.x > center.x);
	let ry = select(radi.w, radi.z, position.x > center.x);
	return select(rx, ry, position.y > center.y);
}

fn select_border_width(
	position: vec2<f32>, 
	rect_min: vec2<f32>, 
	rect_max: vec2<f32>,
	width: vec4<f32>,
	radius: f32,
) -> f32 {
	let center = (rect_min + rect_max) / 2.0;
	let diff = position - center;
	var dx = select(
		position.x - rect_min.x - max(width.w, radius), 
		rect_max.x - position.x - max(width.y, radius),
		diff.x > 0.0
	);
	var dy = select(
		position.y - rect_min.y - max(width.x, radius),
		rect_max.y - position.y - max(width.z, radius),
		diff.y > 0.0
	);

	let wx = select(width.w, width.y, diff.x > 0.0);
	let wy = select(width.x, width.z, diff.y > 0.0);
	return max(select(0.0, wx, dx < 0.0), select(0.0, wy, dy < 0.0));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	var color = uniforms.color;

	let border_radius = select_border_radius(
		in.position,
		uniforms.rect_min,
		uniforms.rect_max,
		uniforms.border_radius,
	);

	let border_width = select_border_width(
		in.position,
		uniforms.rect_min,
		uniforms.rect_max,
		uniforms.border_width,
		border_radius,
	);

	if border_width > 0.0 {
		let internal_border = max(border_radius - border_width, 0.0);

		let internal_dist = quad_distance(
			in.position,
			uniforms.rect_min + vec2<f32>(border_width),
			uniforms.rect_max - vec2<f32>(border_width),
			internal_border,
		);

		let border_mix = smoothstep(
			max(internal_border - 0.5, 0.0),
			internal_border + 0.5,
			internal_dist,
		);

		color = mix(color, uniforms.border_color, border_mix);
	}

	let dist = quad_distance(
		in.position,
		uniforms.rect_min,
		uniforms.rect_max,
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
