#![deny(missing_docs)]

//! Glow renderer for Ori.

use std::{collections::HashMap, ffi, mem, slice};

use glow::HasContext;
use ori_core::{
    canvas::{
        AntiAlias, Canvas, Color, Curve, CurveSegment, FillRule, Paint, Primitive, Shader, Stroke,
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

const VERB_LINE: u8 = 1;
const VERB_QUAD: u8 = 2;
const VERB_CUBIC: u8 = 3;

const NON_ZERO_BIT: u32 = 1 << 31;

unsafe fn slice_as_bytes<T>(slice: &[T]) -> &[u8] {
    slice::from_raw_parts(slice.as_ptr() as *const u8, mem::size_of_val(slice))
}

struct Mask {
    texture: glow::Texture,
    framebuffer: glow::Framebuffer,
}

impl Mask {
    unsafe fn new(gl: &glow::Context, width: u32, height: u32) -> Self {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::R8 as i32,
            width as i32,
            height as i32,
            0,
            glow::RED,
            glow::UNSIGNED_BYTE,
            None,
        );

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );

        let framebuffer = gl.create_framebuffer().unwrap();
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(texture),
            0,
        );
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);

        Self {
            texture,
            framebuffer,
        }
    }
}

/// A glow renderer.
pub struct GlowRenderer {
    gl: glow::Context,
    program: glow::Program,
    width: u32,
    height: u32,
    points: Vec<[f32; 2]>,
    bands: Vec<Vec<[u32; 2]>>,
    band_data: Vec<[u32; 2]>,
    instances: Vec<Instance>,
    point_buffer: glow::Texture,
    band_buffer: glow::Texture,
    point_buffer_height: usize,
    band_buffer_height: usize,
    instance_buffer: glow::Buffer,
    vertex_array: glow::VertexArray,
    images: HashMap<WeakImage, glow::Texture>,
    masks: Vec<Mask>,
    mask: Option<usize>,
    default_image: glow::Texture,
    active_image: Option<glow::Texture>,
    scratch_curve: Curve,
}

impl Drop for GlowRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_texture(self.point_buffer);
            self.gl.delete_texture(self.band_buffer);
            self.gl.delete_buffer(self.instance_buffer);
            self.gl.delete_vertex_array(self.vertex_array);

            for texture in self.images.values() {
                self.gl.delete_texture(*texture);
            }

            self.clear_masks();
            self.gl.delete_texture(self.default_image);
        }
    }
}

impl GlowRenderer {
    const TEXTURE_BUFFER_WIDTH: usize = 2048;
    const MAX_INSTANCES: usize = 256;
    const MAX_BANDS: usize = 256;

    /// # Safety
    /// - This can never truly be safe, this is loading opengl functions, here be dragons.
    pub unsafe fn new(loader: impl FnMut(&str) -> *const ffi::c_void) -> Result<Self, GlError> {
        let gl = glow::Context::from_loader_function(loader);
        let program = Self::create_program(
            &gl,
            include_str!("shader.vert"),
            include_str!("shader.frag"),
        )?;

        let point_buffer = Self::create_point_buffer(&gl, 1);
        let band_buffer = Self::create_band_buffer(&gl, 1);
        let instance_buffer = gl.create_buffer()?;

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(instance_buffer));
        gl.buffer_data_size(
            glow::ARRAY_BUFFER,
            size_of::<Instance>() as i32 * Self::MAX_INSTANCES as i32,
            glow::STATIC_DRAW,
        );

        let vertex_array = Self::create_vertex_array(&gl, instance_buffer).unwrap();

        let default_data = ImageData::new(vec![255; 4], 1, 1);
        let default_image = Self::create_image(&gl, &default_data);

        if gl.get_error() != glow::NO_ERROR {
            panic!("OpenGL error");
        }

        Ok(Self {
            gl,
            program,
            width: 0,
            height: 0,
            points: Vec::new(),
            bands: Vec::with_capacity(Self::MAX_BANDS),
            band_data: Vec::new(),
            instances: Vec::with_capacity(Self::MAX_INSTANCES),
            point_buffer,
            band_buffer,
            point_buffer_height: 1,
            band_buffer_height: 1,
            instance_buffer,
            vertex_array,
            images: HashMap::new(),
            masks: Vec::new(),
            mask: None,
            default_image,
            active_image: None,
            scratch_curve: Curve::new(),
        })
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
    ) -> Result<(), GlError> {
        self.idle();

        if self.width != width || self.height != height {
            self.clear_masks();
        }

        self.width = width;
        self.height = height;
        self.mask = None;

        self.gl.clear_color(color.r, color.g, color.b, color.a);
        self.gl.clear(glow::COLOR_BUFFER_BIT);

        self.gl.viewport(0, 0, width as i32, height as i32);

        self.gl.enable(glow::BLEND);
        self.gl.blend_equation(glow::FUNC_ADD);
        self.gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);

        let x_scale = 2.0 / width as f32;
        let y_scale = 2.0 / height as f32;
        let scale = Vector::new(x_scale, -y_scale);

        let transform = Affine {
            matrix: Matrix::from_scale(scale * scale_factor),
            translation: Vector::new(-1.0, 1.0),
        };

        for primitive in canvas.primitives() {
            self.draw_primitive(primitive, transform)?;
        }

        self.dispatch();

        if self.gl.get_error() != glow::NO_ERROR {
            panic!("OpenGL error");
        }

        Ok(())
    }

    unsafe fn clear_masks(&mut self) {
        for mask in self.masks.drain(..) {
            self.gl.delete_texture(mask.texture);
            self.gl.delete_framebuffer(mask.framebuffer);
        }
    }

    unsafe fn create_point_buffer(gl: &glow::Context, height: u32) -> glow::Texture {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RG32F as i32,
            Self::TEXTURE_BUFFER_WIDTH as i32,
            height as i32,
            0,
            glow::RG,
            glow::FLOAT,
            None,
        );

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );

        texture
    }

    unsafe fn create_band_buffer(gl: &glow::Context, height: u32) -> glow::Texture {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RG32UI as i32,
            Self::TEXTURE_BUFFER_WIDTH as i32,
            height as i32,
            0,
            glow::RG_INTEGER,
            glow::UNSIGNED_INT,
            None,
        );

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );

        texture
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

    unsafe fn create_program(
        gl: &glow::Context,
        vert: &str,
        frag: &str,
    ) -> Result<glow::Program, GlError> {
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

                    let index = self.mask.map_or(0, |m| m + 1);

                    if index >= self.masks.len() {
                        let mask = Mask::new(&self.gl, self.width, self.height);
                        self.masks.push(mask);
                    }

                    let gpu_mask = &self.masks[index];
                    (self.gl).bind_framebuffer(glow::FRAMEBUFFER, Some(gpu_mask.framebuffer));
                    self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
                    self.gl.clear(glow::COLOR_BUFFER_BIT);

                    self.gl.disable(glow::BLEND);

                    self.fill_curve(
                        &mask.curve,
                        &mask.fill,
                        &Paint {
                            shader: Shader::Solid(Color::WHITE),
                            anti_alias: AntiAlias::Fast,
                            ..Default::default()
                        },
                        transform,
                    )?;

                    self.dispatch();

                    self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                    self.gl.enable(glow::BLEND);

                    self.mask = Some(index);
                }

                for primitive in primitives.iter() {
                    self.draw_primitive(primitive, transform * *layer_transform)?;
                }

                if mask.is_some() {
                    self.dispatch();

                    match self.mask {
                        Some(0) => self.mask = None,
                        Some(mask) => self.mask = Some(mask - 1),
                        None => unreachable!(),
                    }
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

        let point_buffer_height = self.points.len() / Self::TEXTURE_BUFFER_WIDTH + 1;
        let band_buffer_height = self.band_data.len() / Self::TEXTURE_BUFFER_WIDTH + 1;

        if point_buffer_height > self.point_buffer_height {
            let height = point_buffer_height;

            self.gl.delete_texture(self.point_buffer);

            self.point_buffer_height = height;
            self.point_buffer = Self::create_point_buffer(&self.gl, height as u32);
        }

        if band_buffer_height > self.band_buffer_height {
            let height = band_buffer_height;

            self.gl.delete_texture(self.band_buffer);

            self.band_buffer_height = height;
            self.band_buffer = Self::create_band_buffer(&self.gl, height as u32);
        }

        (self.gl).bind_texture(glow::TEXTURE_2D, Some(self.point_buffer));
        (self.gl).tex_sub_image_2d(
            glow::TEXTURE_2D,
            0,
            0,
            0,
            Self::TEXTURE_BUFFER_WIDTH as i32,
            self.point_buffer_height as i32,
            glow::RG,
            glow::FLOAT,
            glow::PixelUnpackData::Slice(slice_as_bytes(&self.points)),
        );

        (self.gl).bind_texture(glow::TEXTURE_2D, Some(self.band_buffer));
        (self.gl).tex_sub_image_2d(
            glow::TEXTURE_2D,
            0,
            0,
            0,
            Self::TEXTURE_BUFFER_WIDTH as i32,
            self.band_buffer_height as i32,
            glow::RG_INTEGER,
            glow::UNSIGNED_INT,
            glow::PixelUnpackData::Slice(slice_as_bytes(&self.band_data)),
        );

        (self.gl).bind_buffer(glow::ARRAY_BUFFER, Some(self.instance_buffer));
        (self.gl).buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, slice_as_bytes(&self.instances));

        let texture = self.active_image.unwrap_or(self.default_image);
        let mask = match self.mask {
            Some(mask) => self.masks[mask].texture,
            None => self.default_image,
        };

        self.gl.use_program(Some(self.program));

        self.gl.active_texture(glow::TEXTURE0);
        self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        self.gl.active_texture(glow::TEXTURE1);
        self.gl.bind_texture(glow::TEXTURE_2D, Some(mask));

        self.gl.active_texture(glow::TEXTURE2);
        (self.gl).bind_texture(glow::TEXTURE_2D, Some(self.point_buffer));

        self.gl.active_texture(glow::TEXTURE3);
        (self.gl).bind_texture(glow::TEXTURE_2D, Some(self.band_buffer));

        let location = self.gl.get_uniform_location(self.program, "image");
        self.gl.uniform_1_i32(location.as_ref(), 0);

        let location = self.gl.get_uniform_location(self.program, "mask");
        self.gl.uniform_1_i32(location.as_ref(), 1);

        let location = self.gl.get_uniform_location(self.program, "points");
        self.gl.uniform_1_i32(location.as_ref(), 2);

        let location = self.gl.get_uniform_location(self.program, "bands");
        self.gl.uniform_1_i32(location.as_ref(), 3);

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

    fn point_buffer_cap(&self) -> usize {
        Self::TEXTURE_BUFFER_WIDTH * self.point_buffer_height
    }

    fn band_buffer_cap(&self) -> usize {
        Self::TEXTURE_BUFFER_WIDTH * self.band_buffer_height
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

        if self.points.len() >= self.point_buffer_cap()
            || self.band_data.len() >= self.band_buffer_cap()
        {
            self.points.truncate(self.point_buffer_cap());
            self.band_data.truncate(self.band_buffer_cap());
            self.dispatch();

            let (index, count) = self.push_bands(curve);
            band_index = index;
            band_count = count;
        }

        let (image, image_transform, image_offset_opacity) = match paint.shader {
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

        match paint.anti_alias {
            AntiAlias::None => flags |= 0 << 8,
            AntiAlias::Fast => flags |= 4 << 8,
            AntiAlias::Full => flags |= 8 << 8,
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
            image_offset_opacity,
        };

        self.instances.push(instance);

        Ok(())
    }

    unsafe fn push_bands(&mut self, curve: &Curve) -> (u32, u32) {
        let count = curve.bounds().height() / 5.0;
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
