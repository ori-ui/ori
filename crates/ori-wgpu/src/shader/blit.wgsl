var<private> vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
	vec2<f32>(-1.0, -1.0),
	vec2<f32>( 1.0, -1.0),
	vec2<f32>(-1.0,  1.0),
	vec2<f32>( 1.0, -1.0),
	vec2<f32>(-1.0,  1.0),
	vec2<f32>( 1.0,  1.0),
);

@group(0) @binding(0)
var source: texture_2d<f32>;

@group(0) @binding(1)
var source_sampler: sampler;

struct VertexOutput {
	@builtin(position) clip: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
	var out: VertexOutput;

	out.uv = vertices[index] * vec2<f32>(0.5, -0.5) + 0.5;
	out.clip = vec4<f32>(vertices[index], 0.0, 1.0);

	return out;
}

@fragment
fn fragment(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
	return textureSample(source, source_sampler, uv);
}
