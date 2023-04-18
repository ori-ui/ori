struct Uniforms {
	resolution: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var image: texture_2d<f32>;

@group(1) @binding(1)
var image_sampler: sampler;

struct VertexInput {
	@location(0)
	position: vec2<f32>,
	@location(1)
	uv: vec2<f32>,
	@location(2)
	color: vec4<f32>,
}

struct VertexOutput {
	@builtin(position)
	position: vec4<f32>,
	@location(0)
	uv: vec2<f32>,
	@location(1)
	color: vec4<f32>,
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	let position = in.position / uniforms.resolution * vec2<f32>(2.0, -2.0) - vec2<f32>(1.0, -1.0);
	out.position = vec4<f32>(position, 0.0, 1.0);
	out.uv = in.uv;
	out.color = in.color;

	return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	return in.color * textureSample(image, image_sampler, in.uv);
}
