use std::{collections::HashMap, mem};

use glow::HasContext;
use ori_core::{
    canvas::{Batch, Vertex},
    image::{Image, Texture, WeakImage},
    layout::Size,
};

use super::GlowError;

#[derive(Debug)]
struct PreparedBatch {
    vertex_buffer: glow::Buffer,
    vertex_array: glow::VertexArray,
    index_buffer: glow::Buffer,
}

impl PreparedBatch {
    unsafe fn new(gl: &glow::Context) -> Result<Self, GlowError> {
        let (vbo, vao) = Self::create_vertex_array(gl)?;
        let index_buffer = gl.create_buffer().map_err(GlowError::Gl)?;

        Ok(Self {
            vertex_buffer: vbo,
            vertex_array: vao,
            index_buffer,
        })
    }

    unsafe fn create_vertex_array(
        gl: &glow::Context,
    ) -> Result<(glow::Buffer, glow::VertexArray), GlowError> {
        let vertex_buffer = gl.create_buffer().map_err(GlowError::Gl)?;
        let vertex_array = gl.create_vertex_array().map_err(GlowError::Gl)?;

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
        gl.bind_vertex_array(Some(vertex_array));

        let size = mem::size_of::<Vertex>() as i32;

        gl.enable_vertex_attrib_array(0);
        gl.enable_vertex_attrib_array(1);
        gl.enable_vertex_attrib_array(2);

        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, size, 0);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, size, 8);
        gl.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, size, 16);

        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);

        Ok((vertex_buffer, vertex_array))
    }

    unsafe fn delete(&self, gl: &glow::Context) {
        gl.delete_buffer(self.vertex_buffer);
        gl.delete_buffer(self.index_buffer);
        gl.delete_vertex_array(self.vertex_array);
    }
}

#[derive(Debug)]
pub struct MeshRender {
    batches: Vec<PreparedBatch>,
    program: glow::Program,
    textures: HashMap<WeakImage, glow::Texture>,
    fallback: glow::Texture,
}

impl MeshRender {
    unsafe fn create_shader(
        gl: &glow::Context,
        shader_type: u32,
        source: &str,
    ) -> Result<glow::Shader, GlowError> {
        let shader = gl.create_shader(shader_type).map_err(GlowError::Gl)?;

        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let log = gl.get_shader_info_log(shader);
            return Err(GlowError::Gl(log));
        }

        Ok(shader)
    }

    unsafe fn create_program(gl: &glow::Context) -> Result<glow::Program, GlowError> {
        let program = gl.create_program().map_err(GlowError::Gl)?;

        #[cfg(not(target_os = "android"))]
        let (vert_source, frag_source) = (
            include_str!("shader/mesh_gl.vert"),
            include_str!("shader/mesh_gl.frag"),
        );

        #[cfg(target_os = "android")]
        let (vert_source, frag_source) = (
            include_str!("shader/mesh_es.vert"),
            include_str!("shader/mesh_es.frag"),
        );

        let vert_shader = Self::create_shader(gl, glow::VERTEX_SHADER, vert_source)?;
        let frag_shader = Self::create_shader(gl, glow::FRAGMENT_SHADER, frag_source)?;

        gl.attach_shader(program, vert_shader);
        gl.attach_shader(program, frag_shader);
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            return Err(GlowError::Gl(log));
        }

        gl.detach_shader(program, vert_shader);
        gl.detach_shader(program, frag_shader);
        gl.delete_shader(vert_shader);
        gl.delete_shader(frag_shader);

        Ok(program)
    }

    pub unsafe fn new(gl: &glow::Context) -> Result<Self, GlowError> {
        let program = Self::create_program(gl)?;
        let fallback = Self::create_texture(gl, &Image::default())?;

        Ok(Self {
            batches: Vec::new(),
            program,
            textures: HashMap::new(),
            fallback,
        })
    }

    unsafe fn create_texture(
        gl: &glow::Context,
        image: &Image,
    ) -> Result<glow::Texture, GlowError> {
        let texture = gl.create_texture().map_err(GlowError::Gl)?;

        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        let filter = if image.filter() {
            glow::LINEAR
        } else {
            glow::NEAREST
        };

        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, filter as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, filter as i32);

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            image.width() as i32,
            image.height() as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(image.pixels()),
        );

        gl.bind_texture(glow::TEXTURE_2D, None);

        Ok(texture)
    }

    unsafe fn get_texture(
        &mut self,
        gl: &glow::Context,
        texture: &Option<Texture>,
    ) -> Result<glow::Texture, GlowError> {
        match texture {
            Some(Texture::Image(image)) => match self.textures.get(&image.downgrade()) {
                Some(&texture) => Ok(texture),
                None => {
                    let texture = Self::create_texture(gl, image)?;
                    self.textures.insert(image.downgrade(), texture);
                    Ok(texture)
                }
            },
            _ => Ok(self.fallback),
        }
    }

    unsafe fn clean_textures(&mut self, gl: &glow::Context) {
        let mut invalid = Vec::new();

        for (image, texture) in self.textures.iter() {
            if image.strong_count() == 0 {
                invalid.push((image.clone(), *texture));
            }
        }

        for (image, texture) in invalid {
            gl.delete_texture(texture);
            self.textures.remove(&image);
        }
    }

    pub unsafe fn clean(&mut self, gl: &glow::Context) {
        self.clean_textures(gl);
    }

    pub unsafe fn render_batch(
        &mut self,
        gl: &glow::Context,
        batch: &Batch,
        logical_size: Size,
        scale_factor: f32,
    ) -> Result<(), GlowError> {
        if self.batches.len() <= batch.index {
            self.batches.push(PreparedBatch::new(gl)?);
        }

        let texture = self.get_texture(gl, &batch.mesh.texture)?;

        let prepared = &mut self.batches[batch.index];

        gl.use_program(Some(self.program));

        let location = gl.get_uniform_location(self.program, "resolution");
        gl.uniform_2_f32(location.as_ref(), logical_size.width, logical_size.height);

        let location = gl.get_uniform_location(self.program, "image");
        gl.uniform_1_i32(location.as_ref(), 0);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(prepared.vertex_buffer));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            batch.mesh.vertex_bytes(),
            glow::STREAM_DRAW,
        );

        gl.bind_vertex_array(Some(prepared.vertex_array));

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(prepared.index_buffer));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            batch.mesh.index_bytes(),
            glow::STREAM_DRAW,
        );

        // winit on wasm32 is just strange, this should be needed
        // FIXME: this is a hack, i don't like it
        #[cfg(target_arch = "wasm32")]
        let scale_factor = 1.0;

        // opengl is bad... i don't like having to do this...
        let scissor_y = logical_size.height - batch.clip.max.y;

        let clip_x = batch.clip.min.x * scale_factor;
        let clip_y = scissor_y * scale_factor;
        let clip_width = batch.clip.width() * scale_factor;
        let clip_height = batch.clip.height() * scale_factor;

        gl.scissor(
            clip_x.round() as i32,
            clip_y.round() as i32,
            clip_width.round() as i32,
            clip_height.round() as i32,
        );

        let index_count = batch.mesh.indices.len() as i32;
        gl.draw_elements(glow::TRIANGLES, index_count, glow::UNSIGNED_INT, 0);

        // unbind everything
        gl.use_program(None);
        gl.bind_vertex_array(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

        Ok(())
    }

    pub unsafe fn delete(&self, gl: &glow::Context) {
        gl.delete_program(self.program);
        gl.delete_texture(self.fallback);

        for batch in self.batches.iter() {
            batch.delete(gl);
        }

        for texture in self.textures.values() {
            gl.delete_texture(*texture);
        }
    }
}
