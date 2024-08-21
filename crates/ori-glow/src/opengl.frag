#version 330

flat in uint v_flags;
flat in uint v_band_index;
in vec2 v_vertex;
in vec4 v_bounds;
in vec4 v_color;
in mat2 v_transform_inv;
in mat2 v_image_transform;
in vec3 v_image_offset_opacity;

out vec4 f_color;

const uint MAX_CURVE_POINTS = 4096u;
const uint MAX_CURVE_BANDS = 4096u;

const uint NON_ZERO_BIT = 1u << 31u;
const uint ANTI_ALIAS_BIT = 1u << 30u;
const uint BAND_COUNT_MASK = 0x000000ffu;

uniform CurvePoints {
    vec2 curve_points[MAX_CURVE_POINTS];
};

uniform CurveBands {
    uvec2 curve_bands[MAX_CURVE_BANDS];
};

uniform Uniforms {
    vec2 resolution;
};

uniform sampler2D image;

const uint VERB_MOVE = 0u;
const uint VERB_LINE = 1u;
const uint VERB_QUAD = 2u;
const uint VERB_CUBIC = 3u;

const float PI = 3.1415926535897932384626433832795;
const float EPSILON = 1.0e-5;
const float NONE = 1.0e21;

float quad_bezier(float a, float b, float c, float t) {
    float s = 1.0 - t;
    return s * s * a + 2.0 * s * t * b + t * t * c;
}

float cubic_bezier(float a, float b, float c, float d, float t) {
    float s = 1.0 - t;
    return s * s * s * a + 3.0 * s * s * t * b + 3.0 * s * t * t * c + t * t * t * d;
}

vec2 square_roots(float a, float b, float c) {
    if (abs(a) < EPSILON) {
        return vec2(-c / b, NONE);
    }

    float d = b * b - 4.0 * a * c;

    if (d < 0.0) {
        return vec2(NONE);
    }

    d = sqrt(d);

    float a2 = 2.0 * a;

    return vec2((-b - d) / a2, (-b + d) / a2);
}

float cbrt(float x) {
    return x < 0.0 ? -pow(-x, 1.0 / 3.0) : pow(x, 1.0 / 3.0);
}

vec3 cube_roots(float a, float b, float c, float d) {
    if (abs(a) < EPSILON) {
        return vec3(square_roots(b, c, d), NONE);
    }

    float third = 1.0 / 3.0;

    float inv_a = 1.0 / a;
    float B = b * (third * inv_a); 
    float C = c * (third * inv_a);
    float D = d * inv_a;

    if (isinf(B) || isinf(C) || isinf(D)) return vec3(NONE);

    float dd = -B * B + C;
    float dc = -C * B + D;
    float db = B * D - C * C;

    float discr = 4.0 * dd * db - dc * dc;
    float de = -2.0 * B * dd + dc;

    /*
    if (abs(discr) < 1e-8) {
        float t1 = sqrt(-dd) * sign(de);
        return vec3(t1 - B, -2.0 * t1 - B, NONE);
    } else 
    */

    if (discr < 0.0) {
        float sq = sqrt(-0.25 * discr);
        float r = -0.5 * de;
        float t1 = cbrt(r + sq) + cbrt(r - sq);
        return vec3(t1 - B, NONE, NONE);
    } else {
        float th = atan(sqrt(discr), -de) * third;

        float thsin = sin(th);
        float thcos = cos(th);

        float r0 = thcos;
        float ss3 = thsin * sqrt(3.0);
        float r1 = 0.5 * (-thcos + ss3);
        float r2 = 0.5 * (-thcos - ss3);
        float t = 2.0 * sqrt(-dd);
        return vec3(t * r0 - B, t * r1 - B, t * r2 - B);
    }
}

bool is_inside_segment(float t) {
    return t >= 0.0 && t < 1.0;
}

uint line_intersection_count(vec2 p0, vec2 p1, vec2 h) {
    if (p0.y < h.y && p1.y < h.y) return 0u;
    if (p0.y > h.y && p1.y > h.y) return 0u;

    float a = p1.y - p0.y;
    float b = p0.y - h.y;

    if (abs(a) < EPSILON || abs(b) < EPSILON) {
        return 0u;
    }

    float t = -b / a;

    bool is_inside = is_inside_segment(t);
    bool is_right = mix(p0.x, p1.x, t) > h.x;

    return is_inside && is_right ? 1u : 0u;
}

bool is_point_on_quad(vec2 p0, vec2 p1, vec2 p2, vec2 h, float t) {
    bool is_inside = is_inside_segment(t);
    bool is_right = quad_bezier(p0.x, p1.x, p2.x, t) > h.x;

    return is_inside && is_right;
}

bool is_point_on_cubic(vec2 p0, vec2 p1, vec2 p2, vec2 p3, vec2 h, float t) {
    if (abs(p0.y - h.y) < EPSILON) return abs(p0.x - h.x) < EPSILON;
    if (abs(p3.y - h.y) < EPSILON) return abs(p3.x - h.x) < EPSILON;

    bool is_inside = is_inside_segment(t);
    bool is_right; 

    if (abs(t) < 1e-10) {
        is_right = p0.x > h.x;
    } else {
        is_right = cubic_bezier(p0.x, p1.x, p2.x, p3.x, t) > h.x;
    }

    return is_inside && is_right;
}

uint quad_intersection_count(vec2 p0, vec2 p1, vec2 p2, vec2 v) {
    if (p0.y < v.y && p1.y < v.y && p2.y < v.y) return 0u;
    if (p0.y > v.y && p1.y > v.y && p2.y > v.y) return 0u;

    float a = p0.y - 2.0 * p1.y + p2.y;
    float b = 2.0 * (p1.y - p0.y);
    float c = p0.y - v.y;

    vec2 roots = square_roots(a, b, c);

    uint count = 0u;

    if (is_point_on_quad(p0, p1, p2, v, roots.x)) count++;
    if (is_point_on_quad(p0, p1, p2, v, roots.y)) count++;

    return count;
}

uint cubic_intersection_count(vec2 p0, vec2 p1, vec2 p2, vec2 p3, vec2 v) {
    if (p0.y < v.y && p1.y < v.y && p2.y < v.y && p3.y < v.y) return 0u;
    if (p0.y > v.y && p1.y > v.y && p2.y > v.y && p3.y > v.y) return 0u;

    float a = -p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y;
    float b = 3.0 * (p0.y - 2.0 * p1.y + p2.y);
    float c = 3.0 * (p1.y - p0.y);
    float d = p0.y - v.y;

    vec3 roots = cube_roots(a, b, c, d);

    uint count = 0u;

    if (is_point_on_cubic(p0, p1, p2, p3, v, roots.x)) count++;
    if (is_point_on_cubic(p0, p1, p2, p3, v, roots.y)) count++;
    if (is_point_on_cubic(p0, p1, p2, p3, v, roots.z)) count++;

    return count;
}

int line_winding_count(vec2 p0, vec2 p1, vec2 v) {
    if (p0.y < v.y && p1.y < v.y) return 0;
    if (p0.y > v.y && p1.y > v.y) return 0;

    float a = p1.y - p0.y;
    float b = p0.y - v.y;

    float t = -b / a;

    bool is_inside = is_inside_segment(t);
    bool is_right = mix(p0.x, p1.x, t) > v.x;

    if (is_inside && is_right) {
        return a > 0.0 ? 1 : -1;
    }

    return 0;
}

int quad_winding_count(vec2 p0, vec2 p1, vec2 p2, vec2 h) {
    if (p0.y < h.y && p1.y < h.y && p2.y < h.y) return 0;
    if (p0.y > h.y && p1.y > h.y && p2.y > h.y) return 0;

    float a = p0.y - 2.0 * p1.y + p2.y;
    float b = 2.0 * (p1.y - p0.y);
    float c = p0.y - h.y;

    vec2 roots = square_roots(a, b, c);

    int winding = 0;

    if (is_point_on_quad(p0, p1, p2, h, roots.x)) {
        float d = 2.0 * a * roots.x + b;
        winding += d > 0.0 ? 1 : -1;
    }

    if (is_point_on_quad(p0, p1, p2, h, roots.y)) {
        float d = 2.0 * a * roots.y + b;
        winding += d > 0.0 ? 1 : -1;
    }

    return winding;
}

int cubic_winding_count(vec2 p0, vec2 p1, vec2 p2, vec2 p3, vec2 h) {
    if (p0.y < h.y && p1.y < h.y && p2.y < h.y && p3.y < h.y) return 0;
    if (p0.y > h.y && p1.y > h.y && p2.y > h.y && p3.y > h.y) return 0;

    float a = -p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y;
    float b = 3.0 * (p0.y - 2.0 * p1.y + p2.y);
    float c = 3.0 * (p1.y - p0.y);
    float d = p0.y - h.y;

    vec3 roots = cube_roots(a, b, c, d);

    int winding = 0;

    if (is_point_on_cubic(p0, p1, p2, p3, h, roots.x)) {
        float d = 3.0 * a * roots.x * roots.x + 2.0 * b * roots.x + c;
        if (abs(d) > EPSILON) winding += d > 0.0 ? 1 : -1;
    } 

    if (is_point_on_cubic(p0, p1, p2, p3, h, roots.y)) {
        float d = 3.0 * a * roots.y * roots.y + 2.0 * b * roots.y + c;
        if (abs(d) > EPSILON) winding += d > 0.0 ? 1 : -1;
    }

    if (is_point_on_cubic(p0, p1, p2, p3, h, roots.z)) {
        float d = 3.0 * a * roots.z * roots.z + 2.0 * b * roots.z + c;
        if (abs(d) > EPSILON) winding += d > 0.0 ? 1 : -1;
    }

    return winding;
}

uint get_band(vec2 v) {
    uint band_count = v_flags & BAND_COUNT_MASK;
    float y = v.y - v_bounds.y;
    uint band = uint(floor(y / v_bounds.w * float(band_count)));
    return min(band, band_count - 1u);
}

bool is_inside_even_odd(vec2 v) {
    uint band = v_band_index + get_band(v);
    uint segment_offset = curve_bands[band].x;
    uint segment_count = curve_bands[band].y;

    uint crossings = 0u;

    vec2 p0 = vec2(0.0);
    vec2 p1 = vec2(0.0);
    vec2 p2 = vec2(0.0);
    vec2 p3 = vec2(0.0);

    for (uint i = 0u; i < segment_count; i++) {
        uvec2 segment = curve_bands[segment_offset + i];
        
        switch (segment.y) {
        case VERB_LINE:
            p0 = curve_points[segment.x + 0u];
            p1 = curve_points[segment.x + 1u];

            crossings += line_intersection_count(p0, p1, v);
            break;

        case VERB_QUAD:
            p0 = curve_points[segment.x + 0u];
            p1 = curve_points[segment.x + 1u];
            p2 = curve_points[segment.x + 2u];

            crossings += quad_intersection_count(p0, p1, p2, v);
            break;

        case VERB_CUBIC:
            p0 = curve_points[segment.x + 0u];
            p1 = curve_points[segment.x + 1u];
            p2 = curve_points[segment.x + 2u];
            p3 = curve_points[segment.x + 3u];

            crossings += cubic_intersection_count(p0, p1, p2, p3, v);
            break;
                
        default:
            break;
        }
    }

    return crossings % 2u == 1u;
}

bool is_inside_non_zero(vec2 v) { 
    uint band = v_band_index + get_band(v);
    uint segment_offset = curve_bands[band].x;
    uint segment_count = curve_bands[band].y;

    int winding = 0;

    vec2 p0 = vec2(0.0);
    vec2 p1 = vec2(0.0);
    vec2 p2 = vec2(0.0);
    vec2 p3 = vec2(0.0);

    for (uint i = 0u; i < segment_count; i++) {
        uvec2 segment = curve_bands[segment_offset + i];
        
        switch (segment.y) {
        case VERB_LINE:
            p0 = curve_points[segment.x + 0u];
            p1 = curve_points[segment.x + 1u];

            winding += line_winding_count(p0, p1, v);
            break;

        case VERB_QUAD:
            p0 = curve_points[segment.x + 0u];
            p1 = curve_points[segment.x + 1u];
            p2 = curve_points[segment.x + 2u];

            winding += quad_winding_count(p0, p1, p2, v);
            break;

        case VERB_CUBIC:
            p0 = curve_points[segment.x + 0u];
            p1 = curve_points[segment.x + 1u];
            p2 = curve_points[segment.x + 2u];
            p3 = curve_points[segment.x + 3u];

            winding += cubic_winding_count(p0, p1, p2, p3, v);
            break;
                
        default:
            break;
        }
    }

    return winding != 0;
}

bool is_inside(vec2 v) {
    if ((v_flags & NON_ZERO_BIT) != 0u) {
        return is_inside_non_zero(v);
    } else {
        return is_inside_even_odd(v);
    }
}

const vec2[] offsets = vec2[](
    vec2(1.0, 5.0) / 6.5 - (3.5 / 6.5),
    vec2(2.0, 2.0) / 6.5 - (3.5 / 6.5),
    vec2(3.0, 6.0) / 6.5 - (3.5 / 6.5),
    vec2(4.0, 3.0) / 6.5 - (3.5 / 6.5),
    vec2(5.0, 4.0) / 6.5 - (3.5 / 6.5),
    vec2(6.0, 1.0) / 6.5 - (3.5 / 6.5)
);

void main() {
    float alpha = 0.0;

    if ((v_flags & ANTI_ALIAS_BIT) != 0u) {
        for (uint i = 0u; i < 6u; i++) {
            vec2 v = v_vertex + offsets[i];
            alpha += is_inside(v) ? 1.0 : 0.0;
        }

        alpha /= 6.0;
    } else {
        alpha = is_inside(v_vertex) ? 1.0 : 0.0;
    }

    vec2 image_size = vec2(textureSize(image, 0));
    vec2 image_uv = v_image_transform * (v_vertex + v_image_offset_opacity.xy);
    vec4 color = texture(image, image_uv / image_size);
    color.a *= v_image_offset_opacity.z;

    if (alpha < 0.01) discard;

    f_color = v_color * color;
    f_color.a *= alpha;
}
