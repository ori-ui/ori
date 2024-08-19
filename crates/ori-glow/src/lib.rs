#![deny(missing_docs)]

//! Glow renderer for Ori.

use std::{collections::HashMap, ffi, mem, slice};

use glow::HasContext;
use ori_core::{
    canvas::{
        BlendMode, Canvas, Color, Curve, CurveSegment, FillRule, Paint, Primitive, Shader, Stroke,
    },
    image::{ImageData, WeakImage},
    layout::{Affine, Matrix, Point, Vector},
};

/// OpenGL error.
#[derive(Debug)]
pub struct GlError {
    /// Error message.
    pub message: String,
}

impl From<String> for GlError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

#[repr(C)]
#[derive(Debug)]
struct Instance {
    transform: [f32; 4],
    translation: [f32; 2],
    bounds: [f32; 4],
    color: [f32; 4],
    flags: u32,
    band_index: u32,
    image_transform: [f32; 4],
    image_offset_opacity: [f32; 3],
}

#[repr(C)]
#[derive(Debug)]
struct Uniform {
    resolution: [f32; 2],
}

const VERB_LINE: u8 = 1;
const VERB_QUAD: u8 = 2;
const VERB_CUBIC: u8 = 3;

const NON_ZERO_BIT: u32 = 1 << 31;

unsafe fn slice_as_bytes<T>(slice: &[T]) -> &[u8] {
    slice::from_raw_parts(slice.as_ptr() as *const u8, mem::size_of_val(slice))
}

/// A glow renderer.
pub struct GlowRenderer {
    gl: glow::Context,
    program: glow::Program,
    width: u32,
    height: u32,
    stencil: i32,
    points: Vec<[f32; 2]>,
    bands: Vec<Vec<[u32; 2]>>,
    band_data: Vec<[u32; 2]>,
    instances: Vec<Instance>,
    point_buffer: glow::Buffer,
    band_buffer: glow::Buffer,
    uniform_buffer: glow::Buffer,
    instance_buffer: glow::Buffer,
    vertex_array: glow::VertexArray,
    images: HashMap<WeakImage, glow::Texture>,
    default_image: glow::Texture,
    active_image: Option<glow::Texture>,
    scratch_curve: Curve,
}

impl GlowRenderer {
    const MAX_CURVE_POINTS: usize = 4096;
    const MAX_BAND_DATA: usize = 4096;
    const MAX_INSTANCES: usize = 256;
    const MAX_BANDS: usize = 256;

    /// # Safety
    /// - This can never truly be safe, this is loading opengl functions, here be dragons.
    pub unsafe fn new(loader: impl FnMut(&str) -> *const ffi::c_void) -> Self {
        let gl = glow::Context::from_loader_function(loader);
        let program = Self::create_shader_program(&gl).unwrap();

        let point_buffer = gl.create_buffer().unwrap();
        let band_buffer = gl.create_buffer().unwrap();
        let uniform_buffer = gl.create_buffer().unwrap();
        let instance_buffer = gl.create_buffer().unwrap();

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(point_buffer));
        gl.buffer_data_size(
            glow::ARRAY_BUFFER,
            size_of::<[f32; 2]>() as i32 * Self::MAX_CURVE_POINTS as i32,
            glow::DYNAMIC_DRAW,
        );

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(band_buffer));
        gl.buffer_data_size(
            glow::ARRAY_BUFFER,
            size_of::<[u32; 2]>() as i32 * Self::MAX_BAND_DATA as i32,
            glow::DYNAMIC_DRAW,
        );

        gl.bind_buffer(glow::UNIFORM_BUFFER, Some(uniform_buffer));
        gl.buffer_data_size(
            glow::UNIFORM_BUFFER,
            size_of::<Uniform>() as i32,
            glow::DYNAMIC_DRAW,
        );

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(instance_buffer));
        gl.buffer_data_size(
            glow::ARRAY_BUFFER,
            size_of::<Instance>() as i32 * Self::MAX_INSTANCES as i32,
            glow::STATIC_DRAW,
        );

        gl.bind_buffer_base(glow::UNIFORM_BUFFER, 0, Some(point_buffer));
        gl.bind_buffer_base(glow::UNIFORM_BUFFER, 1, Some(band_buffer));
        gl.bind_buffer_base(glow::UNIFORM_BUFFER, 2, Some(uniform_buffer));

        let vertex_array = Self::create_vertex_array(&gl, instance_buffer).unwrap();

        let default_data = ImageData::new(vec![255; 4], 1, 1);
        let default_image = Self::create_image(&gl, &default_data);

        Self {
            gl,
            program,
            width: 0,
            height: 0,
            stencil: 0,
            points: Vec::with_capacity(Self::MAX_CURVE_POINTS),
            bands: Vec::with_capacity(Self::MAX_BANDS),
            band_data: Vec::with_capacity(Self::MAX_BAND_DATA),
            instances: Vec::with_capacity(Self::MAX_INSTANCES),
            point_buffer,
            band_buffer,
            uniform_buffer,
            instance_buffer,
            vertex_array,
            images: HashMap::new(),
            default_image,
            active_image: None,
            scratch_curve: Curve::new(),
        }
    }

    /// # Safety
    /// - This can never truly be safe, this is calling opengl functions, here be dragons.
    pub unsafe fn render(
        &mut self,
        canvas: &Canvas,
        color: Color,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) {
        self.idle();

        self.width = width;
        self.height = height;

        self.stencil = 0;

        self.gl.enable(glow::STENCIL_TEST);
        self.gl.enable(glow::DEPTH_TEST);

        self.gl.clear_color(color.r, color.g, color.b, color.a);
        self.gl.clear_stencil(self.stencil);
        self.gl.stencil_mask(0xFF);

        self.gl.clear(glow::COLOR_BUFFER_BIT);
        self.gl.clear(glow::STENCIL_BUFFER_BIT);

        self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::INCR);
        self.gl.stencil_func(glow::LEQUAL, self.stencil, 0xFF);
        self.gl.stencil_mask(0x00);

        self.gl.viewport(0, 0, width as i32, height as i32);

        self.gl.enable(glow::BLEND);
        self.gl.blend_equation(glow::FUNC_ADD);
        self.gl.blend_func_separate(
            glow::SRC_ALPHA,
            glow::ONE_MINUS_SRC_ALPHA,
            glow::ONE,
            glow::ONE_MINUS_SRC_ALPHA,
        );

        let x_scale = 2.0 / width as f32;
        let y_scale = 2.0 / height as f32;
        let scale = Vector::new(x_scale, -y_scale);

        let transform = Affine {
            matrix: Matrix::from_scale(scale * scale_factor),
            translation: Vector::new(-1.0, 1.0),
        };

        for primitive in canvas.primitives() {
            self.draw_primitive(primitive, transform).unwrap();
        }

        self.dispatch();
    }

    unsafe fn create_image(gl: &glow::Context, data: &ImageData) -> glow::Texture {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            data.width() as i32,
            data.height() as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(data.data()),
        );

        let filter = match data.filter() {
            true => glow::LINEAR,
            false => glow::NEAREST,
        };

        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, filter as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, filter as i32);

        texture
    }

    unsafe fn create_vertex_array(
        gl: &glow::Context,
        instance_buffer: glow::Buffer,
    ) -> Result<glow::VertexArray, GlError> {
        let vertex_array = gl.create_vertex_array().unwrap();

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(instance_buffer));

        let stride = mem::size_of::<Instance>() as i32;

        gl.bind_vertex_array(Some(vertex_array));
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 4, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 16);
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, stride, 24);
        gl.enable_vertex_attrib_array(3);
        gl.vertex_attrib_pointer_f32(3, 4, glow::FLOAT, false, stride, 40);
        gl.enable_vertex_attrib_array(4);
        gl.vertex_attrib_pointer_i32(4, 1, glow::UNSIGNED_INT, stride, 56);
        gl.enable_vertex_attrib_array(5);
        gl.vertex_attrib_pointer_i32(5, 1, glow::UNSIGNED_INT, stride, 60);
        gl.enable_vertex_attrib_array(6);
        gl.vertex_attrib_pointer_f32(6, 4, glow::FLOAT, false, stride, 64);
        gl.enable_vertex_attrib_array(7);
        gl.vertex_attrib_pointer_f32(7, 3, glow::FLOAT, false, stride, 80);

        gl.vertex_attrib_divisor(0, 1);
        gl.vertex_attrib_divisor(1, 1);
        gl.vertex_attrib_divisor(2, 1);
        gl.vertex_attrib_divisor(3, 1);
        gl.vertex_attrib_divisor(4, 1);
        gl.vertex_attrib_divisor(5, 1);
        gl.vertex_attrib_divisor(6, 1);
        gl.vertex_attrib_divisor(7, 1);

        gl.bind_vertex_array(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);

        Ok(vertex_array)
    }

    unsafe fn idle(&mut self) {
        self.images.retain(|weak, &mut texture| {
            if weak.strong_count() == 0 {
                self.gl.delete_texture(texture);
                false
            } else {
                true
            }
        });
    }

    unsafe fn create_shader_program(gl: &glow::Context) -> Result<glow::Program, GlError> {
        let vert = include_str!("opengl.vert");
        let frag = include_str!("opengl.frag");

        let program = gl.create_program()?;

        let vertex = gl.create_shader(glow::VERTEX_SHADER)?;
        gl.shader_source(vertex, vert);

        let fragment = gl.create_shader(glow::FRAGMENT_SHADER)?;
        gl.shader_source(fragment, frag);

        gl.compile_shader(vertex);
        if !gl.get_shader_compile_status(vertex) {
            return Err(GlError::from(gl.get_shader_info_log(vertex)));
        }

        gl.compile_shader(fragment);
        if !gl.get_shader_compile_status(fragment) {
            return Err(GlError::from(gl.get_shader_info_log(fragment)));
        }

        gl.attach_shader(program, vertex);
        gl.attach_shader(program, fragment);

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            return Err(GlError::from(gl.get_program_info_log(program)));
        }

        gl.delete_shader(vertex);
        gl.delete_shader(fragment);

        let points_index = gl.get_uniform_block_index(program, "CurvePoints").unwrap();
        let bands_index = gl.get_uniform_block_index(program, "CurveBands").unwrap();
        let uniform_index = gl.get_uniform_block_index(program, "Uniforms").unwrap();

        gl.uniform_block_binding(program, points_index, 0);
        gl.uniform_block_binding(program, bands_index, 1);
        gl.uniform_block_binding(program, uniform_index, 2);

        gl.use_program(Some(program));

        Ok(program)
    }

    unsafe fn draw_primitive(
        &mut self,
        primitive: &Primitive,
        transform: Affine,
    ) -> Result<(), GlError> {
        #[allow(clippy::single_match)]
        match primitive {
            Primitive::Fill { curve, fill, paint } => {
                self.fill_curve(curve, fill, paint, transform)?;
            }
            Primitive::Stroke {
                curve,
                stroke,
                paint,
            } => {
                self.stroke_curve(curve, stroke, paint, transform)?;
            }
            Primitive::Layer {
                primitives,
                transform: layer_transform,
                mask,
                ..
            } => {
                if let Some(mask) = mask {
                    self.dispatch();
                    self.stencil += 1;
                    self.gl.stencil_mask(0xFF);

                    self.fill_curve(
                        &mask.curve,
                        &mask.fill,
                        &Paint {
                            shader: Shader::Solid(Color::TRANSPARENT),
                            blend: BlendMode::Destination,
                            anti_alias: false,
                        },
                        transform,
                    )?;

                    self.dispatch();
                    self.gl.stencil_mask(0x00);
                    self.gl.stencil_func(glow::LEQUAL, self.stencil, 0xFF);
                }

                for primitive in primitives {
                    self.draw_primitive(primitive, transform * *layer_transform)?;
                }

                if let Some(mask) = mask {
                    self.dispatch();
                    self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::DECR);
                    self.gl.stencil_mask(0xFF);

                    self.fill_curve(
                        &mask.curve,
                        &mask.fill,
                        &Paint {
                            shader: Shader::Solid(Color::TRANSPARENT),
                            blend: BlendMode::Destination,
                            anti_alias: false,
                        },
                        transform,
                    )?;

                    self.dispatch();
                    self.stencil -= 1;
                    self.gl.stencil_mask(0x00);
                    self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::INCR);
                    self.gl.stencil_func(glow::LEQUAL, self.stencil, 0xFF);
                }
            }
        }

        Ok(())
    }

    unsafe fn stroke_curve(
        &mut self,
        curve: &Curve,
        stroke: &Stroke,
        paint: &Paint,
        transform: Affine,
    ) -> Result<(), GlError> {
        let mut scratch_curve = mem::take(&mut self.scratch_curve);
        scratch_curve.clear();
        scratch_curve.stroke_curve(curve, *stroke);

        self.fill_curve(&scratch_curve, &FillRule::NonZero, paint, transform)?;
        self.scratch_curve = scratch_curve;

        Ok(())
    }

    unsafe fn dispatch(&mut self) {
        if self.instances.is_empty() {
            return;
        }

        let uniform = Uniform {
            resolution: [self.width as f32, self.height as f32],
        };

        (self.gl).bind_buffer(glow::UNIFORM_BUFFER, Some(self.uniform_buffer));
        (self.gl).buffer_sub_data_u8_slice(glow::UNIFORM_BUFFER, 0, slice_as_bytes(&[uniform]));

        (self.gl).bind_buffer(glow::ARRAY_BUFFER, Some(self.point_buffer));
        (self.gl).buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, slice_as_bytes(&self.points));

        (self.gl).bind_buffer(glow::ARRAY_BUFFER, Some(self.band_buffer));
        (self.gl).buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, slice_as_bytes(&self.band_data));

        (self.gl).bind_buffer(glow::ARRAY_BUFFER, Some(self.instance_buffer));
        (self.gl).buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, slice_as_bytes(&self.instances));

        self.gl.bind_buffer(glow::UNIFORM_BUFFER, None);

        let texture = self.active_image.unwrap_or(self.default_image);

        self.gl.active_texture(glow::TEXTURE0);
        self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        let location = self.gl.get_uniform_location(self.program, "image");
        self.gl.uniform_1_i32(location.as_ref(), 0);

        self.gl.use_program(Some(self.program));
        self.gl.bind_vertex_array(Some(self.vertex_array));
        (self.gl).draw_arrays_instanced(glow::TRIANGLE_STRIP, 0, 6, self.instances.len() as i32);

        self.gl.bind_vertex_array(None);
        self.gl.use_program(None);

        self.gl.bind_buffer(glow::ARRAY_BUFFER, None);

        self.points.clear();
        self.band_data.clear();
        self.instances.clear();
        self.active_image = None;
    }

    unsafe fn fill_curve(
        &mut self,
        curve: &Curve,
        fill: &FillRule,
        paint: &Paint,
        transform: Affine,
    ) -> Result<(), GlError> {
        if self.instances.len() >= Self::MAX_INSTANCES {
            self.dispatch();
        }

        let (mut band_index, mut band_count) = self.push_bands(curve);

        if self.points.len() >= Self::MAX_CURVE_POINTS
            || self.band_data.len() >= Self::MAX_BAND_DATA
        {
            self.points.truncate(Self::MAX_CURVE_POINTS);
            self.band_data.truncate(Self::MAX_BAND_DATA);
            self.dispatch();

            let (index, count) = self.push_bands(curve);
            band_index = index;
            band_count = count;
        }

        let (image, image_transform, image_data) = match paint.shader {
            Shader::Pattern(ref pattern) => {
                let weak = pattern.image.downgrade();

                let texture = self.images.entry(weak).or_insert_with(|| {
                    //
                    Self::create_image(&self.gl, &pattern.image)
                });

                let transform = pattern.transform.matrix.into();
                let offset_opacity = [
                    pattern.transform.translation.x,
                    pattern.transform.translation.y,
                    pattern.opacity,
                ];

                (Some(*texture), transform, offset_opacity)
            }
            Shader::Solid(_) => (None, Matrix::IDENTITY.into(), [0.0, 0.0, 1.0]),
        };

        if self.active_image != image && !self.instances.is_empty() {
            self.dispatch();
        }

        self.active_image = image;

        let color = match paint.shader {
            Shader::Solid(color) => color,
            _ => Color::WHITE,
        };

        let mut flags = 0;

        if let FillRule::NonZero = fill {
            flags |= NON_ZERO_BIT;
        }

        if paint.anti_alias {
            flags |= 16 << 8;
        } else {
            flags |= 1 << 8;
        }

        flags |= band_count;

        let bounds = curve.bounds();
        let instance = Instance {
            transform: transform.matrix.into(),
            translation: transform.translation.into(),
            bounds: [bounds.min.x, bounds.min.y, bounds.width(), bounds.height()],
            color: color.into(),
            flags,
            band_index,
            image_transform,
            image_offset_opacity: image_data,
        };

        self.instances.push(instance);

        Ok(())
    }

    unsafe fn push_bands(&mut self, curve: &Curve) -> (u32, u32) {
        let count = curve.bounds().height() / 10.0;
        let count = usize::clamp(count.ceil() as usize, 1, Self::MAX_BANDS - 1);

        self.bands.clear();
        self.bands.resize(count, Vec::new());

        let get_band = |p: Point| {
            let y = p.y - curve.bounds().min.y;
            let band = y / curve.bounds().height() * count as f32;
            usize::clamp(band.floor() as usize, 0, count - 1)
        };
        let push_bands = |bands: &mut Vec<Vec<[u32; 2]>>, min: usize, max: usize, point, verb| {
            for band in &mut bands[min..=max] {
                band.push([point as u32, verb]);
            }
        };

        let mut first_point = Point::ZERO;
        let mut b0 = 0;

        for segment in curve.iter() {
            match segment {
                CurveSegment::Move(p0) => {
                    first_point = p0;
                    b0 = get_band(p0);

                    self.points.push(p0.into());
                }
                CurveSegment::Line(p1) => {
                    let b1 = get_band(p1);

                    push_bands(
                        &mut self.bands,
                        usize::min(b0, b1),
                        usize::max(b0, b1),
                        self.points.len() - 1,
                        VERB_LINE as u32,
                    );

                    self.points.push(p1.into());

                    b0 = b1;
                }
                CurveSegment::Quad(p1, p2) => {
                    let b1 = get_band(p1);
                    let b2 = get_band(p2);

                    push_bands(
                        &mut self.bands,
                        usize::min(b0, usize::min(b1, b2)),
                        usize::max(b0, usize::max(b1, b2)),
                        self.points.len() - 1,
                        VERB_QUAD as u32,
                    );

                    self.points.push(p1.into());
                    self.points.push(p2.into());

                    b0 = b2;
                }
                CurveSegment::Cubic(p1, p2, p3) => {
                    let b1 = get_band(p1);
                    let b2 = get_band(p2);
                    let b3 = get_band(p3);

                    push_bands(
                        &mut self.bands,
                        usize::min(b0, usize::min(b1, usize::min(b2, b3))),
                        usize::max(b0, usize::max(b1, usize::max(b2, b3))),
                        self.points.len() - 1,
                        VERB_CUBIC as u32,
                    );

                    self.points.push(p1.into());
                    self.points.push(p2.into());
                    self.points.push(p3.into());

                    b0 = b3;
                }
                CurveSegment::Close => {
                    let b1 = get_band(first_point);

                    push_bands(
                        &mut self.bands,
                        usize::min(b0, b1),
                        usize::max(b0, b1),
                        self.points.len() - 1,
                        VERB_LINE as u32,
                    );

                    self.points.push(first_point.into());

                    b0 = b1;
                }
            }
        }

        let index = self.band_data.len() as u32;
        let mut offset = index + count as u32;
        for band in &self.bands {
            self.band_data.push([offset, band.len() as u32]);
            offset += band.len() as u32;
        }

        for band in &self.bands {
            self.band_data.extend_from_slice(band);
        }

        (index, count as u32)
    }
}
