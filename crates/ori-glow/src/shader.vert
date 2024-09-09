#version 300 es
precision highp float;
precision highp int;

layout(location = 0) in vec4 transform;
layout(location = 1) in vec2 translation;
layout(location = 2) in vec4 bounds;
layout(location = 3) in vec4 color;
layout(location = 4) in uint flags;
layout(location = 5) in uint band_index;
layout(location = 6) in vec4 image_transform;
layout(location = 7) in vec3 image_offset_opacity;

flat out uint v_flags;
flat out uint v_band_index;
out vec2 v_uv;
out vec2 v_vertex;
out vec4 v_bounds;
out vec4 v_color;
out mat2 v_transform;
out mat2 v_image_transform;
out vec3 v_image_offset_opacity;

const vec2 rect[6] = vec2[6](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0),
    vec2(0.0, 0.0)
);

void main() {
    mat2 transform = mat2(transform.xy, transform.zw);

    v_vertex = bounds.xy - 4.0 + rect[gl_VertexID] * (bounds.zw + 8.0);
    v_bounds = bounds;
    v_color = color;
    v_flags = flags;
    v_transform = transform;
    v_band_index = band_index;
    // i have no idea why this is necessary, but taking the inverse works
    v_image_transform = inverse(mat2(image_transform.xy, image_transform.zw));
    v_image_offset_opacity = image_offset_opacity;

    vec2 clip = transform * v_vertex + translation;
    v_uv = clip * 0.5 + 0.5;

    gl_Position = vec4(clip, 0.0, 1.0);
}
