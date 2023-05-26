use std::sync::{Arc, Mutex};

use glyphon::{
    fontdb::Source, Attrs, Buffer, Family, FontSystem, Metrics, Stretch, Style, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Weight, Wrap,
};
use ori_graphics::{math::Vec2, Rect, TextAlign, TextSection};
use wgpu::{Device, MultisampleState, Queue, RenderPass, TextureFormat};

pub struct Fonts {
    pub font_system: Mutex<FontSystem>,
}

impl Default for Fonts {
    fn default() -> Self {
        Self {
            font_system: Mutex::new(FontSystem::new()),
        }
    }
}

impl Fonts {
    pub fn load_font_data(&self, data: Vec<u8>) {
        let mut font_system = self.font_system.lock().unwrap();
        let source = Source::Binary(Arc::new(data));
        font_system.db_mut().load_font_source(source);
    }

    pub fn create_attrs(section: &TextSection) -> Attrs<'_> {
        let family = match section.font_family {
            Some(ref name) => Family::Name(&name),
            None => Family::SansSerif,
        };

        Attrs {
            color_opt: None,
            family,
            stretch: Stretch::Normal,
            style: Style::Normal,
            weight: Weight::NORMAL,
            metadata: 0,
        }
    }

    pub fn create_buffer(&self, section: &TextSection) -> Buffer {
        let metrics = Metrics {
            font_size: section.scale,
            line_height: section.scale,
        };

        let mut font_system = self.font_system.lock().unwrap();
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_size(&mut font_system, section.rect.width(), f32::INFINITY);
        buffer.set_text(&mut font_system, &section.text, Self::create_attrs(section));

        let wrap = if section.wrap { Wrap::Word } else { Wrap::None };
        buffer.set_wrap(&mut font_system, wrap);

        buffer
    }

    pub fn measure_text(&self, section: &TextSection, buffer: &Buffer) -> Rect {
        // TODO: i have no idea what this is doing
        // this is just a copy paste from
        //
        // https://github.com/iced-rs/iced/blob/master/wgpu/src/text.rs
        let (total_lines, max_with) = buffer
            .layout_runs()
            .enumerate()
            .fold((0, 0.0), |(_, max), (i, buffer)| {
                (i + 1, buffer.line_w.max(max))
            });

        let total_height = total_lines as f32 * buffer.metrics().line_height;

        // here we're getting the font from the first glyph in the first line
        // and then getting the descender from that font to calculate the descent
        // and offsetting the text by that amount
        //
        // this shouldn't be necessary, but it is, due to a bug in cosmic-text
        // https://github.com/pop-os/cosmic-text/issues/123
        let mut font_system = self.font_system.lock().unwrap();

        let font = if let Some(line) = buffer.layout_runs().next() {
            (line.glyphs.get(0)).and_then(|g| font_system.get_font(g.cache_key.font_id))
        } else {
            None
        };

        let descent = if let Some(font) = font {
            let descender = font.rustybuzz().descender();
            let units_per_em = font.rustybuzz().units_per_em();

            let scale = buffer.metrics().font_size / units_per_em as f32;
            descender as f32 * scale
        } else {
            0.0
        };

        Rect {
            min: section.rect.min + Vec2::new(0.0, descent),
            max: section.rect.min + Vec2::new(max_with, total_height),
        }
    }

    pub fn create_area<'a>(
        &'a self,
        section: &'a TextSection,
        buffer: &'a Buffer,
        clip: Option<Rect>,
    ) -> TextArea<'a> {
        let rect = self.measure_text(section, buffer);

        let left = match section.h_align {
            TextAlign::Start => section.rect.left(),
            TextAlign::Center => section.rect.center().x - rect.width() / 2.0,
            TextAlign::End => section.rect.right() - rect.width(),
        };

        let top = match section.v_align {
            TextAlign::Start => section.rect.top(),
            TextAlign::Center => section.rect.center().y - rect.height() / 2.0,
            TextAlign::End => section.rect.bottom() - rect.height(),
        };

        let bounds = match clip {
            Some(clip) => {
                let rect = clip.intersect(section.rect);

                TextBounds {
                    left: rect.left() as i32,
                    top: rect.top() as i32,
                    right: rect.right() as i32,
                    bottom: rect.bottom() as i32,
                }
            }
            None => TextBounds {
                left: section.rect.left() as i32,
                top: section.rect.top() as i32,
                right: section.rect.right() as i32,
                bottom: section.rect.bottom() as i32,
            },
        };

        TextArea {
            buffer,
            left: left.round() as i32,
            top: top.round() as i32,
            bounds,
            default_color: glyphon::Color::rgba(
                (section.color.r * 255.0) as u8,
                (section.color.g * 255.0) as u8,
                (section.color.b * 255.0) as u8,
                (section.color.a * 255.0) as u8,
            ),
        }
    }
}

pub struct TextPipeline {
    text_atlas: TextAtlas,
    swash_cache: SwashCache,
    layers: Vec<TextRenderer>,
}

impl TextPipeline {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        Self {
            text_atlas: TextAtlas::new(device, queue, format),
            swash_cache: SwashCache::new(),
            layers: Vec::new(),
        }
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        layer: usize,
        fonts: &Fonts,
        width: u32,
        height: u32,
        text: &[(&TextSection, Option<Rect>)],
    ) {
        if layer >= self.layers.len() {
            self.layers.resize_with(layer + 1, || {
                TextRenderer::new(
                    &mut self.text_atlas,
                    device,
                    MultisampleState {
                        count: 4,
                        ..Default::default()
                    },
                    None,
                )
            });
        }

        let layer = &mut self.layers[layer];

        // create buffers
        let buffers = text
            .iter()
            .map(|(section, _)| fonts.create_buffer(section))
            .collect::<Vec<_>>();

        // create areas
        let areas = text
            .iter()
            .zip(buffers.iter())
            .map(|((section, clip), buffer)| fonts.create_area(section, buffer, *clip))
            .collect::<Vec<_>>();

        // lock the font_system and prepare the layer
        let mut font_system = fonts.font_system.lock().unwrap();
        layer
            .prepare(
                device,
                queue,
                &mut font_system,
                &mut self.text_atlas,
                glyphon::Resolution { width, height },
                &areas,
                &mut self.swash_cache,
            )
            .unwrap();
    }

    pub fn render<'a>(&'a self, pass: &mut RenderPass<'a>, layer: usize) {
        let layer = &self.layers[layer];
        layer.render(&self.text_atlas, pass).unwrap();
    }
}
